use anyhow::Result;
use std::{
    fs::Permissions,
    io::Write,
    os::unix::fs::{chown, PermissionsExt},
    rc::Rc,
    time::{Duration, SystemTime},
};

use log::*;

use crate::{Config, StravaApi, TraceType};

pub struct RoutesStore {
    pub config: Rc<Config>,
    pub ttl: u64,
    pub api: Rc<StravaApi>,
    pub permissions: u32,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
}

impl RoutesStore {
    /// Get a route from cache if not expired, otherwise download it.
    pub fn get_route(
        &self,
        route_id: u64,
        trace_type: TraceType,
        access_token: &str,
    ) -> Result<()> {
        let now = SystemTime::now();

        let extension = match trace_type {
            TraceType::Tcx => "tcx",
            TraceType::Gpx => "gpx",
        };

        let path = self
            .config
            .downloaded_routes
            .join(format!("{}.{}", route_id, extension));

        let is_expired = if !path.exists() {
            true
        } else {
            let mtime = std::fs::metadata(&path)?.modified()?;
            mtime < now
        };

        if is_expired {
            debug!("downloading route {}", path.display());
            let response = self.api.get_route(route_id, trace_type, access_token)?;

            let mut file = std::fs::File::create(&path)?;
            file.write_all(&response.bytes()?)?;
            file.set_permissions(Permissions::from_mode(self.permissions))?;
            file.set_modified(now + Duration::from_secs(self.ttl))?;
            drop(file);
            chown(&path, self.uid, self.gid)?;
        }

        Ok(())
    }
}
