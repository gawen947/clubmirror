use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Duration, TimeZone, Utc, Weekday};
use chrono_tz::Europe::Brussels;
use users::{
    get_group_by_gid, get_group_by_name, get_user_by_name, get_user_by_uid, gid_t, uid_t, Group,
    User,
};

/// Try to get the user by name or try to get it by uid instead.
pub fn get_user_by_name_or_uid(user: &str) -> Result<User> {
    if let Some(user) = get_user_by_name(user) {
        return Ok(user);
    }

    let uid = user.parse::<uid_t>()?;
    let user = get_user_by_uid(uid).ok_or_else(|| anyhow!("No such user {}", user))?;

    Ok(user)
}

/// Try to get the group by name or try to get it by gid instead.
pub fn get_group_by_name_or_gid(group: &str) -> Result<Group> {
    if let Some(group) = get_group_by_name(group) {
        return Ok(group);
    }

    let gid = group.parse::<gid_t>()?;
    let group = get_group_by_gid(gid).ok_or_else(|| anyhow!("No such group {}", group))?;

    Ok(group)
}

/// Get the next tuesday at 6AM Brussels timezone from the given date
pub fn get_next_tuesday_6am_brussels(date: &DateTime<Utc>) -> DateTime<Utc> {
    let date_at_bxl = date.with_timezone(&Brussels);

    // handle the fact that if we are after 6AM on tuesday, we should be considered as tomorrow
    let mut weekday = date_at_bxl.weekday().num_days_from_monday();
    let mut day_offset = 0;
    let date_at_bxl_6am = Brussels
        .with_ymd_and_hms(
            date_at_bxl.year(),
            date_at_bxl.month(),
            date_at_bxl.day(),
            6,
            0,
            0,
        )
        .unwrap();
    if date_at_bxl.weekday() == Weekday::Tue && date_at_bxl > date_at_bxl_6am {
        weekday += 1;
        day_offset = 1;
    }

    let days_until_tuesday = (8 - weekday) % 7 + day_offset;
    let next_tuesday = date_at_bxl.date_naive() + Duration::days(days_until_tuesday as i64);
    let next_tuesday_6am = Brussels
        .with_ymd_and_hms(
            next_tuesday.year(),
            next_tuesday.month(),
            next_tuesday.day(),
            6,
            0,
            0,
        )
        .unwrap();

    next_tuesday_6am.with_timezone(&Utc)
}

#[cfg(test)]
mod tests {
    use super::*; // Assuming the function is in the parent module
    use chrono::{DateTime, Timelike, Utc, Weekday};
    use chrono_tz::Europe::Brussels;

    fn brussels_to_utc(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> DateTime<Utc> {
        Brussels
            .with_ymd_and_hms(year, month, day, hour, min, sec)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn test_from_monday() {
        // Monday 10:00 AM Brussels -> next Tuesday 6:00 AM
        let monday = brussels_to_utc(2024, 1, 8, 10, 0, 0); // Jan 8, 2024 is a Monday
        let result = get_next_tuesday_6am_brussels(&monday);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be tomorrow (Jan 9)
        assert_eq!(result_brussels.day(), 9);
    }

    #[test]
    fn test_from_tuesday() {
        // Tuesday 10:00 AM Brussels -> next Tuesday 6:00 AM (next week)
        let tuesday = brussels_to_utc(2024, 1, 9, 10, 0, 0); // Jan 9, 2024 is a Tuesday
        let result = get_next_tuesday_6am_brussels(&tuesday);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be next week (Jan 16)
        assert_eq!(result_brussels.day(), 16);
    }

    #[test]
    fn test_from_wednesday() {
        // Wednesday 10:00 AM Brussels -> next Tuesday 6:00 AM
        let wednesday = brussels_to_utc(2024, 1, 10, 10, 0, 0); // Jan 10, 2024 is a Wednesday
        let result = get_next_tuesday_6am_brussels(&wednesday);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be next Tuesday (Jan 16)
        assert_eq!(result_brussels.day(), 16);
    }

    #[test]
    fn test_from_thursday() {
        // Thursday 10:00 AM Brussels -> next Tuesday 6:00 AM
        let thursday = brussels_to_utc(2024, 1, 11, 10, 0, 0); // Jan 11, 2024 is a Thursday
        let result = get_next_tuesday_6am_brussels(&thursday);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be next Tuesday (Jan 16)
        assert_eq!(result_brussels.day(), 16);
    }

    #[test]
    fn test_from_friday() {
        // Friday 10:00 AM Brussels -> next Tuesday 6:00 AM
        let friday = brussels_to_utc(2024, 1, 12, 10, 0, 0); // Jan 12, 2024 is a Friday
        let result = get_next_tuesday_6am_brussels(&friday);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be next Tuesday (Jan 16)
        assert_eq!(result_brussels.day(), 16);
    }

    #[test]
    fn test_from_saturday() {
        // Saturday 10:00 AM Brussels -> next Tuesday 6:00 AM
        let saturday = brussels_to_utc(2024, 1, 13, 10, 0, 0); // Jan 13, 2024 is a Saturday
        let result = get_next_tuesday_6am_brussels(&saturday);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be next Tuesday (Jan 16)
        assert_eq!(result_brussels.day(), 16);
    }

    #[test]
    fn test_from_sunday() {
        // Sunday 10:00 AM Brussels -> next Tuesday 6:00 AM
        let sunday = brussels_to_utc(2024, 1, 14, 10, 0, 0); // Jan 14, 2024 is a Sunday
        let result = get_next_tuesday_6am_brussels(&sunday);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be next Tuesday (Jan 16)
        assert_eq!(result_brussels.day(), 16);
    }

    #[test]
    fn test_tuesday_5_59_am() {
        // Tuesday 5:59 AM Brussels -> today at 6:00 AM (same Tuesday)
        let tuesday_559 = brussels_to_utc(2024, 1, 9, 5, 59, 0); // Jan 9, 2024 is a Tuesday
        let result = get_next_tuesday_6am_brussels(&tuesday_559);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be today (Jan 9)
        assert_eq!(result_brussels.day(), 9);
        assert_eq!(result_brussels.month(), 1);
    }

    #[test]
    fn test_tuesday_6_01_am() {
        // Tuesday 6:01 AM Brussels -> next week Tuesday at 6:00 AM
        let tuesday_601 = brussels_to_utc(2024, 1, 9, 6, 1, 0); // Jan 9, 2024 is a Tuesday
        let result = get_next_tuesday_6am_brussels(&tuesday_601);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be next week (Jan 16)
        assert_eq!(result_brussels.day(), 16);
        assert_eq!(result_brussels.month(), 1);
    }

    #[test]
    fn test_tuesday_6_00_am_exact() {
        // Tuesday 6:00 AM Brussels -> next week Tuesday at 6:00 AM
        let tuesday_600 = brussels_to_utc(2024, 1, 9, 6, 0, 0); // Jan 9, 2024 is a Tuesday
        let result = get_next_tuesday_6am_brussels(&tuesday_600);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be next week (Jan 9)
        assert_eq!(result_brussels.day(), 9);
        assert_eq!(result_brussels.month(), 1);
    }

    #[test]
    fn test_timezone_handling() {
        // Test during different seasons to ensure timezone conversion is correct

        // Winter time (CET = UTC+1)
        let winter_date = brussels_to_utc(2024, 1, 15, 10, 0, 0); // January
        let winter_result = get_next_tuesday_6am_brussels(&winter_date);
        let winter_brussels = winter_result.with_timezone(&Brussels);
        assert_eq!(winter_brussels.hour(), 6);

        // Summer time (CEST = UTC+2)
        let summer_date = brussels_to_utc(2024, 7, 15, 10, 0, 0); // July
        let summer_result = get_next_tuesday_6am_brussels(&summer_date);
        let summer_brussels = summer_result.with_timezone(&Brussels);
        assert_eq!(summer_brussels.hour(), 6);
    }

    #[test]
    fn test_month_boundary() {
        // Test when next Tuesday crosses month boundary
        let end_of_month = brussels_to_utc(2024, 1, 30, 10, 0, 0); // Jan 30, 2024 is a Tuesday
        let result = get_next_tuesday_6am_brussels(&end_of_month);
        let result_brussels = result.with_timezone(&Brussels);

        assert_eq!(result_brussels.weekday(), Weekday::Tue);
        assert_eq!(result_brussels.hour(), 6);
        assert_eq!(result_brussels.minute(), 0);
        assert_eq!(result_brussels.second(), 0);
        // Should be Feb 6, 2024
        assert_eq!(result_brussels.day(), 6);
        assert_eq!(result_brussels.month(), 2);
    }
}
