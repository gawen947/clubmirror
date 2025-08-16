use chrono::{DateTime, Utc};

use crate::{get_next_tuesday_6am_brussels, Event};

pub fn quirk_bkn_early_ride(mut events: Vec<Event>) -> Vec<Event> {
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
        let next_tuesday = get_next_tuesday_6am_brussels(&Utc::now());

        let mut new_early_ride = event.clone();
        new_early_ride.upcoming_occurrences = vec![next_tuesday];
        events.push(new_early_ride);
    }

    events
}
