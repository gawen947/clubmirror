use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

static TOKEN_URL: &str = "https://www.strava.com/oauth/token";
static API_URL: &str = "https://www.strava.com/api/v3";

pub struct StravaApi {
    client: Client,
}

impl StravaApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn refresh_access_token(
        &self,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<TokenResponse> {
        let params = HashMap::from([
            ("client_id", client_id.to_owned()),
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
            .get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "API call failed: {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap()
            ));
        }

        Ok(response)
    }

    pub fn get_group_events(&self, club_id: &str, access_token: &str) -> Result<Vec<Event>> {
        let response = self.get(&format!("clubs/{}/group_events", club_id), access_token)?;

        Ok(response.json::<Vec<Event>>()?)
    }
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp: u64 = Deserialize::deserialize(deserializer)?;
    Ok(UNIX_EPOCH + Duration::from_secs(timestamp))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    // cf: https://developers.strava.com/docs/authentication
    pub access_token: String,
    pub refresh_token: String,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub expires_at: SystemTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub id: u64,
    pub resource_state: u8,
    pub title: String,
    pub description: String,
    pub club_id: u64,
    pub club: Club,
    pub organizing_athlete: Athlete,
    pub activity_type: String,
    pub created_at: DateTime<Utc>,
    pub route_id: Option<u64>,
    pub route: Option<serde_json::Value>, // null most of the time
    pub women_only: bool,
    pub private: bool,
    pub skill_levels: u8,
    pub terrain: u8,
    pub upcoming_occurrences: Vec<DateTime<Utc>>,
    pub zone: String,
    pub address: String,
    pub joined: bool,
    pub start_latlng: Option<LatLng>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LatLng {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Club {
    pub id: u64,
    pub resource_state: u8,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub badge_type_id: u8,
    pub profile_medium: String,
    pub profile: String,
    pub friend: Option<serde_json::Value>, // null most of the time
    pub follower: Option<serde_json::Value>, // null most of the time
    pub blocked: bool,
    pub can_follow: bool,
}
