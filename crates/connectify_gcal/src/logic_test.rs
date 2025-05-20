#[cfg(test)]
mod tests {
    use crate::logic::calculate_available_slots;
    use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeDelta, TimeZone, Utc, Weekday};
    use tracing::info;
    #[test]
    fn test_calculate_available_slots_no_busy_periods() {
        // Test case: No busy periods, should return slots at regular intervals
        // Use a fixed date (Monday) for deterministic testing
        let query_start = Utc.ymd(2025, 5, 5).and_hms(0, 0, 0); // Monday
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
            for slot in &slots {
                let slot_time = slot.time();
                info!(
                    "Slot time: {:?}, work_start: {:?}, work_end: {:?}",
                    slot_time, work_start, work_end
                );
                assert!(
                    slot_time >= work_start && slot_time <= work_end,
                    "Slot should be within working hours: {:?}",
                    slot
                );
            }

            // Check that slots are properly spaced
            for i in 1..slots.len() {
                let time_diff = slots[i] - slots[i - 1];
                assert!(
                    time_diff >= duration,
                    "Slots should be at least duration apart: {:?} and {:?}",
                    slots[i - 1],
                    slots[i]
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
        let query_start = Utc.ymd(2025, 5, 5).and_hms(0, 0, 0); // Monday
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
            for slot in &slots {
                let slot_end = *slot + duration;
                assert!(
                    slot_end <= busy_start || *slot >= busy_end,
                    "Slot should not overlap with busy period: {:?} to {:?}",
                    slot,
                    slot_end
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
        let query_start = Utc.ymd(2025, 5, 5).and_hms(0, 0, 0); // Monday
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
                let time_diff = slots[i] - slots[i - 1];
                assert!(
                    time_diff >= duration + buffer,
                    "Slots should be at least duration + buffer apart: {:?} and {:?}",
                    slots[i - 1],
                    slots[i]
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
        let query_start = Utc.ymd(2025, 5, 5).and_hms(0, 0, 0); // Monday
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
            for slot in &slots {
                let slot_time = slot.time();
                let slot_end_time = (*slot + duration).time();

                assert!(
                    slot_time >= work_start,
                    "Slot should start after work hours begin: {:?}",
                    slot
                );
                assert!(
                    slot_end_time <= work_end,
                    "Slot should end before work hours end: {:?} + {:?} = {:?}",
                    slot,
                    duration,
                    *slot + duration
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
