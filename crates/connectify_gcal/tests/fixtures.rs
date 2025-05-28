//! Test fixtures for Google Calendar tests
//!
//! This module provides common test fixtures and factory functions
//! to create test data for Google Calendar tests.

use chrono::{DateTime, Duration, NaiveTime, Utc, Weekday};
use connectify_common::services::CalendarEvent;
use connectify_config::{AppConfig, GcalConfig, PriceTier, StripeConfig};
use std::sync::Arc;

/// Creates a test calendar event with the given parameters
pub fn create_test_calendar_event(
    start_time_offset_hours: i64,
    duration_minutes: i64,
    summary: &str,
    description: Option<&str>,
) -> CalendarEvent {
    let start_time = Utc::now() + Duration::hours(start_time_offset_hours);
    let end_time = start_time + Duration::minutes(duration_minutes);

    CalendarEvent {
        start_time: start_time.to_rfc3339(),
        end_time: end_time.to_rfc3339(),
        summary: summary.to_string(),
        description: description.map(|s| s.to_string()),
        payment_id: None,
        payment_amount: None,
        payment_method: None,
        room_name: None,
    }
}

/// Creates a list of busy periods for testing
pub fn create_busy_periods(
    base_time: DateTime<Utc>,
    count: usize,
    duration_hours: i64,
    gap_hours: i64,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let mut busy_periods = Vec::new();
    let mut current_time = base_time;

    for _ in 0..count {
        let start = current_time;
        let end = start + Duration::hours(duration_hours);
        busy_periods.push((start, end));
        current_time = end + Duration::hours(gap_hours);
    }

    busy_periods
}

/// Creates a mock AppConfig for testing
pub fn create_mock_config() -> Arc<AppConfig> {
    // Create price tiers
    let price_tiers = vec![
        PriceTier {
            duration_minutes: 30,
            unit_amount: 5000, // $50.00
            currency: Some("USD".to_string()),
            product_name: Some("30-minute consultation".to_string()),
        },
        PriceTier {
            duration_minutes: 60,
            unit_amount: 10000, // $100.00
            currency: Some("USD".to_string()),
            product_name: Some("60-minute consultation".to_string()),
        },
        PriceTier {
            duration_minutes: 90,
            unit_amount: 15000, // $150.00
            currency: Some("USD".to_string()),
            product_name: Some("90-minute consultation".to_string()),
        },
    ];

    // Create Stripe config
    let stripe_config = StripeConfig {
        success_url: "https://example.com/success".to_string(),
        cancel_url: "https://example.com/cancel".to_string(),
        unit_amount: Some(10000),
        product_name: Some("Test Product".to_string()),
        payment_success_url: "https://example.com/payment-success".to_string(),
        price_tiers,
        default_currency: Some("USD".to_string()),
    };

    // Create GCal config
    let gcal_config = GcalConfig {
        calendar_id: Some("primary".to_string()),
        time_slot_duration: Some(30),
        key_path: Some("test_key.json".to_string()),
        preparation_time_minutes: Some(120),
        time_zone: Some("Europe/Zurich".to_string()),
        working_days: Some(vec![
            "Mon".to_string(),
            "Tue".to_string(),
            "Wed".to_string(),
            "Thu".to_string(),
            "Fri".to_string(),
        ]),
        work_start_time: Some("09:00".to_string()),
        work_end_time: Some("17:00".to_string()),
    };

    // Create and return the AppConfig
    Arc::new(AppConfig {
        use_gcal: true,
        use_stripe: true,
        use_twilio: false,
        use_payrexx: false,
        use_fulfillment: false,
        use_calendly: false,
        use_adhoc: false,
        server: connectify_config::ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        database: Some(connectify_config::DatabaseConfig {
            url: "postgres://localhost/test".to_string(),
        }),
        gcal: Some(gcal_config),
        stripe: Some(stripe_config),
        twilio: None,
        payrexx: None,
        fulfillment: None,
        adhoc_settings: None,
    })
}

/// Creates working hours and days configuration for testing
#[allow(dead_code)]
pub fn create_working_hours_config() -> (NaiveTime, NaiveTime, [Weekday; 7]) {
    let work_start = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let work_end = NaiveTime::from_hms_opt(23, 59, 59).unwrap();
    let working_days = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    (work_start, work_end, working_days)
}

/// Creates a date range for testing
#[allow(dead_code)]
pub fn create_date_range(
    start_offset_days: i64,
    duration_days: i64,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let start = Utc::now() + Duration::days(start_offset_days);
    let end = start + Duration::days(duration_days);
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_calendar_event() {
        let event = create_test_calendar_event(1, 60, "Test Event", Some("Test Description"));

        assert_eq!(event.summary, "Test Event");
        assert_eq!(event.description, Some("Test Description".to_string()));

        // Parse the times to verify duration
        let start = DateTime::parse_from_rfc3339(&event.start_time).unwrap();
        let end = DateTime::parse_from_rfc3339(&event.end_time).unwrap();

        let duration = end - start;
        assert_eq!(duration, Duration::minutes(60));
    }

    #[test]
    fn test_create_busy_periods() {
        let base_time = Utc::now();
        let busy_periods = create_busy_periods(base_time, 3, 2, 1);

        assert_eq!(busy_periods.len(), 3);

        // Check first period
        let (start1, end1) = busy_periods[0];
        assert_eq!(start1, base_time);
        assert_eq!(end1, base_time + Duration::hours(2));

        // Check second period
        let (start2, end2) = busy_periods[1];
        assert_eq!(start2, end1 + Duration::hours(1));
        assert_eq!(end2, start2 + Duration::hours(2));
    }

    #[test]
    fn test_create_mock_config() {
        let config = create_mock_config();

        // Check GCal config
        assert!(config.use_gcal);
        assert!(config.gcal.is_some());
        let gcal_config = config.gcal.as_ref().unwrap();
        assert_eq!(gcal_config.calendar_id, Some("primary".to_string()));

        // Check Stripe config
        assert!(config.use_stripe);
        assert!(config.stripe.is_some());
        let stripe_config = config.stripe.as_ref().unwrap();
        assert_eq!(stripe_config.price_tiers.len(), 3);
        assert_eq!(stripe_config.price_tiers[0].duration_minutes, 30);
        assert_eq!(stripe_config.price_tiers[1].duration_minutes, 60);
        assert_eq!(stripe_config.price_tiers[2].duration_minutes, 90);
    }
}
