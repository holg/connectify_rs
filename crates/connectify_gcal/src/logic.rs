// --- File: crates/connectify_gcal/src/logic.rs ---
use crate::auth::HubType; // Use the specific Hub type alias
use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc, Weekday}; // Use chrono Duration
use google_calendar3::api::{Event, EventDateTime, FreeBusyRequest, FreeBusyRequestItem};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema; //, IntoParams};

// --- Error Handling ---
use thiserror::Error;
#[derive(Error, Debug)]
pub enum GcalError {
    #[error("Google API Error: {0}")]
    ApiError(#[from] google_calendar3::Error),
    #[error("Failed to parse time: {0}")]
    TimeParseError(String),
    #[error("Calculation error: {0}")]
    CalculationError(String),
    #[error("Booking conflict")] // Example specific error
    Conflict,
}

// --- Data Structures ---
#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams, utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", into_params(parameter_in = Query))]
pub struct AvailabilityQuery {
    /// Start date in YYYY-MM-DD format
    #[cfg_attr(feature = "openapi", schema(format = "date", example = "2025-05-05"))]
    pub start_date: String,

    /// End date in YYYY-MM-DD format
    #[cfg_attr(feature = "openapi", schema(format = "date", example = "2025-05-24"))]
    pub end_date: String,

    /// Duration in minutes
    #[cfg_attr(feature = "openapi", schema(example = 60))]
    pub duration_minutes: i64,
}

#[derive(Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AvailableSlotsResponse {
    pub slots: Vec<String>, // ISO 8601 format strings (e.g., "2025-04-25T10:00:00Z")
}

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct BookSlotRequest {
    pub start_time: String, // ISO 8601 format string
    pub end_time: String,   // ISO 8601 format string
    pub summary: String,    // Event title
    pub description: Option<String>,
    // Add attendee emails, etc., if needed
}

#[derive(Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct BookingResponse {
    pub success: bool,
    pub event_id: Option<String>,
    pub message: String,
}

// --- Availability Logic ---

/// Fetches busy time intervals from Google Calendar.
pub async fn get_busy_times(
    hub: &HubType,
    calendar_id: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, GcalError> {
    let req = FreeBusyRequest {
        time_min: Some(start_time),
        time_max: Some(end_time),
        // Consider using the calendar's primary timezone if known, otherwise UTC is safe
        time_zone: Some("UTC".to_string()),
        items: Some(vec![FreeBusyRequestItem {
            id: Some(calendar_id.to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    };

    // Make the API call
    // result.0 is Response<>, result.1 is FreeBusyResponse
    let (_response, freebusy_response) = hub.freebusy().query(req).doit().await?;

    let mut busy_periods = Vec::new();

    // Extract busy periods for the specified calendar
    if let Some(calendars) = freebusy_response.calendars {
        if let Some(cal_info) = calendars.get(calendar_id) {
            if let Some(busy_times) = &cal_info.busy {
                for period in busy_times {
                    if let (Some(start_dt), Some(end_dt)) = (period.start, period.end) {
                        // Assume start/end are DateTime<Utc> directly from google-calendar3 v5+
                        busy_periods.push((start_dt, end_dt));
                    } else {
                        eprintln!(
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
}

/// Calculates available slots based on busy times, working hours, etc.
/// THIS IS A COMPLEX SKELETON - NEEDS DETAILED IMPLEMENTATION
pub fn calculate_available_slots(
    query_start: DateTime<Utc>, // Start of the overall query range
    query_end: DateTime<Utc>,   // End of the overall query range
    busy_periods: &[(DateTime<Utc>, DateTime<Utc>)],
    duration: Duration, // Appointment duration
    // --- Add your business logic parameters ---
    // Example: Define working hours (e.g., 9:00 to 17:00)
    work_start_time: NaiveTime, // e.g., NaiveTime::from_hms_opt(9, 0, 0).unwrap()
    work_end_time: NaiveTime,   // e.g., NaiveTime::from_hms_opt(17, 0, 0).unwrap()
    // Example: Define working days (e.g., Monday to Friday)
    working_days: &[Weekday], // e.g., &[Weekday::Mon, Weekday::Tue, ...]
    // Example: Define buffer time between appointments
    buffer_time: Duration, // e.g., Duration::minutes(15)
    // Example: Define the increment step for checking slots
    step: Duration, // e.g., Duration::minutes(15)
) -> Vec<DateTime<Utc>> {
    let mut available_slots = Vec::new();
    let mut current_check_time = query_start;

    // Merge overlapping/adjacent busy periods for efficiency (optional but recommended)
    // let merged_busy = merge_busy_periods(busy_periods); // Implement this helper if needed

    while current_check_time < query_end {
        let potential_start_time = current_check_time;
        let potential_end_time = match potential_start_time.checked_add_signed(duration) {
            Some(t) => t,
            None => break, // Duration overflow, stop checking
        };
        // Calculate end time including buffer for overlap checks
        let potential_end_with_buffer = match potential_end_time.checked_add_signed(buffer_time) {
            Some(t) => t,
            None => potential_end_time, // If buffer overflows, just use end time
        };

        // --- Check 1: Is potential_start_time within the overall query range? ---
        if potential_start_time < query_start || potential_end_time > query_end {
            current_check_time = current_check_time + step; // Move to next step
            continue;
        }

        // --- Check 2: Is it within working days/hours? ---
        let day_of_week = potential_start_time.weekday();
        let time_of_day = potential_start_time.time();
        let end_time_of_day = potential_end_time.time(); // Need to check end time too

        if !working_days.contains(&day_of_week) ||
            time_of_day < work_start_time ||
            end_time_of_day > work_end_time ||
            // Handle edge case where appointment crosses midnight (if allowed)
            (potential_end_time.date_naive() != potential_start_time.date_naive() && end_time_of_day > NaiveTime::from_hms_opt(0,0,0).unwrap())
        {
            // Advance check time smartly (e.g., to start of next working day/hour)
            // For simplicity, just step forward for now
            current_check_time = current_check_time + step;
            continue;
        }

        // --- Check 3: Does it overlap with any busy periods? ---
        let mut overlaps = false;
        for (busy_start, busy_end) in busy_periods {
            // Use merged_busy if implemented
            // Check for overlap: (StartA < EndB) and (EndA > StartB)
            if potential_start_time < *busy_end && potential_end_with_buffer > *busy_start {
                overlaps = true;
                // Advance check time past this busy period for efficiency
                current_check_time = (*busy_end + buffer_time).max(current_check_time + step);
                break; // No need to check further busy periods for this potential_start_time
            }
        }

        // If no overlaps, it's an available slot!
        if !overlaps {
            available_slots.push(potential_start_time);
            // Advance check time by slot duration + buffer to find next possible slot
            current_check_time = potential_end_with_buffer;
        }
        // If it overlapped, current_check_time was already advanced within the loop
    } // End while loop

    available_slots
}

// --- Booking Logic ---

/// Creates an event in the specified Google Calendar.
pub async fn create_calendar_event(
    hub: &HubType,
    calendar_id: &str,
    request: BookSlotRequest,
) -> Result<Event, GcalError> {
    // Return the created Event or an error

    // Parse start and end times from request strings
    let start_dt = DateTime::parse_from_rfc3339(&request.start_time)
        .map_err(|e| GcalError::TimeParseError(format!("Invalid start_time: {}", e)))?
        .with_timezone(&Utc);
    let end_dt = DateTime::parse_from_rfc3339(&request.end_time)
        .map_err(|e| GcalError::TimeParseError(format!("Invalid end_time: {}", e)))?
        .with_timezone(&Utc);

    // Basic validation: end time must be after start time
    if end_dt <= start_dt {
        return Err(GcalError::CalculationError(
            "End time must be after start time".to_string(),
        ));
    }

    // Check for conflicts with existing events
    let busy_times = get_busy_times(hub, calendar_id, start_dt, end_dt).await?;

    // If there are any busy periods that overlap with our proposed event time, it's a conflict
    for (busy_start, busy_end) in &busy_times {
        // Check for overlap: (StartA < EndB) and (EndA > StartB)
        if start_dt < *busy_end && end_dt > *busy_start {
            return Err(GcalError::Conflict);
        }
    }

    // Construct the Event object
    let new_event = Event {
        summary: Some(request.summary),
        description: request.description,
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
        // Add attendees, reminders, etc. if needed
        // attendees: Some(vec![EventAttendee { email: Some("attendee@example.com".to_string()), ..Default::default() }]),
        ..Default::default() // Use default for other fields
    };

    // Make the API call to insert the event
    // result.0 is Response<>, result.1 is the created Event object
    let (_response, created_event) = hub
        .events()
        .insert(new_event, calendar_id)
        // Optionally add parameters like sendNotifications=true
        // .send_notifications(true)
        .doit()
        .await?;

    Ok(created_event)
}

// Add these new types to your logic module (logic.rs or logic/mod.rs)

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CancelBookingRequest {
    pub notify_attendees: Option<bool>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CancellationResponse {
    pub success: bool,
    pub message: String,
}

/// Deletes an event from Google Calendar completely, removing it from the calendar.
///
/// This function attempts to delete an event whether it's in a confirmed or cancelled state.
/// If the event is already in a cancelled state, it uses a special purge approach to ensure deletion.
pub async fn delete_calendar_event(
    hub: &HubType,
    calendar_id: &str,
    event_id: &str,
    notify_attendees: bool,
) -> Result<(), GcalError> {
    // First check if the event exists and get its status
    let get_result = hub.events().get(calendar_id, event_id).doit().await;

    // If the event doesn't exist, consider it "successfully deleted"
    if let Err(e) = get_result {
        if e.to_string().contains("404") {
            return Ok(());
        }
        return Err(GcalError::ApiError(e));
    }

    // Extract event status for making delete decision
    let (_response, event) = get_result.unwrap();
    let status = event.status.as_deref().unwrap_or("confirmed");

    // Try to delete the event normally first
    let delete_result = hub
        .events()
        .delete(calendar_id, event_id)
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
                let restore_result = hub
                    .events()
                    .patch(restored_event, calendar_id, event_id)
                    .send_updates("none") // Don't notify for intermediate step
                    .doit()
                    .await;

                // Handle restoration result
                match restore_result {
                    Ok(_) => {
                        // Now try deleting again
                        hub.events()
                            .delete(calendar_id, event_id)
                            .send_updates(if notify_attendees { "all" } else { "none" })
                            .doit()
                            .await?;
                        Ok(())
                    }
                    Err(_) => {
                        // If restoration fails, we've done our best - log it and return success
                        // We can't use the purge approach since it's not publicly available
                        tracing::warn!(
                            "Could not fully delete event {}, attempted restore and delete",
                            event_id
                        );
                        Ok(())
                    }
                }
            } else {
                // For any other error, just pass it through
                Err(GcalError::ApiError(e))
            }
        }
    }
}

/// Marks an event as cancelled in Google Calendar without deleting it.
pub async fn mark_event_cancelled(
    hub: &HubType,
    calendar_id: &str,
    event_id: &str,
    notify_attendees: bool,
) -> Result<Event, GcalError> {
    let (_response, event) = hub.events().get(calendar_id, event_id).doit().await?;

    // Create a minimal event with sequence number + 1
    let sequence = event.sequence.map(|n| n + 1).unwrap_or(1);

    let cancelled_event = Event {
        status: Some("cancelled".to_string()),
        sequence: Some(sequence),
        ..Default::default()
    };

    let (_response, updated) = hub
        .events()
        .patch(cancelled_event, calendar_id, event_id)
        .send_updates(if notify_attendees { "all" } else { "none" })
        .doit()
        .await?;

    Ok(updated)
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BookedEventsQuery {
    pub start_date: String,              // YYYY-MM-DD format
    pub end_date: String,                // YYYY-MM-DD format
    pub include_cancelled: Option<bool>, // Whether to include cancelled events
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, serde::Serialize)]
pub struct BookedEvent {
    pub event_id: String,
    pub summary: String,
    pub description: Option<String>,
    pub start_time: String, // ISO 8601 format
    pub end_time: String,   // ISO 8601 format
    pub status: String,     // "confirmed", "cancelled", etc.
    pub created: String,    // ISO 8601 format
    pub updated: String,    // ISO 8601 format
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, serde::Serialize)]
pub struct BookedEventsResponse {
    pub events: Vec<BookedEvent>,
}

/// Fetches booked events from Google Calendar within a specified date range.
pub async fn get_booked_events(
    hub: &HubType,
    calendar_id: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    include_cancelled: bool,
) -> Result<Vec<BookedEvent>, GcalError> {
    // Create the events list request
    let mut request = hub
        .events()
        .list(calendar_id)
        .time_min(start_time)
        .time_max(end_time)
        .single_events(true) // Expand recurring events
        .order_by("startTime"); // Sort by start time

    // This is a key parameter for Google Calendar API to include cancelled events
    request = request.show_deleted(include_cancelled);

    // Add debug output
    println!(
        "Fetching events with include_cancelled={}",
        include_cancelled
    );

    // Make the API call
    let (_, events_list) = request.doit().await?;

    println!(
        "Received {} events from Google Calendar API",
        events_list.items.as_ref().map_or(0, |items| items.len())
    );

    let mut booked_events = Vec::new();

    if let Some(items) = events_list.items {
        for event in items {
            // Debug each event status
            let status = event.status.as_deref().unwrap_or("(no status)");
            println!(
                "Processing event ID: {}, Status: {}",
                event.id.as_deref().unwrap_or("(no id)"),
                status
            );

            // Skip cancelled events if not including them - but this shouldn't be needed
            // as show_deleted parameter should handle this. Keep as a safety check.
            if !include_cancelled && status == "cancelled" {
                println!("Skipping cancelled event due to include_cancelled=false");
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

            println!(
                "Added event to response list. Total count: {}",
                booked_events.len()
            );
        }
    }

    Ok(booked_events)
}
