use chrono::{DateTime, Utc};
use log::*;

use crate::{get_next_tuesday_6am_brussels, Event};

/// Map the latest EARLY RIDE to next tuesday 6AM
pub fn quirk_bkn_early_ride(
    mut events: Vec<Event>,
    start: &Option<DateTime<Utc>>,
    end: &Option<DateTime<Utc>>,
) -> Vec<Event> {
    // find latest EARLY RIDE
    let latest_early_ride = events
        .iter()
        .filter(|event| event.title.contains("EARLY RIDE"))
        .max_by_key(|event| {
            event
                .upcoming_occurrences
                .iter()
                .max()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
        });

    // reschedule a new early ride next monday
    if let Some(event) = latest_early_ride {
        debug!("applying quirk_bkn_early_ride");
        let next_tuesday = get_next_tuesday_6am_brussels(&Utc::now());

        // skip if outside boundaries
        let mut skip: bool = false;
        if let Some(start) = start {
            skip = skip || next_tuesday < *start;
        }
        if let Some(end) = end {
            skip = skip || next_tuesday > *end;
        }

        if !skip {
            let mut new_early_ride = event.clone();
            new_early_ride.upcoming_occurrences = vec![next_tuesday];
            events.push(new_early_ride);
        }
    }

    events
}

/**
Sort the events based on the min of the the upcoming_occurrences.
Not useful most of the time because the API answer with a shorted list.
But in the case of other quirks like quirk_bkn_early_ride() where we reschedule
the events, we have to sort the events again before giving them to the templates.
*/
pub fn quirk_sort_events(mut events: Vec<Event>) -> Vec<Event> {
    debug!("applying quirk_sort_events");
    events.sort_by_key(|event| event.upcoming_occurrences.iter().min().cloned());
    events
}
