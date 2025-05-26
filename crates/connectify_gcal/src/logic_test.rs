#[cfg(test)]
mod tests {
    use crate::logic::calculate_available_slots;
    use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeZone, Utc, Weekday};
    use tracing::info;
    #[test]
    fn test_calculate_available_slots_no_busy_periods() {
        // Test case: No busy periods, should return slots at regular intervals
        // Use a fixed date (Monday) for deterministic testing
        let query_start = Utc.with_ymd_and_hms(2025, 5, 5, 0, 0, 0).unwrap(); // Monday
        let query_end = query_start + Duration::days(1);
        let busy_periods: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();
        let duration = Duration::minutes(60);
        let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        let working_days = [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ];
        let buffer = Duration::minutes(0);
        let step = Duration::minutes(15);

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

        // Check if the current day is a working day
        let is_working_day = working_days.contains(&query_start.weekday());

        if is_working_day {
            // If it's a working day, we should have slots
            assert!(
                !slots.is_empty(),
                "Should have available slots on a working day"
            );

            // Check that slots are within working hours
            info!("Slots: {:?}", slots);
            for (start_time_str, _) in &slots {
                // Parse the start time string to a DateTime
                let start_time_dt = DateTime::parse_from_rfc3339(start_time_str)
                    .expect("Failed to parse RFC3339 time");

                // Get the time component
                let slot_time = start_time_dt.time();

                info!(
                    "Slot time: {:?}, work_start: {:?}, work_end: {:?}",
                    slot_time, work_start, work_end
                );
                assert!(
                    slot_time >= work_start && slot_time <= work_end,
                    "Slot should be within working hours: {:?}",
                    start_time_str
                );
            }

            // Check that slots are properly spaced
            for i in 1..slots.len() {
                let (prev_start_str, _) = &slots[i - 1];
                let (curr_start_str, _) = &slots[i];

                // Parse the start time strings to DateTime objects
                let prev_start = DateTime::parse_from_rfc3339(prev_start_str)
                    .expect("Failed to parse RFC3339 time");
                let curr_start = DateTime::parse_from_rfc3339(curr_start_str)
                    .expect("Failed to parse RFC3339 time");

                let time_diff = curr_start.signed_duration_since(prev_start);
                assert!(
                    time_diff >= duration,
                    "Slots should be at least duration apart: {:?} and {:?}",
                    prev_start_str,
                    curr_start_str
                );
            }
        } else {
            // If it's not a working day, we should have no slots
            assert!(
                slots.is_empty(),
                "Should have no available slots on a non-working day"
            );
        }
    }

    #[test]
    fn test_calculate_available_slots_with_busy_periods() {
        // Test case: Some busy periods, should return slots that don't overlap
        // Use a fixed date (Monday) for deterministic testing
        let query_start = Utc.with_ymd_and_hms(2025, 5, 5, 0, 0, 0).unwrap(); // Monday
        let query_end = query_start + Duration::days(1);

        // Create a busy period in the middle of the day
        let busy_start = query_start + Duration::hours(12);
        let busy_end = busy_start + Duration::hours(2);
        let busy_periods = vec![(busy_start, busy_end)];

        let duration = Duration::minutes(60);
        let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        let working_days = [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ];
        let buffer = Duration::minutes(0);
        let step = Duration::minutes(15);

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

        // Check if the current day is a working day
        let is_working_day = working_days.contains(&query_start.weekday());

        if is_working_day {
            // If it's a working day, we should have slots
            assert!(
                !slots.is_empty(),
                "Should have available slots on a working day"
            );

            // Check that no slot overlaps with the busy period
            for (start_str, end_str) in &slots {
                // Parse the start and end time strings to DateTime objects
                let slot_start =
                    DateTime::parse_from_rfc3339(start_str).expect("Failed to parse RFC3339 time");
                let slot_end =
                    DateTime::parse_from_rfc3339(end_str).expect("Failed to parse RFC3339 time");

                // Simplify the check: ensure slots don't overlap with busy period
                assert!(
                    slot_end.with_timezone(&Utc) <= busy_start
                        || slot_start.with_timezone(&Utc) >= busy_end,
                    "Slot should not overlap with busy period: {:?} to {:?}",
                    start_str,
                    end_str
                );
            }
        } else {
            // If it's not a working day, we should have no slots
            assert!(
                slots.is_empty(),
                "Should have no available slots on a non-working day"
            );
        }
    }

    #[test]
    fn test_calculate_available_slots_with_buffer() {
        // Test case: With buffer time between appointments
        // Use a fixed date (Monday) for deterministic testing
        let query_start = Utc.with_ymd_and_hms(2025, 5, 5, 0, 0, 0).unwrap(); // Monday
        let query_end = query_start + Duration::days(1);
        let busy_periods: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();
        let duration = Duration::minutes(60);
        let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        let working_days = [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ];
        let buffer = Duration::minutes(15); // 15-minute buffer between appointments
        let step = Duration::minutes(15);

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

        // Check if the current day is a working day
        let is_working_day = working_days.contains(&query_start.weekday());

        if is_working_day {
            // If it's a working day, we should have slots
            assert!(
                !slots.is_empty(),
                "Should have available slots on a working day"
            );

            // Check that slots are properly spaced with buffer
            for i in 1..slots.len() {
                let (prev_start_str, _) = &slots[i - 1];
                let (curr_start_str, _) = &slots[i];

                // Parse the start time strings to DateTime objects
                let prev_start = DateTime::parse_from_rfc3339(prev_start_str)
                    .expect("Failed to parse RFC3339 time");
                let curr_start = DateTime::parse_from_rfc3339(curr_start_str)
                    .expect("Failed to parse RFC3339 time");

                let time_diff = curr_start.signed_duration_since(prev_start);
                assert!(
                    time_diff >= duration + buffer,
                    "Slots should be at least duration + buffer apart: {:?} and {:?}",
                    prev_start_str,
                    curr_start_str
                );
            }
        } else {
            // If it's not a working day, we should have no slots
            assert!(
                slots.is_empty(),
                "Should have no available slots on a non-working day"
            );
        }
    }

    #[test]
    fn test_calculate_available_slots_respects_working_hours() {
        // Test case: Ensure slots are only within working hours
        // Use a fixed date (Monday) for deterministic testing
        let query_start: chrono::DateTime<chrono::Utc> =
            Utc.with_ymd_and_hms(2025, 5, 5, 0, 0, 0).unwrap(); // Monday, May 5, 2025
        let query_end = query_start + Duration::days(1);
        let busy_periods: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();
        let duration = Duration::minutes(60);
        let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        let working_days = [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ];
        let buffer = Duration::minutes(0);
        let step = Duration::minutes(15);

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

        // Check if the current day is a working day
        let is_working_day = working_days.contains(&query_start.weekday());

        if is_working_day {
            // If it's a working day, we should have slots
            assert!(
                !slots.is_empty(),
                "Should have available slots on a working day"
            );

            // Check that all slots start within working hours
            for (start_str, end_str) in &slots {
                // Parse the start and end time strings to DateTime objects
                let slot_start =
                    DateTime::parse_from_rfc3339(start_str).expect("Failed to parse RFC3339 time");
                let slot_end =
                    DateTime::parse_from_rfc3339(end_str).expect("Failed to parse RFC3339 time");

                let slot_time = slot_start.time();
                let slot_end_time = slot_end.time();

                assert!(
                    slot_time >= work_start,
                    "Slot should start after work hours begin: {:?}",
                    start_str
                );
                assert!(
                    slot_end_time <= work_end,
                    "Slot should end before work hours end: {:?} + {:?} = {:?}",
                    start_str,
                    duration,
                    end_str
                );
            }
        } else {
            // If it's not a working day, we should have no slots
            assert!(
                slots.is_empty(),
                "Should have no available slots on a non-working day"
            );
        }
    }
}
