pub mod strava;
pub use strava::*;

use anyhow::Result;
use clap::{arg, command, value_parser, ArgAction};
use log::*;
use serde::{Deserialize, Serialize};
use std::fs::{self, Permissions};
use std::io::Write;
use std::os::unix::fs::{chown, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::{Duration, SystemTime};
use toml;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Config {
    client_id: String,
    client_secret: String, // to obtain authorization codes, might not be needed either
    refresh_token: String,

    cache_path: PathBuf,
    template_path: PathBuf,
    generated_site_path: PathBuf,

    clubs: Vec<String>, // IDs of the clubs
}

impl Config {
    fn new() -> Result<Self> {
        let mut config_path: PathBuf = match std::env::consts::OS {
            "freebsd" => "/usr/local/etc".into(),
            _ => "/etc".into(),
        };
        config_path.push("lbs-mirror.toml");

        // check file permissions for security
        let metadata = fs::metadata(&config_path)?;
        anyhow::ensure!(
            metadata.is_file(),
            "{} is not a regular file",
            config_path.display()
        );
        anyhow::ensure!(
            metadata.mode() & 0o777 == 0o600,
            "{} is not permission 600",
            config_path.display()
        );
        anyhow::ensure!(
            metadata.uid() == 0 && metadata.gid() == 0,
            "{} should be owned by 0:0",
            config_path.display()
        );

        let config_str = fs::read_to_string(config_path)?;
        let config_file: Config = toml::from_str(&config_str)?;
        debug!("config loaded");

        Ok(config_file)
    }

    fn check_cache_permission(&self) -> Result<()> {
        let metadata = fs::metadata(&self.cache_path)?;
        anyhow::ensure!(
            metadata.is_dir(),
            "{} is not a directory",
            self.cache_path.display()
        );
        anyhow::ensure!(
            metadata.mode() & 0o777 == 0o600,
            "{} is not permission 600",
            self.cache_path.display()
        );
        anyhow::ensure!(
            metadata.uid() == 0 && metadata.gid() == 0,
            "{} should be owned by 0:0",
            self.cache_path.display()
        );

        Ok(())
    }
}

struct App {
    config: Config,
    access_token_path: PathBuf,
    api: StravaApi,
}

impl App {
    fn new(config: Config, api: StravaApi) -> Self {
        Self {
            access_token_path: config.cache_path.clone().join("access_token"),
            config,
            api,
        }
    }

    fn get_access_token(&self) -> Result<String> {
        let threshold = SystemTime::now() + Duration::from_secs(8);
        let should_refresh = match fs::metadata(&self.access_token_path) {
            Err(_) => true,
            Ok(metadata) => metadata.modified()? < threshold,
        };

        if !should_refresh {
            return Ok(fs::read_to_string(&self.access_token_path)?);
        }
        debug!("access_token needs to be refreshed");

        // refresh token using the API
        let token_response = self.api.refresh_access_token(
            &self.config.client_id,
            &self.config.client_secret,
            &self.config.refresh_token,
        )?;

        anyhow::ensure!(
            token_response.refresh_token == self.config.refresh_token,
            "refresh token changed and I was not implemented to handle that :("
        );

        let mut file = fs::File::create(&self.access_token_path)?;
        file.write_all(token_response.access_token.as_bytes())?;
        file.set_permissions(Permissions::from_mode(0o600))?;
        file.set_modified(token_response.expires_at)?;
        drop(file);
        chown(&self.access_token_path, Some(0), Some(0))?;

        Ok(token_response.refresh_token)
    }

    fn process_club(&self, club_id: &str) -> Result<()> {
        info!("processing club {}", club_id);

        let access_token = self.get_access_token()?;
        let events = self.api.get_group_events(club_id, &access_token)?;

        // save it as club_id.json
        let json_string = serde_json::to_string_pretty(&events)?;
        fs::write(&format!("{}-events.json", club_id), &json_string)?;

        Ok(())
    }
}

fn main() {
    // basic command line parsing
    let matches = command!()
        .arg(
            arg!(
                -q --quiet "Silence all output"
            )
            .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(
                -v --verbose "Increase log verbosity"
            )
            .required(false)
            .action(ArgAction::Count),
        )
        .get_matches();

    // configure logger
    stderrlog::new()
        .module(module_path!())
        .quiet(matches.get_flag("quiet"))
        .verbosity(matches.get_count("verbose") as usize)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .unwrap();
    debug!("log operational");

    // load config
    let config = Config::new().unwrap_or_else(|e| {
        error!("Cannot load config: {:#}", e);
        exit(1);
    });
    config.check_cache_permission().unwrap_or_else(|e| {
        error!("Invalid cache path: {:#}", e);
        exit(1);
    });
    debug!("config loaded");

    let api = StravaApi::new();
    let app = App::new(config, api);
    debug!("app ready");

    // fixme: do that in // but we probably don't need it since we will later have a SPA page
    let mut code = 0;
    for club in &app.config.clubs {
        app.process_club(club).unwrap_or_else(|e| {
            warn!("Could not process club {}: {:#}", club, e);
            code = 2;
        });
    }
    exit(code);
}
