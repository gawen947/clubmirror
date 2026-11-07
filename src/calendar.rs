use anyhow::Result;
use chrono::Duration;
use chrono_tz::Tz;
use icalendar::{Calendar, Component, EventLike};
use log::*;
use std::path::Path;

use crate::{strava::Event, utils::Perms};

/// Write an ICS calendar with one entry per occurrence of each event.
pub fn save_calendar(
    events: &[Event],
    name: &str,
    timezone: Tz,
    path: &Path,
    perms: &Perms,
) -> Result<()> {
    debug!("generating calendar {}", path.display());
    let mut calendar = Calendar::new();
    calendar.name(name); // X-WR-CALNAME
    calendar.timezone(timezone.name()); // X-WR-TIMEZONE

    for event in events {
        let url = event.url();
        for occurrence in &event.upcoming_occurrences {
            let ics_event = icalendar::Event::new()
                // stable across runs, so calendar clients don't duplicate entries
                .uid(&format!(
                    "{}-{}@clubmirror",
                    event.id,
                    occurrence.timestamp()
                ))
                .summary(&event.title)
                .description(&event.description)
                .location(&event.address)
                .url(&url)
                .starts(*occurrence)
                .ends(*occurrence + Duration::hours(2)) // events are assumed to be 2h long
                .done();
            calendar.push(ics_event);
        }
    }

    perms.write(path, calendar.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_has_name_stable_uid_and_url() {
        // nullable athlete fields must not break parsing
        let event: Event = serde_json::from_value(serde_json::json!({
            "id": 7, "resource_state": 2, "title": "Sunday ride", "description": "d",
            "club_id": 42, "club": {"id": 42, "resource_state": 2, "name": "My Club"},
            "organizing_athlete": {"id": 1, "resource_state": 2, "firstname": "A", "lastname": "B",
                "bio": null, "city": null, "state": null, "sex": null, "created_at": null},
            "activity_type": "Ride", "created_at": "", "route_id": null, "route": null,
            "women_only": false, "private": false, "skill_levels": null, "terrain": null,
            "upcoming_occurrences": ["2026-06-14T08:00:00Z"], "zone": "Z", "address": "Somewhere",
            "joined": false, "start_latlng": null
        }))
        .unwrap();

        let path = std::env::temp_dir().join(format!("clubmirror-{}.ics", std::process::id()));
        let perms = Perms {
            mode: 0o644,
            uid: None,
            gid: None,
        };
        save_calendar(&[event], "My Club", Tz::Europe__Brussels, &path, &perms).unwrap();

        let ics = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert!(ics.contains("X-WR-CALNAME:My Club"));
        assert!(ics.contains("X-WR-TIMEZONE:Europe/Brussels"));
        assert!(ics.contains("UID:7-1781424000@clubmirror"));
        assert!(ics.contains("URL:https://www.strava.com/clubs/42/group_events/7"));
    }
}
