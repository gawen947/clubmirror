use anyhow::Result;
use chrono::Duration;
use icalendar::{Calendar, Component, Event, EventLike};
use log::*;
use std::path::Path;

pub fn save_calendar(events: &Vec<crate::Event>, path: &Path) -> Result<()> {
    debug!("generating calendar {}", path.display());
    let mut calendar = Calendar::new();
    // todo: give the calendar a name

    for event in events {
        for occurence in &event.upcoming_occurrences {
            let ics_event = Event::new()
                .summary(&event.title)
                .description(&event.description)
                .location(&event.address)
                .starts(*occurence)
                .ends(*occurence + Duration::hours(2)) // events are assumed to be 2h long
                .done();
            calendar.push(ics_event);
        }
    }

    std::fs::write(path, calendar.to_string())?;

    Ok(())
}
