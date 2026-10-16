mod calendar;
mod config;
mod quirk;
mod routes;
mod strava;
mod template;
mod utils;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use clap::{Arg, ArgAction};
use log::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::exit, time::SystemTime};
use tera::Context;

use crate::calendar::save_calendar;
use crate::config::{default_config_path, ClubConfig, Config};
use crate::quirk::quirk_sort_events;
use crate::routes::RoutesStore;
use crate::strava::{Event, StravaApi, TraceType};
use crate::template::render_site;
use crate::utils::{get_group_by_name_or_gid, get_user_by_name_or_uid, write_private, Perms};

/// Rotated refresh token, kept in the cache directory. `origin` is the refresh token of the
/// config file it was derived from: if the config changes, the cached token is stale.
#[derive(Serialize, Deserialize)]
struct StoredRefreshToken {
    origin: String,
    current: String,
}

struct App {
    config: Config,
    api: StravaApi,
    routes: RoutesStore,
    perms: Perms,
    access_token_path: PathBuf,
    refresh_token_path: PathBuf,
}

impl App {
    fn new(config: Config, api: StravaApi) -> Result<Self> {
        let uid = match &config.generated.user {
            Some(user) => Some(get_user_by_name_or_uid(user)?.uid()),
            None => None,
        };
        let gid = match &config.generated.group {
            Some(group) => Some(get_group_by_name_or_gid(group)?.gid()),
            None => None,
        };
        let perms = Perms {
            mode: config.generated.permissions,
            uid,
            gid,
        };

        let routes = RoutesStore {
            dir: config.generated.routes_path.clone(),
            ttl: config.generated.routes_ttl,
            perms,
        };

        Ok(Self {
            access_token_path: config.cache_path.join("access_token"),
            refresh_token_path: config.cache_path.join("refresh_token.json"),
            config,
            api,
            routes,
            perms,
        })
    }

    /// The refresh token to use: the rotated one if it derives from the configured one.
    fn current_refresh_token(&self) -> String {
        let configured = &self.config.api.refresh_token;
        fs::read_to_string(&self.refresh_token_path)
            .ok()
            .and_then(|s| serde_json::from_str::<StoredRefreshToken>(&s).ok())
            .filter(|stored| &stored.origin == configured)
            .map(|stored| stored.current)
            .unwrap_or_else(|| configured.clone())
    }

    fn get_access_token(&self) -> Result<String> {
        let threshold = SystemTime::now() + std::time::Duration::from_secs(8);
        let should_refresh = match fs::metadata(&self.access_token_path) {
            Err(_) => true,
            Ok(metadata) => metadata.modified()? < threshold,
        };

        if !should_refresh {
            return Ok(fs::read_to_string(&self.access_token_path)?);
        }
        debug!("access_token needs to be refreshed");

        let refresh_token = self.current_refresh_token();
        let token_response = self.api.refresh_access_token(
            self.config.api.client_id,
            &self.config.api.client_secret,
            &refresh_token,
        )?;

        // persist the new refresh token first: the old one may already be invalid
        if token_response.refresh_token != refresh_token {
            info!("refresh token rotated, saving the new one");
            let stored = StoredRefreshToken {
                origin: self.config.api.refresh_token.clone(),
                current: token_response.refresh_token.clone(),
            };
            write_private(
                &self.refresh_token_path,
                &serde_json::to_vec(&stored)?,
                None,
            )?;
        }

        write_private(
            &self.access_token_path,
            token_response.access_token.as_bytes(),
            Some(token_response.expires_at),
        )?;

        Ok(token_response.access_token)
    }

    fn process_club(&self, club: &ClubConfig) -> Result<()> {
        info!("processing club {}", club.id);

        let access_token = self.get_access_token()?;

        let events = if let Some(local_json_path) = &self.config.api.quirk_local_json {
            self.api.get_group_events_from_file(local_json_path)?
        } else {
            self.api.get_group_events(club.id, &access_token)?
        };

        let timezone = club.timezone_or(self.config.timezone);
        let club_name = club
            .name
            .clone()
            .or_else(|| events.first().map(|event| event.club.name.clone()))
            .unwrap_or_else(|| format!("Club {}", club.id));

        // save the calendar if needed
        let generated = &self.config.generated;
        let calendar_file = format!("{}.ics", club.id);
        if let Some(calendars_path) = &generated.calendars_path {
            save_calendar(
                &events,
                &club_name,
                timezone,
                &calendars_path.join(&calendar_file),
                &self.perms,
            )?;
        }

        // filter events based on time
        let now = Utc::now();
        let mut events: Vec<Event> = events
            .into_iter()
            .filter(|event| {
                in_window(
                    &now,
                    event.upcoming_occurrences.iter().min().copied(),
                    event.upcoming_occurrences.iter().max().copied(),
                    event.created_at,
                    club.past_days,
                    club.future_days,
                )
            })
            .collect();

        if club.sort_events {
            events = quirk_sort_events(events);
        }

        // save events JSON if needed
        if let Some(json_destination_path) = &generated.quirk_save_json {
            self.perms.write(
                &json_destination_path.join(format!("{}.json", club.id)),
                serde_json::to_string_pretty(&events)?,
            )?;
        }

        // download routes if needed
        for event in events.iter() {
            if let Some(route_id) = event.route_id {
                self.routes
                    .get_route(&self.api, route_id, TraceType::Tcx, &access_token)?;
                self.routes
                    .get_route(&self.api, route_id, TraceType::Gpx, &access_token)?;
            }
        }

        // generate the site
        let mut context = Context::new();
        context.insert("club_id", &club.id);
        context.insert("club_name", &club_name);
        context.insert("timezone", timezone.name());
        context.insert("events", &events);
        context.insert("total_events", &events.len());
        context.insert(
            "generated_at",
            &now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        );
        if let Some(url) = &generated.routes_url {
            context.insert("routes_url", url);
        }
        if let Some(url) = &generated.calendars_url {
            context.insert("calendars_url", url);
            context.insert(
                "calendar_url",
                &format!("{}/{}", url.trim_end_matches('/'), calendar_file),
            );
        }

        render_site(
            &club.template_dir,
            &club.output_dir,
            &context,
            &generated.template_extensions,
            &self.perms,
        )
    }
}

/// Whether an event is inside the club's past/future window.
/// Past: the last occurrence (or the creation date if none) must be newer than `past_days`.
/// Future: the first occurrence must be less than `future_days` ahead.
fn in_window(
    now: &DateTime<Utc>,
    first: Option<DateTime<Utc>>,
    last: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    past_days: Option<u32>,
    future_days: Option<u32>,
) -> bool {
    if let Some(days) = past_days {
        let limit = *now - Duration::days(days as i64);
        if let Some(reference) = last.or(created_at) {
            if reference <= limit {
                return false;
            }
        }
    }

    if let Some(days) = future_days {
        let limit = *now + Duration::days(days as i64);
        if let Some(first) = first {
            if first >= limit {
                return false;
            }
        }
    }

    true
}

fn main() {
    let matches = clap::command!()
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("PATH")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Path to the configuration file [default: system config dir]"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Silence all output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Increase log verbosity")
                .action(ArgAction::Count),
        )
        .get_matches();

    stderrlog::new()
        .module(module_path!())
        .quiet(matches.get_flag("quiet"))
        .verbosity(matches.get_count("verbose") as usize)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .unwrap();
    debug!("log operational");

    let config_path = matches
        .get_one::<PathBuf>("config")
        .cloned()
        .unwrap_or_else(default_config_path);
    let config = Config::load(&config_path).unwrap_or_else(|e| {
        error!("Cannot load config: {:#}", e);
        exit(1);
    });
    config.prepare_cache().unwrap_or_else(|e| {
        error!("Invalid cache path: {:#}", e);
        exit(1);
    });
    debug!("config loaded from {}", config_path.display());

    let api = StravaApi::new(
        config.api.warn_on_404,
        config.api.log_response_path.clone(),
    );
    let app = App::new(config, api).unwrap_or_else(|e| {
        error!("cannot initialize app: {:#}", e);
        exit(1);
    });
    debug!("app ready");

    let mut code = 0;
    for club in &app.config.clubs {
        app.process_club(club).unwrap_or_else(|e| {
            warn!("could not process club {}: {:#}", club.id, e);
            code = 2;
        });
    }
    exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, day, 12, 0, 0).unwrap()
    }

    #[test]
    fn window_past_and_future_are_both_applied() {
        let now = at(15);
        // past window only
        assert!(!in_window(&now, Some(at(1)), Some(at(2)), None, Some(7), None));
        assert!(in_window(&now, Some(at(10)), Some(at(10)), None, Some(7), None));
        // future window only
        assert!(!in_window(&now, Some(at(25)), Some(at(25)), None, None, Some(7)));
        // both: the future limit must still apply when a past limit is set (old bug)
        assert!(!in_window(&now, Some(at(25)), Some(at(25)), None, Some(7), Some(7)));
        assert!(in_window(&now, Some(at(16)), Some(at(16)), None, Some(7), Some(7)));
    }

    #[test]
    fn window_without_occurrences_uses_creation_date() {
        let now = at(15);
        assert!(!in_window(&now, None, None, Some(at(1)), Some(7), None));
        assert!(in_window(&now, None, None, Some(at(14)), Some(7), None));
        assert!(in_window(&now, None, None, None, Some(7), Some(7)));
    }
}
