

// --- File: crates/connectify_gcal/src/test.rs ---
//! Tests for the Google Calendar service.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use chrono::{DateTime, Duration, Utc};
    use connectify_common::services::{CalendarEvent, CalendarService};
    use crate::service::mock::MockCalendarService;

    #[tokio::test]
    async fn test_create_and_get_events() {
        // Create a mock calendar service
        let service = MockCalendarService::new();
        
        // Create a test calendar ID
        let calendar_id = "test-calendar";
        
        // Create a test event
        let now = Utc::now();
        let start_time = now + Duration::hours(1);
        let end_time = start_time + Duration::hours(1);
        
        let event = CalendarEvent {
            start_time: start_time.to_rfc3339(),
            end_time: end_time.to_rfc3339(),
            summary: "Test Event".to_string(),
            description: Some("This is a test event".to_string()),
        };
        
        // Create the event
        let result = service.create_event(calendar_id, event.clone()).await.unwrap();
        
        // Verify the event was created
        assert!(result.event_id.is_some());
        assert_eq!(result.status, "confirmed");
        
        // Get the booked events
        let events = service.get_booked_events(
            calendar_id,
            now,
            now + Duration::hours(3),
            false,
        ).await.unwrap();
        
        // Verify the event is in the list
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "Test Event");
        assert_eq!(events[0].description, Some("This is a test event".to_string()));
        
        // Mark the event as cancelled
        let event_id = result.event_id.unwrap();
        let cancel_result = service.mark_event_cancelled(calendar_id, &event_id, true).await.unwrap();
        
        // Verify the event was cancelled
        assert_eq!(cancel_result.status, "cancelled");
        
        // Get the booked events (excluding cancelled)
        let events = service.get_booked_events(
            calendar_id,
            now,
            now + Duration::hours(3),
            false,
        ).await.unwrap();
        
        // Verify the event is not in the list
        assert_eq!(events.len(), 0);
        
        // Get the booked events (including cancelled)
        let events = service.get_booked_events(
            calendar_id,
            now,
            now + Duration::hours(3),
            true,
        ).await.unwrap();
        
        // Verify the event is in the list
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].status, "cancelled");
        
        // Delete the event
        service.delete_event(calendar_id, &event_id, true).await.unwrap();
        
        // Get the booked events (including cancelled)
        let events = service.get_booked_events(
            calendar_id,
            now,
            now + Duration::hours(3),
            true,
        ).await.unwrap();
        
        // Verify the event is not in the list
        assert_eq!(events.len(), 0);
    }
    
    #[tokio::test]
    async fn test_busy_times() {
        // Create a mock calendar service
        let service = MockCalendarService::new();
        
        // Create a test calendar ID
        let calendar_id = "test-calendar";
        
        // Create a test event
        let now = Utc::now();
        let start_time = now + Duration::hours(1);
        let end_time = start_time + Duration::hours(1);
        
        let event = CalendarEvent {
            start_time: start_time.to_rfc3339(),
            end_time: end_time.to_rfc3339(),
            summary: "Test Event".to_string(),
            description: Some("This is a test event".to_string()),
        };
        
        // Create the event
        service.create_event(calendar_id, event.clone()).await.unwrap();
        
        // Get the busy times
        let busy_times = service.get_busy_times(
            calendar_id,
            now,
            now + Duration::hours(3),
        ).await.unwrap();
        
        // Verify the busy times
        assert_eq!(busy_times.len(), 1);
        
        let (busy_start, busy_end) = busy_times[0];
        assert_eq!(busy_start, DateTime::parse_from_rfc3339(&event.start_time).unwrap().with_timezone(&Utc));
        assert_eq!(busy_end, DateTime::parse_from_rfc3339(&event.end_time).unwrap().with_timezone(&Utc));
        
        // Try to create an overlapping event
        let overlapping_event = CalendarEvent {
            start_time: (start_time + Duration::minutes(30)).to_rfc3339(),
            end_time: (end_time + Duration::minutes(30)).to_rfc3339(),
            summary: "Overlapping Event".to_string(),
            description: None,
        };
        
        // This should fail with a conflict error
        let result = service.create_event(calendar_id, overlapping_event.clone()).await;
        assert!(result.is_err());
        
        // Create a non-overlapping event
        let non_overlapping_event = CalendarEvent {
            start_time: (end_time + Duration::minutes(30)).to_rfc3339(),
            end_time: (end_time + Duration::minutes(90)).to_rfc3339(),
            summary: "Non-overlapping Event".to_string(),
            description: None,
        };
        
        // This should succeed
        let result = service.create_event(calendar_id, non_overlapping_event.clone()).await;
        assert!(result.is_ok());
        
        // Get the busy times again
        let busy_times = service.get_busy_times(
            calendar_id,
            now,
            now + Duration::hours(5),
        ).await.unwrap();
        
        // Verify the busy times
        assert_eq!(busy_times.len(), 2);
    }
}
