pub mod calendar;
pub mod quirk;
pub mod routes;
pub mod strava;
pub mod utils;
use chrono::{DateTime, Utc};
pub use strava::*;
pub mod template;
pub use template::*;
pub use utils::*;

use anyhow::Result;
use clap::{arg, command, ArgAction};
use log::*;
use serde::{Deserialize, Serialize};
use std::fs::{self, Permissions};
use std::io::Write;
use std::os::unix::fs::{chown, MetadataExt, PermissionsExt};
use std::path::PathBuf;
use std::process::exit;
use std::rc::Rc;
use std::time::SystemTime;
use toml;

use crate::calendar::save_calendar;
use crate::quirk::{quirk_bkn_early_ride, quirk_sort_events};
use crate::routes::RoutesStore;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    client_id: u64,
    client_secret: String,
    refresh_token: String,
    warn_on_404: Option<bool>, // warn on 404 error from the API instead of failing

    cache_path: PathBuf,
    template_path: PathBuf,
    generated_site_path: PathBuf,
    log_response_path: Option<PathBuf>, // save strava response in the following path for debugging
    downloaded_routes: Option<PathBuf>, // try to download the TCX/GPX from strava
    calendars_path: Option<PathBuf>,    // create an ICS calendar from all events
    routes_url: String,                 // provided to the templates to download the routes TCX/GPX
    calendars_url: Option<String>,      // provided to the templates to download the calendars

    clubs: Vec<u64>, // IDs of the clubs

    routes_ttl: u64, // expiry time in seconds for route files (TCX/GPX)

    generated_permission: u32,
    generated_user: Option<String>,
    generated_group: Option<String>,
    generated_past_days: Option<u32>, // show events up to x days in the past
    generated_future_days: Option<u32>, // show events up to x days in the future

    quirk_save_json: Option<PathBuf>, // save the resulting JSON for each clubs in the given path
    quirk_local_json: Option<PathBuf>, // use the given JSON instead of asking the API (avoid spamming it)
    quirk_bkn_early_ride: Option<bool>, // report the latest early ride of the JSON to next tuesday
    quirk_bkn_early_ride_start: Option<DateTime<Utc>>, // no early ride quirk before this date
    quirk_bkn_early_ride_end: Option<DateTime<Utc>>, // no early ride quirk after this date
    quirk_sort_events: Option<bool>,   // sort the events after filter (useful if rescheduled)
}

impl Config {
    fn new() -> Result<Self> {
        let mut config_path: PathBuf = match std::env::consts::OS {
            "freebsd" => "/usr/local/etc".into(),
            _ => "/etc".into(),
        };
        config_path.push("lbs-mirror");
        config_path.push("config.toml");

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
            metadata.mode() & 0o777 == 0o700,
            "{} is not permission 700",
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
    config: Rc<Config>,
    access_token_path: PathBuf,
    routes: RoutesStore,
    api: Rc<StravaApi>,

    generated_uid: Option<u32>,
    generated_gid: Option<u32>,
}

impl App {
    fn new(config: Config, api: StravaApi) -> Result<Self> {
        let generated_uid = if let Some(user) = &config.generated_user {
            Some(get_user_by_name_or_uid(user)?.uid())
        } else {
            None
        };
        let generated_gid = if let Some(group) = &config.generated_group {
            Some(get_group_by_name_or_gid(group)?.gid())
        } else {
            None
        };

        let config = Rc::new(config);
        let api = Rc::new(api);

        let routes = RoutesStore {
            config: Rc::clone(&config),
            api: Rc::clone(&api),
            ttl: config.routes_ttl,
            permissions: config.generated_permission,
            uid: generated_uid,
            gid: generated_gid,
        };

        Ok(Self {
            access_token_path: config.cache_path.clone().join("access_token"),
            config,
            api,
            routes,
            generated_uid,
            generated_gid,
        })
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

        // refresh token using the API
        let token_response = self.api.refresh_access_token(
            self.config.client_id,
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

        Ok(token_response.access_token)
    }

    /// filter predicate for events that are too far in the past or the future
    fn event_filter(&self, now: &DateTime<Utc>, event: &Event) -> bool {
        let created_at = event.created_at;
        let occurences = &event.upcoming_occurrences;

        let occurence_min = occurences.iter().min().copied();
        let occurence_max = occurences.iter().max().copied();

        if let Some(past_days) = self.config.generated_past_days {
            let past = now.clone() - chrono::Duration::days(past_days as i64);

            if let Some(occurence_max) = occurence_max {
                return occurence_max > past;
            } else if let Some(created_at) = created_at {
                return created_at > past;
            }
        }

        if let Some(future_days) = self.config.generated_future_days {
            let future = now.clone() + chrono::Duration::days(future_days as i64);

            if let Some(occurence_min) = occurence_min {
                return occurence_min < future;
            }
        }

        true
    }

    fn process_club(&self, club_id: u64) -> Result<()> {
        info!("processing club {}", club_id);

        let access_token = self.get_access_token()?;

        let mut events = if let Some(local_json_path) = &self.config.quirk_local_json {
            self.api.get_group_events_from_file(local_json_path)?
        } else {
            self.api.get_group_events(club_id, &access_token)?
        };

        // apply the BKN early ride quirk if needed
        if self.config.quirk_bkn_early_ride.unwrap_or(false) {
            events = quirk_bkn_early_ride(
                events,
                &self.config.quirk_bkn_early_ride_start,
                &self.config.quirk_bkn_early_ride_end,
            );
        }

        // save the calendar if needed
        if let Some(calendars_path) = &self.config.calendars_path {
            save_calendar(&events, &calendars_path.join(&format!("{}.ics", club_id)))?;
        }

        // filter events based on time
        let now = Utc::now();
        let mut events: Vec<Event> = events
            .into_iter()
            .filter(|event| self.event_filter(&now, event))
            .collect();

        // sort events if needed
        if self.config.quirk_sort_events.unwrap_or(false) {
            events = quirk_sort_events(events);
        }

        // save events JSON if needed
        if let Some(json_destination_path) = &self.config.quirk_save_json {
            let json_string = serde_json::to_string_pretty(&events)?;
            fs::write(
                json_destination_path.join(format!("{}.json", club_id)),
                &json_string,
            )?;
        }

        // download routes if needed
        for event in events.iter() {
            if let Some(route_id) = event.route_id {
                self.routes
                    .get_route(route_id, TraceType::Tcx, &access_token)?;
                self.routes
                    .get_route(route_id, TraceType::Gpx, &access_token)?;
            }
        }

        // generate the template
        let club_id_str = club_id.to_string();
        let template_path = self.config.template_path.join(&club_id_str);
        let generated_path = self.config.generated_site_path.join(&club_id_str);
        let template = Template::new(
            &template_path,
            &generated_path,
            &self.config.routes_url,
            self.config.calendars_url.as_deref(),
            &events,
            club_id,
            self.config.generated_permission,
            self.generated_uid,
            self.generated_gid,
        )?;
        template.generate()?;

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

    let api = StravaApi::new(config.warn_on_404.unwrap_or(false), config.log_response_path.clone());
    let app = App::new(config, api).unwrap_or_else(|e| {
        error!("cannot initialize app: {:#}", e);
        exit(1);
    });
    debug!("app ready");

    // fixme: do that in // but we probably don't need it since we will later have a SPA page
    let mut code = 0;
    for club in &app.config.clubs {
        app.process_club(*club).unwrap_or_else(|e| {
            warn!("could not process club {}: {:#}", club, e);
            code = 2;
        });
    }
    exit(code);
}
