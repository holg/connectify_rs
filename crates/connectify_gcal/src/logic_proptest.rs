#[cfg(test)]
mod tests {
    use crate::logic::calculate_available_slots;
    use chrono::{DateTime, Duration, NaiveTime, Utc, Weekday};
    use chrono_tz::Tz;
    use proptest::prelude::*;

    // Helper function to create a valid time range
    fn create_time_range(
        start_offset_hours: i64,
        duration_days: i64,
    ) -> (DateTime<Tz>, DateTime<Tz>) {
        let time_zone = Tz::Europe__Zurich;
        let start = Utc::now().with_timezone(&time_zone) + Duration::hours(start_offset_hours);
        let end = start + Duration::days(duration_days);
        (start, end)
    }

    // Helper function to create a list of busy periods
    fn create_busy_periods(
        base_time: DateTime<Tz>,
        count: usize,
        max_duration_hours: i64,
    ) -> Vec<(DateTime<Tz>, DateTime<Tz>)> {
        let time_zone = Tz::Europe__Zurich;
        let mut busy_periods = Vec::new();
        let mut current_time = base_time.with_timezone(&time_zone);

        for _ in 0..count {
            let start = current_time + Duration::hours(1);
            let end = start + Duration::hours(max_duration_hours.max(1));
            busy_periods.push((start, end));
            current_time = end + Duration::hours(1);
        }

        busy_periods
    }

    // Helper function to parse RFC3339 string to DateTime<Utc>
    fn parse_datetime(datetime_str: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(datetime_str)
            .expect("Failed to parse RFC3339 datetime")
            .with_timezone(&Utc)
    }

    proptest! {
        // Test that slots are within working hours
        #[test]
        fn test_slots_within_working_hours(
            start_offset_hours in 0..24i64,
            duration_days in 1..7i64,
            appointment_duration_minutes in 15..120i64,
            work_start_hour in 0..12i64,
            work_end_hour in 13..23i64,
            busy_count in 0..5usize,
            max_busy_duration_hours in 1..4i64,
        ) {
            // Create a valid time range
            let (start, end) = create_time_range(start_offset_hours, duration_days);

            // Create work hours
            let work_start = NaiveTime::from_hms_opt(work_start_hour as u32, 0, 0).unwrap();
            let work_end = NaiveTime::from_hms_opt(work_end_hour as u32, 0, 0).unwrap();

            // Create busy periods
            let busy_periods = create_busy_periods(start, busy_count, max_busy_duration_hours);

            // Create appointment duration
            let appointment_duration = Duration::minutes(appointment_duration_minutes);

            // Define working days (all days for simplicity)
            let working_days = [
                Weekday::Mon, Weekday::Tue, Weekday::Wed,
                Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun
            ];

            // Calculate available slots
            let slots = calculate_available_slots(
                start,
                end,
                &busy_periods,
                appointment_duration,
                work_start,
                work_end,
                &working_days,
                Duration::minutes(0), // No buffer
                Duration::minutes(15), // 15-minute step
            );
            let time_zone = Tz::Europe__Zurich;
            // Check that all slots are within working hours
            for (start_str, end_str) in &slots {
                let slot = parse_datetime(start_str).with_timezone(&time_zone);
                let slot_end = parse_datetime(end_str).with_timezone(&time_zone);

                let slot_time = slot.time();
                let slot_end_time = slot_end.time();

                // The slot should start after or at work_start
                prop_assert!(slot_time >= work_start,
                    "Slot should start after work hours begin: {:?}, work start: {:?}",
                    slot_time, work_start);

                // The slot should end before or at work_end
                prop_assert!(slot_end_time <= work_end,
                    "Slot should end before work hours end: {:?}, work end: {:?}",
                    slot_end_time, work_end);
            }
        }

        // Test that slots don't overlap with busy periods
        #[test]
        fn test_slots_dont_overlap_busy_periods(
            start_offset_hours in 0..24i64,
            duration_days in 1..7i64,
            appointment_duration_minutes in 15..120i64,
            busy_count in 1..5usize, // At least one busy period
            max_busy_duration_hours in 1..4i64,
        ) {
            // Create a valid time range
            let (start, end) = create_time_range(start_offset_hours, duration_days);

            // Create work hours (full day for simplicity)
            let work_start = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
            let work_end = NaiveTime::from_hms_opt(23, 59, 59).unwrap();

            // Create busy periods
            let busy_periods = create_busy_periods(start, busy_count, max_busy_duration_hours);

            // Create appointment duration
            let appointment_duration = Duration::minutes(appointment_duration_minutes);

            // Define working days (all days for simplicity)
            let working_days = [
                Weekday::Mon, Weekday::Tue, Weekday::Wed,
                Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun
            ];

            // Calculate available slots
            let slots = calculate_available_slots(
                start,
                end,
                &busy_periods,
                appointment_duration,
                work_start,
                work_end,
                &working_days,
                Duration::minutes(0), // No buffer
                Duration::minutes(15), // 15-minute step
            );

            // Check that no slot overlaps with any busy period
            for (start_str, _end_str) in &slots {
                let slot = parse_datetime(start_str);
                let slot_end = slot + appointment_duration; // Add duration to get end time

                for (busy_start, busy_end) in &busy_periods {
                    // Check for overlap: (StartA < EndB) and (EndA > StartB)
                    let overlaps = &slot < busy_end && &slot_end > busy_start;

                    prop_assert!(!overlaps,
                        "Slot {:?} to {:?} overlaps with busy period {:?} to {:?}",
                        slot, slot_end, busy_start, busy_end);
                }
            }
        }

        // Test that slots are properly spaced
        #[test]
        fn test_slots_properly_spaced(
            start_offset_hours in 0..24i64,
            duration_days in 1..7i64,
            appointment_duration_minutes in 15..120i64,
            buffer_minutes in 0..30i64,
        ) {
            // Create a valid time range
            let (start, end) = create_time_range(start_offset_hours, duration_days);

            // Create work hours (full day for simplicity)
            let work_start = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
            let work_end = NaiveTime::from_hms_opt(23, 59, 59).unwrap();

            // No busy periods
            let busy_periods = Vec::new();

            // Create appointment duration and buffer
            let appointment_duration = Duration::minutes(appointment_duration_minutes);
            let buffer = Duration::minutes(buffer_minutes);

            // Define working days (all days for simplicity)
            let working_days = [
                Weekday::Mon, Weekday::Tue, Weekday::Wed,
                Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun
            ];

            // Calculate available slots
            let slots = calculate_available_slots(
                start,
                end,
                &busy_periods,
                appointment_duration,
                work_start,
                work_end,
                &working_days,
                buffer,
                Duration::minutes(15), // 15-minute step
            );

            // Check that slots are properly spaced
            for i in 1..slots.len() {
                let current_slot = parse_datetime(&slots[i].0);
                let prev_slot = parse_datetime(&slots[i-1].0);
                let time_diff = current_slot - prev_slot;

                // The difference should be at least the appointment duration plus buffer
                prop_assert!(time_diff >= appointment_duration + buffer,
                    "Slots should be at least duration + buffer apart: {:?} and {:?}, diff: {:?}, expected: {:?}",
                    prev_slot, current_slot, time_diff, appointment_duration + buffer);
            }
        }
    }
}
