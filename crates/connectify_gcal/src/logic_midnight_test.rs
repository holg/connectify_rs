#[cfg(test)]
mod tests {
    use crate::logic::calculate_available_slots;
    use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeZone, Timelike, Utc, Weekday};
    use chrono_tz::Tz;
    use std::str::FromStr;
    use tracing::debug;

    #[test]
    fn test_calculate_available_slots_with_midnight_slot() {
        // Test case: Ensure slots are available at 23:00-00:00 and first slot is 2h in the future
        let time_zone = Tz::from_str("Europe/Zurich").unwrap();

        // Use the current time in Zurich timezone
        let now = Utc::now().with_timezone(&time_zone);

        // Set prepare_time_minutes to 120 (2 hours) as required
        let prepare_time = Duration::minutes(120);

        // Query from now + prepare_time to the end of the next day
        let query_start = now + prepare_time;
        let query_end = time_zone
            .with_ymd_and_hms(now.year(), now.month(), now.day() + 1, 00, 00, 00)
            .unwrap();

        let busy_periods: Vec<(DateTime<Tz>, DateTime<Tz>)> = Vec::new();
        let duration = Duration::minutes(60);

        // Set working hours from 00:00 to 23:59 to cover the whole day
        let work_start = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let work_end = NaiveTime::from_hms_opt(23, 59, 0).unwrap();

        // Include all days of the week
        let working_days = [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ];

        // Set buffer and step
        let buffer = Duration::minutes(0);
        let step = Duration::minutes(60);

        debug!("Test inputs for test_calculate_available_slots_with_midnight_slot:");
        debug!("  now: {}", now);
        debug!("  query_start: {}", query_start);
        debug!("  query_end: {}", query_end);
        debug!("  busy_periods: {:?}", busy_periods);
        debug!("  duration: {} minutes", duration.num_minutes());
        debug!("  work_start: {}", work_start);
        debug!("  work_end: {}", work_end);
        debug!("  working_days: {:?}", working_days);
        debug!("  buffer: {} minutes", buffer.num_minutes());
        debug!("  prepare_time: {} minutes", prepare_time.num_minutes());
        debug!("  step: {} minutes", step.num_minutes());

        let slots = calculate_available_slots(
            query_start,
            query_end,
            &busy_periods,
            duration,
            work_start,
            work_end,
            &working_days,
            buffer,
            step,
        );

        debug!("Test outputs for test_calculate_available_slots_with_midnight_slot:");
        debug!("  slots: {:?}", slots);

        // Check if we have slots
        assert!(!slots.is_empty(), "Should have available slots");

        // Check that the first slot is at least 2 hours in the future
        if let Some((first_slot_start, _)) = slots.first() {
            let first_slot_time = DateTime::parse_from_rfc3339(first_slot_start)
                .expect("Failed to parse RFC3339 time");

            let time_diff = first_slot_time.signed_duration_since(now);
            assert!(
                time_diff >= prepare_time,
                "First slot should be at least {} minutes in the future, but was {} minutes",
                prepare_time.num_minutes(),
                time_diff.num_minutes()
            );
        }

        // Check that we have a 23:00-00:00 slot
        let mut has_midnight_slot = false;
        for (start_str, end_str) in &slots {
            let slot_start =
                DateTime::parse_from_rfc3339(start_str).expect("Failed to parse RFC3339 time");
            let slot_end =
                DateTime::parse_from_rfc3339(end_str).expect("Failed to parse RFC3339 time");

            // Check if this is a 23:00-00:00 slot
            if slot_start.hour() == 23
                && slot_start.minute() == 0
                && slot_end.hour() == 0
                && slot_end.minute() == 0
            {
                has_midnight_slot = true;
                break;
            }
        }

        assert!(
            has_midnight_slot,
            "Should have a 23:00-00:00 slot available"
        );
    }
}
