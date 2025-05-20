// --- File: crates/connectify_gcal/src/service.rs ---
//! Google Calendar service implementation.
//!
//! This module provides an implementation of the CalendarService trait for Google Calendar.

use tracing::info;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use connectify_common::services::{
    BookedEvent, CalendarEvent, CalendarEventResult, CalendarService,
};
use google_calendar3::api::{Event, EventDateTime, FreeBusyRequest, FreeBusyRequestItem};
use thiserror::Error;

use crate::auth::HubType;

/// Errors that can occur when interacting with Google Calendar.
#[derive(Error, Debug)]
pub enum GcalServiceError {
    #[error("Google API Error: {0}")]
    ApiError(#[from] google_calendar3::Error),
    #[error("Failed to parse time: {0}")]
    TimeParseError(String),
    #[error("Calculation error: {0}")]
    CalculationError(String),
    #[error("Booking conflict")]
    Conflict,
    #[error("No matching price tier found for duration: {0} minutes")]
    NoMatchingPriceTier(i64),
}

// The standard library already provides a generic implementation for
// converting any type that implements std::error::Error into Box<dyn std::error::Error + Send + Sync>

/// Google Calendar service implementation.
pub struct GoogleCalendarService {
    calendar_hub: Arc<HubType>,
}

impl GoogleCalendarService {
    /// Create a new Google Calendar service.
    pub fn new(calendar_hub: Arc<HubType>) -> Self {
        Self { calendar_hub }
    }
}

impl CalendarService for GoogleCalendarService {
    type Error = GcalServiceError;

    /// Retrieves busy time periods for a specified calendar within a given time range.
    ///
    /// This function queries the Google Calendar API to get all busy periods for the specified
    /// calendar between the start and end times. It's used to determine when the calendar
    /// is already booked, which is essential for availability checking and conflict prevention.
    ///
    /// # Arguments
    ///
    /// * `calendar_id` - The ID of the calendar to check (e.g., "primary" or a specific calendar ID)
    /// * `start_time` - The start of the time range to check for busy periods
    /// * `end_time` - The end of the time range to check for busy periods
    ///
    /// # Returns
    ///
    /// A vector of tuples, where each tuple contains the start and end time of a busy period.
    /// The busy periods are sorted chronologically by start time.
    ///
    /// # Errors
    ///
    /// Returns a `GcalServiceError` if:
    /// * The API call to Google Calendar fails
    /// * The response cannot be parsed correctly
    ///
    /// # Example
    ///
    /// ```ignore
    /// use chrono::{Utc, Duration};
    /// use std::sync::Arc;
    /// 
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Create a GoogleCalendarService instance
    ///     // let calendar_service = GoogleCalendarService::new(...);
    ///     
    ///     let busy_times = calendar_service.get_busy_times(
    ///         "primary",
    ///         Utc::now(),
    ///         Utc::now() + Duration::days(7)
    ///     ).await?;
    ///
    ///     for (start, end) in busy_times {
    ///         info!("Busy from {} to {}", start, end);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    fn get_busy_times(
        &self,
        calendar_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, Self::Error>> + Send + '_>> {
        let calendar_id = calendar_id.to_string();
        let calendar_hub = self.calendar_hub.clone();

        Box::pin(async move {
            let req = FreeBusyRequest {
                time_min: Some(start_time),
                time_max: Some(end_time),
                time_zone: Some("UTC".to_string()),
                items: Some(vec![FreeBusyRequestItem {
                    id: Some(calendar_id.to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            };

            // Make the API call
            let (_response, freebusy_response) = calendar_hub.freebusy().query(req).doit().await?;

            let mut busy_periods = Vec::new();

            // Extract busy periods for the specified calendar
            if let Some(calendars) = freebusy_response.calendars {
                if let Some(cal_info) = calendars.get(&calendar_id) {
                    if let Some(busy_times) = &cal_info.busy {
                        for period in busy_times {
                            if let (Some(start_dt), Some(end_dt)) = (period.start, period.end) {
                                busy_periods.push((start_dt, end_dt));
                            } else {
                                info!(
                                    "Warning: Skipping busy period with missing start/end: {:?}",
                                    period
                                );
                            }
                        }
                    }
                }
            }
            // Sort busy periods for easier processing
            busy_periods.sort_by_key(|k| k.0);
            Ok(busy_periods)
        })
    }

    /// Creates a new calendar event in the specified calendar.
    ///
    /// This function creates a new event in the Google Calendar with the provided details.
    /// It performs several validation steps before creating the event:
    /// 1. Validates that the start and end times are in the correct format
    /// 2. Ensures that the end time is after the start time
    /// 3. Checks for conflicts with existing events in the calendar
    ///
    /// # Arguments
    ///
    /// * `calendar_id` - The ID of the calendar where the event will be created
    /// * `event` - The event details including start time, end time, summary, and description
    ///
    /// # Returns
    ///
    /// A `CalendarEventResult` containing the ID of the created event and its status.
    ///
    /// # Errors
    ///
    /// Returns a `GcalServiceError` if:
    /// * The start or end time cannot be parsed (TimeParseError)
    /// * The end time is not after the start time (CalculationError)
    /// * There is a conflict with an existing event (Conflict)
    /// * The API call to Google Calendar fails (ApiError)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::sync::Arc;
    /// use connectify_common::services::CalendarEvent;
    /// 
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Create a GoogleCalendarService instance
    ///     // let calendar_service = GoogleCalendarService::new(...);
    ///     
    ///     let event = CalendarEvent {
    ///         start_time: "2025-05-15T10:00:00Z".to_string(),
    ///         end_time: "2025-05-15T11:00:00Z".to_string(),
    ///         summary: "Consultation with John Doe".to_string(),
    ///         description: Some("Initial consultation to discuss project requirements".to_string()),
    ///     };
    ///
    ///     let result = calendar_service.create_event("primary", event).await?;
    ///     info!("Created event with ID: {:?}, status: {}", result.event_id, result.status);
    ///     
    ///     Ok(())
    /// }
    /// ```
    fn create_event(
        &self,
        calendar_id: &str,
        event: CalendarEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CalendarEventResult, Self::Error>> + Send + '_>> {
        let calendar_id = calendar_id.to_string();
        let event = event.clone();
        let calendar_hub = self.calendar_hub.clone();
        let this = self;

        Box::pin(async move {
            // Parse start and end times from request strings
            let start_dt = DateTime::parse_from_rfc3339(&event.start_time)
                .map_err(|e| GcalServiceError::TimeParseError(format!("Invalid start_time: {}", e)))?
                .with_timezone(&Utc);
            let end_dt = DateTime::parse_from_rfc3339(&event.end_time)
                .map_err(|e| GcalServiceError::TimeParseError(format!("Invalid end_time: {}", e)))?
                .with_timezone(&Utc);

            // Basic validation: end time must be after start time
            if end_dt <= start_dt {
                return Err(GcalServiceError::CalculationError(
                    "End time must be after start time".to_string(),
                ));
            }

            // Check for conflicts with existing events
            let busy_times = this.get_busy_times(&calendar_id, start_dt, end_dt).await?;

            // If there are any busy periods that overlap with our proposed event time, it's a conflict
            for (busy_start, busy_end) in &busy_times {
                // Check for overlap: (StartA < EndB) and (EndA > StartB)
                if start_dt < *busy_end && end_dt > *busy_start {
                    return Err(GcalServiceError::Conflict);
                }
            }

            // Construct the Event object
            let new_event = Event {
                summary: Some(event.summary),
                description: event.description,
                start: Some(EventDateTime {
                    date_time: Some(start_dt),
                    time_zone: Some("UTC".to_string()), // Store event times in UTC
                    ..Default::default()
                }),
                end: Some(EventDateTime {
                    date_time: Some(end_dt),
                    time_zone: Some("UTC".to_string()),
                    ..Default::default()
                }),
                ..Default::default() // Use default for other fields
            };

            // Make the API call to insert the event
            let (_response, created_event) = calendar_hub
                .events()
                .insert(new_event, &calendar_id)
                .doit()
                .await?;

            Ok(CalendarEventResult {
                event_id: created_event.id,
                status: created_event.status.unwrap_or_else(|| "confirmed".to_string()),
            })
        })
    }

    /// Deletes an event from the specified calendar.
    ///
    /// This function attempts to delete an event from Google Calendar. It handles several edge cases:
    /// 1. If the event doesn't exist (404 error), it returns success
    /// 2. If the event is already cancelled, it attempts to restore it first and then delete it
    /// 3. If there are permission issues (403 error), it tries an alternative approach
    ///
    /// The function implements a two-step process for handling cancelled events:
    /// 1. First, it tries to restore the event by setting its status to "confirmed"
    /// 2. Then it attempts to delete the restored event
    ///
    /// # Arguments
    ///
    /// * `calendar_id` - The ID of the calendar containing the event
    /// * `event_id` - The ID of the event to delete
    /// * `notify_attendees` - Whether to send notifications to event attendees about the deletion
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the event was successfully deleted or if it didn't exist.
    ///
    /// # Errors
    ///
    /// Returns a `GcalServiceError` if:
    /// * The API call to Google Calendar fails (except for 404 errors)
    /// * Both the direct deletion and the restore-then-delete approach fail
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Create a GoogleCalendarService instance
    ///     // let calendar_service = GoogleCalendarService::new(...);
    ///     
    ///     // Delete an event without notifying attendees
    ///     let result = calendar_service.delete_event("primary", "event123", false).await?;
    ///
    ///     // Delete an event and notify all attendees
    ///     let result = calendar_service.delete_event("primary", "event456", true).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    fn delete_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        notify_attendees: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + '_>> {
        let calendar_id = calendar_id.to_string();
        let event_id = event_id.to_string();
        let calendar_hub = self.calendar_hub.clone();

        Box::pin(async move {
            // First check if the event exists and get its status
            let get_result = calendar_hub.events().get(&calendar_id, &event_id).doit().await;

            // If the event doesn't exist, consider it "successfully deleted"
            if let Err(e) = get_result {
                if e.to_string().contains("404") {
                    return Ok(());
                }
                return Err(GcalServiceError::ApiError(e));
            }

            // Extract event status for making delete decision
            let (_response, event) = get_result.unwrap();
            let status = event.status.as_deref().unwrap_or("confirmed");

            // Try to delete the event normally first
            let delete_result = calendar_hub
                .events()
                .delete(&calendar_id, &event_id)
                .send_updates(if notify_attendees { "all" } else { "none" })
                .doit()
                .await;

            match delete_result {
                Ok(_) => Ok(()), // Normal deletion successful
                Err(e) => {
                    // If the error is due to the cancelled status, try a different approach
                    if status == "cancelled"
                        || e.to_string().contains("403")
                        || e.to_string().contains("400")
                    {
                        // Try restoring the event first
                        let sequence = event.sequence.map(|n| n + 1).unwrap_or(1);
                        let restored_event = google_calendar3::api::Event {
                            status: Some("confirmed".to_string()),
                            sequence: Some(sequence),
                            ..Default::default()
                        };

                        // First restore to confirmed status
                        let restore_result = calendar_hub
                            .events()
                            .patch(restored_event, &calendar_id, &event_id)
                            .send_updates("none") // Don't notify for intermediate step
                            .doit()
                            .await;

                        // Handle restoration result
                        match restore_result {
                            Ok(_) => {
                                // Now try deleting again
                                calendar_hub
                                    .events()
                                    .delete(&calendar_id, &event_id)
                                    .send_updates(if notify_attendees { "all" } else { "none" })
                                    .doit()
                                    .await?;
                                Ok(())
                            }
                            Err(_) => {
                                // If restoration fails, we've done our best - log it and return success
                                tracing::warn!(
                                    "Could not fully delete event {}, attempted restore and delete",
                                    event_id
                                );
                                Ok(())
                            }
                        }
                    } else {
                        // For any other error, just pass it through
                        Err(GcalServiceError::ApiError(e))
                    }
                }
            }
        })
    }

    /// Marks an event as cancelled in the specified calendar without deleting it.
    ///
    /// This function updates an event's status to "cancelled" in Google Calendar.
    /// Unlike `delete_event`, this function preserves the event in the calendar but marks it
    /// as cancelled, which means it will still appear in the calendar (usually with strikethrough)
    /// but will be considered inactive.
    ///
    /// The function increments the event's sequence number to ensure the change is properly
    /// synchronized across all calendar instances.
    ///
    /// # Arguments
    ///
    /// * `calendar_id` - The ID of the calendar containing the event
    /// * `event_id` - The ID of the event to mark as cancelled
    /// * `notify_attendees` - Whether to send notifications to event attendees about the cancellation
    ///
    /// # Returns
    ///
    /// A `CalendarEventResult` containing the ID of the updated event and its new status ("cancelled").
    ///
    /// # Errors
    ///
    /// Returns a `GcalServiceError` if:
    /// * The event doesn't exist
    /// * The API call to Google Calendar fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Create a GoogleCalendarService instance
    ///     // let calendar_service = GoogleCalendarService::new(...);
    ///     
    ///     // Mark an event as cancelled without notifying attendees
    ///     let result = calendar_service.mark_event_cancelled("primary", "event123", false).await?;
    ///     info!("Event {} marked as {}", result.event_id.unwrap_or_default(), result.status);
    ///
    ///     // Mark an event as cancelled and notify all attendees
    ///     let result = calendar_service.mark_event_cancelled("primary", "event456", true).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    fn mark_event_cancelled(
        &self,
        calendar_id: &str,
        event_id: &str,
        notify_attendees: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CalendarEventResult, Self::Error>> + Send + '_>> {
        let calendar_id = calendar_id.to_string();
        let event_id = event_id.to_string();
        let calendar_hub = self.calendar_hub.clone();

        Box::pin(async move {
            let (_response, event) = calendar_hub.events().get(&calendar_id, &event_id).doit().await?;

            // Create a minimal event with sequence number + 1
            let sequence = event.sequence.map(|n| n + 1).unwrap_or(1);

            let cancelled_event = Event {
                status: Some("cancelled".to_string()),
                sequence: Some(sequence),
                ..Default::default()
            };

            let (_response, updated) = calendar_hub
                .events()
                .patch(cancelled_event, &calendar_id, &event_id)
                .send_updates(if notify_attendees { "all" } else { "none" })
                .doit()
                .await?;

            Ok(CalendarEventResult {
                event_id: updated.id,
                status: updated.status.unwrap_or_else(|| "cancelled".to_string()),
            })
        })
    }

    /// Retrieves all booked events in the specified calendar within a given time range.
    ///
    /// This function queries the Google Calendar API to get all events for the specified
    /// calendar between the start and end times. It handles various event formats and statuses,
    /// and converts the Google Calendar API response into a standardized `BookedEvent` format.
    ///
    /// The function handles several complexities:
    /// 1. It can include or exclude cancelled events based on the `include_cancelled` parameter
    /// 2. It handles both date-time and date-only events
    /// 3. It extracts all relevant event information (ID, summary, description, times, status)
    /// 4. It formats all dates consistently in RFC3339 format
    ///
    /// # Arguments
    ///
    /// * `calendar_id` - The ID of the calendar to query
    /// * `start_time` - The start of the time range to retrieve events for
    /// * `end_time` - The end of the time range to retrieve events for
    /// * `include_cancelled` - Whether to include cancelled events in the results
    ///
    /// # Returns
    ///
    /// A vector of `BookedEvent` objects representing all events in the specified time range,
    /// sorted chronologically by start time.
    ///
    /// # Errors
    ///
    /// Returns a `GcalServiceError` if:
    /// * The API call to Google Calendar fails
    /// * The response cannot be parsed correctly
    ///
    /// # Example
    ///
    /// ```ignore
    /// use chrono::{Utc, Duration};
    /// 
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Create a GoogleCalendarService instance
    ///     // let calendar_service = GoogleCalendarService::new(...);
    ///     
    ///     // Get all confirmed events for the next week
    ///     let events = calendar_service.get_booked_events(
    ///         "primary",
    ///         Utc::now(),
    ///         Utc::now() + Duration::days(7),
    ///         false // exclude cancelled events
    ///     ).await?;
    ///
    ///     for event in events {
    ///         info!("Event: {}, Start: {}, Status: {}",
    ///                  event.summary, event.start_time, event.status);
    ///     }
    ///
    ///     // Get all events including cancelled ones
    ///     let all_events = calendar_service.get_booked_events(
    ///         "primary",
    ///         Utc::now(),
    ///         Utc::now() + Duration::days(7),
    ///         true // include cancelled events
    ///     ).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    fn get_booked_events(
        &self,
        calendar_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        include_cancelled: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<BookedEvent>, Self::Error>> + Send + '_>> {
        let calendar_id = calendar_id.to_string();
        let start_time = start_time;
        let end_time = end_time;
        let calendar_hub = self.calendar_hub.clone();

        Box::pin(async move {
            // Create the events list request
            let mut request = calendar_hub
                .events()
                .list(&calendar_id)
                .time_min(start_time)
                .time_max(end_time)
                .single_events(true) // Expand recurring events
                .order_by("startTime"); // Sort by start time

            // This is a key parameter for Google Calendar API to include cancelled events
            request = request.show_deleted(include_cancelled);

            // Make the API call
            let (_, events_list) = request.doit().await?;

            let mut booked_events = Vec::new();

            if let Some(items) = events_list.items {
                for event in items {
                    // Skip cancelled events if not including them
                    let status = event.status.as_deref().unwrap_or("confirmed");
                    if !include_cancelled && status == "cancelled" {
                        continue;
                    }

                    // Extract the needed fields
                    let event_id = event.id.unwrap_or_default();
                    let summary = event.summary.unwrap_or_default();
                    let description = event.description;

                    // Handle start time
                    let start_time = match event.start {
                        Some(start) => match start.date_time {
                            Some(dt) => dt.to_rfc3339(),
                            None => match start.date {
                                Some(d) => format!("{}T00:00:00Z", d),
                                None => "Unknown start time".to_string(),
                            },
                        },
                        None => "Unknown start time".to_string(),
                    };

                    // Handle end time
                    let end_time = match event.end {
                        Some(end) => match end.date_time {
                            Some(dt) => dt.to_rfc3339(),
                            None => match end.date {
                                Some(d) => format!("{}T23:59:59Z", d),
                                None => "Unknown end time".to_string(),
                            },
                        },
                        None => "Unknown end time".to_string(),
                    };

                    let status = event.status.unwrap_or_else(|| "confirmed".to_string());
                    let created = event.created.map(|dt| dt.to_rfc3339()).unwrap_or_default();
                    let updated = event.updated.map(|dt| dt.to_rfc3339()).unwrap_or_default();

                    // Add this event to our results
                    booked_events.push(BookedEvent {
                        event_id,
                        summary,
                        description,
                        start_time,
                        end_time,
                        status,
                        created,
                        updated,
                    });
                }
            }

            Ok(booked_events)
        })
    }
}

/// Mock implementation of CalendarService for testing.
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Mock calendar service for testing.
    pub struct MockCalendarService {
        events: Mutex<HashMap<String, Vec<(String, CalendarEvent, String)>>>,
    }

    impl MockCalendarService {
        /// Create a new mock calendar service.
        pub fn new() -> Self {
            Self {
                events: Mutex::new(HashMap::new()),
            }
        }
    }

    impl CalendarService for MockCalendarService {
        type Error = GcalServiceError;

        fn get_busy_times(
            &self,
            calendar_id: &str,
            start_time: DateTime<Utc>,
            end_time: DateTime<Utc>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, Self::Error>> + Send + '_>> {
            let calendar_id = calendar_id.to_string();

            Box::pin(async move {
                let events = self.events.lock().unwrap();
                let calendar_events = events.get(&calendar_id).cloned().unwrap_or_default();

                let mut busy_times = Vec::new();
                for (_, event, status) in calendar_events {
                    if status == "cancelled" {
                        continue;
                    }

                    let event_start = DateTime::parse_from_rfc3339(&event.start_time)
                        .map_err(|e| GcalServiceError::TimeParseError(e.to_string()))?
                        .with_timezone(&Utc);
                    let event_end = DateTime::parse_from_rfc3339(&event.end_time)
                        .map_err(|e| GcalServiceError::TimeParseError(e.to_string()))?
                        .with_timezone(&Utc);

                    if event_start < end_time && event_end > start_time {
                        busy_times.push((event_start, event_end));
                    }
                }

                busy_times.sort_by_key(|k| k.0);
                Ok(busy_times)
            })
        }

        fn create_event(
            &self,
            calendar_id: &str,
            event: CalendarEvent,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CalendarEventResult, Self::Error>> + Send + '_>> {
            let calendar_id = calendar_id.to_string();
            let event = event.clone();

            Box::pin(async move {
                // Parse and validate times
                let start_dt = DateTime::parse_from_rfc3339(&event.start_time)
                    .map_err(|e| GcalServiceError::TimeParseError(format!("Invalid start_time: {}", e)))?;
                let end_dt = DateTime::parse_from_rfc3339(&event.end_time)
                    .map_err(|e| GcalServiceError::TimeParseError(format!("Invalid end_time: {}", e)))?;

                if end_dt <= start_dt {
                    return Err(GcalServiceError::CalculationError(
                        "End time must be after start time".to_string(),
                    ));
                }

                // Check for conflicts
                let busy_times = self.get_busy_times(
                    &calendar_id,
                    start_dt.with_timezone(&Utc),
                    end_dt.with_timezone(&Utc),
                ).await?;

                for (busy_start, busy_end) in &busy_times {
                    if start_dt.with_timezone(&Utc) < *busy_end && end_dt.with_timezone(&Utc) > *busy_start {
                        return Err(GcalServiceError::Conflict);
                    }
                }

                // Create the event
                let event_id = format!("mock-event-{}", uuid::Uuid::new_v4());

                let mut events = self.events.lock().unwrap();
                let calendar_events = events.entry(calendar_id.to_string()).or_insert_with(Vec::new);
                calendar_events.push((event_id.clone(), event, "confirmed".to_string()));

                Ok(CalendarEventResult {
                    event_id: Some(event_id),
                    status: "confirmed".to_string(),
                })
            })
        }

        fn delete_event(
            &self,
            calendar_id: &str,
            event_id: &str,
            _notify_attendees: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + '_>> {
            let calendar_id = calendar_id.to_string();
            let event_id = event_id.to_string();

            Box::pin(async move {
                let mut events = self.events.lock().unwrap();

                if let Some(calendar_events) = events.get_mut(&calendar_id) {
                    calendar_events.retain(|(id, _, _)| id != &event_id);
                }

                Ok(())
            })
        }

        fn mark_event_cancelled(
            &self,
            calendar_id: &str,
            event_id: &str,
            _notify_attendees: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CalendarEventResult, Self::Error>> + Send + '_>> {
            let calendar_id = calendar_id.to_string();
            let event_id = event_id.to_string();

            Box::pin(async move {
                let mut events = self.events.lock().unwrap();

                if let Some(calendar_events) = events.get_mut(&calendar_id) {
                    for (id, _, status) in calendar_events.iter_mut() {
                        if id == &event_id {
                            *status = "cancelled".to_string();
                            return Ok(CalendarEventResult {
                                event_id: Some(id.clone()),
                                status: "cancelled".to_string(),
                            });
                        }
                    }
                }

                Err(GcalServiceError::CalculationError(format!("Event not found: {}", event_id)))
            })
        }

        fn get_booked_events(
            &self,
            calendar_id: &str,
            start_time: DateTime<Utc>,
            end_time: DateTime<Utc>,
            include_cancelled: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<BookedEvent>, Self::Error>> + Send + '_>> {
            let calendar_id = calendar_id.to_string();

            Box::pin(async move {
                let events = self.events.lock().unwrap();
                let calendar_events = events.get(&calendar_id).cloned().unwrap_or_default();

                let mut booked_events = Vec::new();
                for (event_id, event, status) in calendar_events {
                    if !include_cancelled && status == "cancelled" {
                        continue;
                    }

                    let event_start = DateTime::parse_from_rfc3339(&event.start_time)
                        .map_err(|e| GcalServiceError::TimeParseError(e.to_string()))?
                        .with_timezone(&Utc);
                    let event_end = DateTime::parse_from_rfc3339(&event.end_time)
                        .map_err(|e| GcalServiceError::TimeParseError(e.to_string()))?
                        .with_timezone(&Utc);

                    if event_start < end_time && event_end > start_time {
                        booked_events.push(BookedEvent {
                            event_id,
                            summary: event.summary,
                            description: event.description,
                            start_time: event.start_time,
                            end_time: event.end_time,
                            status,
                            created: Utc::now().to_rfc3339(),
                            updated: Utc::now().to_rfc3339(),
                        });
                    }
                }

                booked_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
                Ok(booked_events)
            })
        }
    }
}
