use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use log::*;
use reqwest::blocking::{Client, Response};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::Path;
use std::path::PathBuf;
use std::{
    collections::HashMap,
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

static TOKEN_URL: &str = "https://www.strava.com/oauth/token";
static API_URL: &str = "https://www.strava.com/api/v3";

pub struct StravaApi {
    client: Client,
    warn_on_404: bool,
    log_response_path: Option<PathBuf>,
}

impl StravaApi {
    pub fn new(warn_on_404: bool, log_response_path: Option<PathBuf>) -> Self {
        Self {
            client: Client::new(),
            warn_on_404,
            log_response_path
        }
    }

    pub fn refresh_access_token(
        &self,
        client_id: u64,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<TokenResponse> {
        let params = HashMap::from([
            ("client_id", client_id.to_string()),
            ("client_secret", client_secret.to_owned()),
            ("refresh_token", refresh_token.to_owned()),
            ("grant_type", "refresh_token".to_owned()),
        ]);

        let response = self
            .client
            .post(TOKEN_URL)
            .form(&params)
            .send()?
            .json::<TokenResponse>()?;

        Ok(response)
    }

    pub fn get(&self, path: &str, access_token: &str) -> Result<Response> {
        let url = format!("{}/{}", API_URL, path);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();

            if self.warn_on_404 && status_code == 404 {
                warn!("got a 404 for {}", &url);
            } else {
                return Err(anyhow!(
                    "API call failed: {} {}",
                    status_code,
                    response.status().canonical_reason().unwrap_or("(unknown)")
                ));
            }
        }

        Ok(response)
    }

    pub fn get_group_events(&self, club_id: u64, access_token: &str) -> Result<Vec<Event>> {
        // parse response as text because we might have to save is (and we cannot clone a response)
        let response = self.get(&format!("clubs/{}/group_events", club_id), access_token)?.text()?;

        if let Some(log_response_path) = &self.log_response_path {
            std::fs::write(&log_response_path, &response)?;
        }

        Ok(serde_json::from_str(&response)?)
    }

    /// Get the group events from a JSON file (only here for debug purposes)
    pub fn get_group_events_from_file(&self, path: &Path) -> Result<Vec<Event>> {
        let json = fs::read_to_string(path)?;
        let events = serde_json::from_str::<Vec<Event>>(&json)?;

        Ok(events)
    }

    /// Download a route
    pub fn get_route(
        &self,
        route_id: u64,
        trace_type: TraceType,
        access_token: &str,
    ) -> Result<Response> {
        let trace_type_url = match trace_type {
            TraceType::Tcx => "export_tcx",
            TraceType::Gpx => "export_gpx",
        };
        let response = self.get(
            &format!("routes/{}/{}", route_id, trace_type_url),
            access_token,
        )?;
        Ok(response)
    }
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp: u64 = Deserialize::deserialize(deserializer)?;
    Ok(UNIX_EPOCH + Duration::from_secs(timestamp))
}

fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(deserializer)?;

    match s {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => DateTime::parse_from_rfc3339(&s)
            .map(|dt| Some(dt.with_timezone(&Utc)))
            .map_err(D::Error::custom),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    // cf: https://developers.strava.com/docs/authentication
    pub access_token: String,
    pub refresh_token: String,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub expires_at: SystemTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    pub id: u64,
    pub resource_state: u8,
    pub title: String,
    pub description: String,
    pub club_id: u64,
    pub club: Club,
    pub organizing_athlete: Athlete,
    pub activity_type: String,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub created_at: Option<DateTime<Utc>>,
    pub route_id: Option<u64>,
    pub route: Option<serde_json::Value>, // null most of the time
    pub women_only: bool,
    pub private: bool,
    pub skill_levels: Option<u8>,
    pub terrain: Option<u8>,
    pub upcoming_occurrences: Vec<DateTime<Utc>>,
    pub zone: String,
    pub address: String,
    pub joined: bool,
    pub start_latlng: Option<LatLng>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LatLng {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Club {
    pub id: u64,
    pub resource_state: u8,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Athlete {
    pub id: u64,
    pub username: Option<String>,
    pub resource_state: u8,
    pub firstname: String,
    pub lastname: String,
    pub bio: String,
    pub city: String,
    pub state: String,
    pub country: Option<String>,
    pub sex: String,
    pub premium: bool,
    pub summit: bool,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub updated_at: Option<DateTime<Utc>>,
    pub badge_type_id: u8,
    pub profile_medium: String,
    pub profile: String,
    pub friend: Option<serde_json::Value>, // null most of the time
    pub follower: Option<serde_json::Value>, // null most of the time
    pub blocked: bool,
    pub can_follow: bool,
}

pub enum TraceType {
    Tcx,
    Gpx,
}
