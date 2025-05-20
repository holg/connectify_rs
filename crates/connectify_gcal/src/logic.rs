// --- File: crates/connectify_gcal/src/logic.rs ---
use crate::auth::HubType; // Use the specific Hub type alias
use crate::service::{GcalServiceError, GoogleCalendarService};
use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc, Weekday}; // Use chrono Duration
use connectify_common::services::{CalendarEvent as CommonCalendarEvent, CalendarService};
use google_calendar3::api::Event; //, EventDateTime};
use serde::{Deserialize, Serialize};
use std::sync::Arc; //, CalendarEventResult, , BookedEvent as CommonBookedEvent};

// To access GCal config
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
    #[error("No matching price tier found for duration: {0} minutes")] // Added for pricing
    NoMatchingPriceTier(i64),
    #[error("Calendar service error: {0}")]
    ServiceError(#[from] GcalServiceError),
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
    #[cfg_attr(feature = "openapi", schema(example = 45))]
    pub duration_minutes: i64,
}

#[derive(Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AvailableSlotsResponse {
    pub slots: Vec<PricedSlot>,
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PricedSlot {
    #[cfg_attr(feature = "openapi", schema(example = "2025-05-15T10:00:00Z"))]
    pub start_time: String, // ISO 8601 format
    #[cfg_attr(feature = "openapi", schema(example = "2025-05-15T11:00:00Z"))]
    pub end_time: String, // ISO 8601 format
    #[cfg_attr(feature = "openapi", schema(example = 60))]
    pub duration_minutes: i64,
    #[cfg_attr(feature = "openapi", schema(example = 7500))] // e.g. 75.00 CHF in cents
    pub price: i64,
    #[cfg_attr(feature = "openapi", schema(example = "CHF"))]
    pub currency: String,
    #[cfg_attr(feature = "openapi", schema(example = "Premium Beratung (60 Min)"))]
    pub product_name: Option<String>,
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
    // Create a GoogleCalendarService instance
    let service = GoogleCalendarService::new(Arc::new(hub.clone()));

    // Use the service to get busy times
    let busy_periods = service
        .get_busy_times(calendar_id, start_time, end_time)
        .await?;

    Ok(busy_periods)
}

/// Configuration for working hours and days
pub struct WorkingHoursConfig<'a> {
    /// Start time of the working day (e.g., 9:00 AM)
    pub start_time: NaiveTime,
    /// End time of the working day (e.g., 5:00 PM)
    pub end_time: NaiveTime,
    /// Days of the week when appointments can be scheduled
    pub working_days: &'a [Weekday],
}

/// Configuration for appointment scheduling
pub struct AppointmentConfig {
    /// Duration of each appointment
    pub duration: Duration,
    /// Buffer time between appointments
    pub buffer_time: Duration,
    /// Time step for checking available slots
    pub step: Duration,
}

/// Calculates available slots based on busy times, working hours, etc.
/// THIS IS A COMPLEX SKELETON - NEEDS DETAILED IMPLEMENTATION
#[allow(clippy::too_many_arguments)]
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
            current_check_time += step; // Move to next step
            continue;
        }

        // --- Check 2: Is it within working days/hours? ---
        let day_of_week = potential_start_time.weekday();
        let time_of_day = potential_start_time.time();
        let end_time_of_day = potential_end_time.time(); // Need to check end time too

        // Debug print to help diagnose issues
        // info!("Checking slot: {:?}, day: {:?}, time: {:?}, end_time: {:?}",
        //          potential_start_time, day_of_week, time_of_day, end_time_of_day);

        if !working_days.contains(&day_of_week) ||
            time_of_day < work_start_time ||
            time_of_day > work_end_time || // Check start time is before work_end_time
            end_time_of_day > work_end_time ||
            // Handle edge case where appointment crosses midnight (if allowed)
            (potential_end_time.date_naive() != potential_start_time.date_naive() && end_time_of_day > NaiveTime::from_hms_opt(0,0,0).unwrap())
        {
            // Advance check time smartly (e.g., to start of next working day/hour)
            // For simplicity, just step forward for now
            current_check_time += step;
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
    // Create a GoogleCalendarService instance
    let service = GoogleCalendarService::new(Arc::new(hub.clone()));

    // Convert BookSlotRequest to CalendarEvent
    let calendar_event = CommonCalendarEvent {
        start_time: request.start_time.clone(),
        end_time: request.end_time.clone(),
        summary: request.summary.clone(),
        description: request.description.clone(),
    };

    // Use the service to create the event
    let result = service.create_event(calendar_id, calendar_event).await?;

    // Construct a minimal Event object with the event ID and status
    let created_event = Event {
        id: result.event_id.clone(),
        status: Some(result.status.clone()),
        ..Default::default()
    };

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
    // Create a GoogleCalendarService instance
    let service = GoogleCalendarService::new(Arc::new(hub.clone()));

    // Use the service to delete the event
    service
        .delete_event(calendar_id, event_id, notify_attendees)
        .await?;

    Ok(())
}

/// Marks an event as cancelled in Google Calendar without deleting it.
pub async fn mark_event_cancelled(
    hub: &HubType,
    calendar_id: &str,
    event_id: &str,
    notify_attendees: bool,
) -> Result<Event, GcalError> {
    // Create a GoogleCalendarService instance
    let service = GoogleCalendarService::new(Arc::new(hub.clone()));

    // Use the service to mark the event as cancelled
    let result = service
        .mark_event_cancelled(calendar_id, event_id, notify_attendees)
        .await?;

    // Construct a minimal Event object with the event ID and status
    let updated_event = Event {
        id: result.event_id.clone(),
        status: Some(result.status.clone()),
        ..Default::default()
    };

    Ok(updated_event)
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
    // Create a GoogleCalendarService instance
    let service = GoogleCalendarService::new(Arc::new(hub.clone()));

    // Use the service to get booked events
    let events = service
        .get_booked_events(calendar_id, start_time, end_time, include_cancelled)
        .await?;

    // Convert the events to the format expected by the handlers
    let booked_events = events
        .into_iter()
        .map(|event| BookedEvent {
            event_id: event.event_id,
            summary: event.summary,
            description: event.description,
            start_time: event.start_time,
            end_time: event.end_time,
            status: event.status,
            created: event.created,
            updated: event.updated,
        })
        .collect();

    Ok(booked_events)
}
