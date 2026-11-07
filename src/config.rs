//! Configuration (TOML). Top-level keys must come before any `[table]`.
//! See `config.example.toml` for a commented example.

use anyhow::{ensure, Context, Result};
use chrono_tz::Tz;
use serde::Deserialize;
use std::{
    fs::{self, Permissions},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

/// Default location of the config file when `--config` is not given.
pub fn default_config_path() -> PathBuf {
    let base = match std::env::consts::OS {
        "freebsd" => "/usr/local/etc",
        _ => "/etc",
    };
    PathBuf::from(base).join("clubmirror").join("config.toml")
}

const fn default_timezone() -> Tz {
    Tz::UTC
}

const fn default_permissions() -> u32 {
    0o644
}

const fn default_routes_ttl() -> u64 {
    86_400
}

fn default_template_extensions() -> Vec<String> {
    [
        "html",
        "htm",
        "xml",
        "txt",
        "md",
        "json",
        "rss",
        "atom",
        "ics",
        "webmanifest",
    ]
    .map(String::from)
    .into()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Directory for cached state (access token, rotated refresh token).
    /// Created with mode 700 if missing; must be owned by the user running clubmirror.
    pub cache_path: PathBuf,

    /// Default IANA timezone (e.g. "Europe/Brussels"), used by clubs that don't set their own.
    #[serde(default = "default_timezone")]
    pub timezone: Tz,

    /// Strava API credentials and API-related options.
    pub api: ApiConfig,

    /// What and how we generate. Every option is optional.
    #[serde(default)]
    pub generated: GeneratedConfig,

    /// The clubs to mirror, one `[[club]]` table each.
    #[serde(rename = "club")]
    pub clubs: Vec<ClubConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    /// Strava application client id.
    pub client_id: u64,

    /// Strava application client secret.
    pub client_secret: String,

    /// Initial refresh token. If Strava rotates it, the new one is persisted in `cache_path`
    /// and takes precedence until you change this value.
    pub refresh_token: String,

    /// Only warn (instead of failing) when the API answers 404.
    #[serde(default)]
    pub warn_on_404: bool,

    /// Debug: save the last raw API response of group events in this file.
    pub log_response_path: Option<PathBuf>,

    /// TEMPORARY quirk (will become a proper debug option): read the events from this JSON
    /// file instead of asking the API, for every club. Avoids spamming the API.
    pub quirk_local_json: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedConfig {
    /// Octal mode of generated files, e.g. `0o644`.
    #[serde(default = "default_permissions")]
    pub permissions: u32,

    /// Owner of generated files (name or uid). Needs root (or matching ownership) to differ
    /// from the current user.
    pub user: Option<String>,

    /// Group of generated files (name or gid).
    pub group: Option<String>,

    /// Template files with one of these extensions are rendered with Tera. Every other file
    /// of the template directory (CSS, images, fonts, ...) is copied verbatim.
    /// Files and directories whose name starts with `_` are only usable as `extends` /
    /// `include` sources and never written to the output. Dotfiles are ignored.
    #[serde(default = "default_template_extensions")]
    pub template_extensions: Vec<String>,

    /// Download the routes (GPX/TCX) of events in this directory. Disabled if unset.
    pub routes_path: Option<PathBuf>,

    /// Seconds before a downloaded route is downloaded again.
    #[serde(default = "default_routes_ttl")]
    pub routes_ttl: u64,

    /// Public base URL of `routes_path`, given to the templates as `routes_url`.
    pub routes_url: Option<String>,

    /// Write one ICS calendar per club (`<club id>.ics`) in this directory. Disabled if unset.
    pub calendars_path: Option<PathBuf>,

    /// Public base URL of `calendars_path`, given to the templates as `calendars_url`
    /// (and `calendar_url` for the calendar of the club being rendered).
    pub calendars_url: Option<String>,

    /// TEMPORARY quirk (will become a proper debug option): save the resulting events of each
    /// club as `<club id>.json` in this directory.
    pub quirk_save_json: Option<PathBuf>,
}

impl Default for GeneratedConfig {
    fn default() -> Self {
        Self {
            permissions: default_permissions(),
            user: None,
            group: None,
            template_extensions: default_template_extensions(),
            routes_path: None,
            routes_ttl: default_routes_ttl(),
            routes_url: None,
            calendars_path: None,
            calendars_url: None,
            quirk_save_json: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClubConfig {
    /// Strava club id.
    pub id: u64,

    /// Display name (calendar name, `club_name` in templates). Defaults to the name
    /// reported by Strava, or "Club <id>" if the club has no events.
    pub name: Option<String>,

    /// Directory containing this club's templates.
    pub template_dir: PathBuf,

    /// Directory where this club's site is generated.
    pub output_dir: PathBuf,

    /// Timezone of the club; defaults to the global `timezone`.
    pub timezone: Option<Tz>,

    /// Hide events whose last occurrence is older than this many days.
    pub past_days: Option<u32>,

    /// Hide events whose first occurrence is more than this many days ahead.
    pub future_days: Option<u32>,

    /// TEMPORARY quirk (will be replaced): sort events by their first occurrence.
    #[serde(default)]
    pub sort_events: bool,
}

impl Config {
    /// Load the config file. It holds secrets, so it must be private to the current user.
    pub fn load(path: &Path) -> Result<Self> {
        let metadata =
            fs::metadata(path).with_context(|| format!("cannot read {}", path.display()))?;
        ensure!(
            metadata.is_file(),
            "{} is not a regular file",
            path.display()
        );
        ensure!(
            metadata.mode() & 0o077 == 0,
            "{} must not be accessible by group/others (chmod 600)",
            path.display()
        );
        ensure!(
            metadata.uid() == users::get_effective_uid(),
            "{} must be owned by the user running clubmirror",
            path.display()
        );

        let config: Config = toml::from_str(&fs::read_to_string(path)?)
            .with_context(|| format!("cannot parse {}", path.display()))?;
        ensure!(!config.clubs.is_empty(), "no [[club]] defined");

        Ok(config)
    }

    /// Create the cache directory if needed and check it is private to the current user.
    pub fn prepare_cache(&self) -> Result<()> {
        if !self.cache_path.exists() {
            fs::create_dir_all(&self.cache_path)?;
            fs::set_permissions(&self.cache_path, Permissions::from_mode(0o700))?;
        }

        let metadata = fs::metadata(&self.cache_path)?;
        ensure!(
            metadata.is_dir(),
            "{} is not a directory",
            self.cache_path.display()
        );
        ensure!(
            metadata.mode() & 0o777 == 0o700,
            "{} is not permission 700",
            self.cache_path.display()
        );
        ensure!(
            metadata.uid() == users::get_effective_uid(),
            "{} must be owned by the user running clubmirror",
            self.cache_path.display()
        );

        Ok(())
    }
}

impl ClubConfig {
    /// Timezone of this club, falling back to the given default.
    pub fn timezone_or(&self, default: Tz) -> Tz {
        self.timezone.unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_config_parses() {
        let config: Config = toml::from_str(include_str!("../config.toml.sample")).unwrap();
        assert!(!config.clubs.is_empty());
        assert_eq!(config.generated.permissions, 0o644);
    }

    #[test]
    fn local_example_config_parses() {
        let config: Config = toml::from_str(include_str!("../examples/config.toml")).unwrap();
        assert!(config.api.quirk_local_json.is_some());
    }

    #[test]
    fn minimal_config_uses_defaults() {
        let config: Config = toml::from_str(
            r#"
            cache_path = "/tmp/c"
            [api]
            client_id = 1
            client_secret = "s"
            refresh_token = "r"
            [[club]]
            id = 42
            template_dir = "/t"
            output_dir = "/o"
            "#,
        )
        .unwrap();
        assert_eq!(config.timezone, Tz::UTC);
        assert_eq!(config.generated.routes_ttl, 86_400);
        assert!(config
            .generated
            .template_extensions
            .contains(&"html".to_string()));
        assert_eq!(config.clubs[0].timezone_or(config.timezone), Tz::UTC);
    }
}
