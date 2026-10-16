//! TEMPORARY: hardcoded quirks for things the Strava API (or a club) gets wrong.
//! This static approach is a placeholder; it will be replaced by something configurable.

use log::*;

use crate::strava::Event;

/// Sort the events based on the min of their upcoming_occurrences.
pub fn quirk_sort_events(mut events: Vec<Event>) -> Vec<Event> {
    debug!("applying quirk_sort_events");
    events.sort_by_key(|event| event.upcoming_occurrences.iter().min().cloned());
    events
}
