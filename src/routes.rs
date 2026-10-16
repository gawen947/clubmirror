use anyhow::Result;
use log::*;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use crate::{
    strava::{StravaApi, TraceType},
    utils::Perms,
};

/// Cache of the GPX/TCX routes. The mtime of a cached file is its expiry date.
pub struct RoutesStore {
    /// Download directory; routes are not downloaded if unset.
    pub dir: Option<PathBuf>,
    pub ttl: u64,
    pub perms: Perms,
}

impl RoutesStore {
    /// Get a route from cache if not expired, otherwise download it.
    pub fn get_route(
        &self,
        api: &StravaApi,
        route_id: u64,
        trace_type: TraceType,
        access_token: &str,
    ) -> Result<()> {
        let Some(dir) = &self.dir else {
            return Ok(());
        };
        let now = SystemTime::now();

        let extension = match trace_type {
            TraceType::Tcx => "tcx",
            TraceType::Gpx => "gpx",
        };
        let path = dir.join(format!("{}.{}", route_id, extension));

        let is_expired = match fs::metadata(&path) {
            Err(_) => true,
            Ok(metadata) => metadata.modified()? < now,
        };
        if !is_expired {
            return Ok(());
        }

        debug!("downloading route {}", path.display());
        let response = api.get_route(route_id, trace_type, access_token)?;
        if !response.status().is_success() {
            // only reachable with warn_on_404: never cache an error page as a route
            return Ok(());
        }

        self.perms.write(&path, response.bytes()?)?;
        fs::OpenOptions::new()
            .write(true)
            .open(&path)?
            .set_modified(now + Duration::from_secs(self.ttl))?;

        Ok(())
    }
}
