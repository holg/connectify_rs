// --- File: crates/connectify_gcal/src/logic.rs ---
use crate::auth::HubType; // Use the specific Hub type alias
use crate::service::{GcalServiceError, GoogleCalendarService};
use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc, Weekday}; // Use chrono Duration
use chrono_tz::Europe::Zurich;
use connectify_common::services::{CalendarEvent as CommonCalendarEvent, CalendarService};
use google_calendar3::api::Event; //, EventDateTime};
use serde::{Deserialize, Serialize};
use std::sync::Arc; //, CalendarEventResult, , BookedEvent as CommonBookedEvent};
use tracing::{debug, error};

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
    pub payment_method: Option<String>,
    pub payment_id: Option<String>,
    pub payment_amount: Option<i64>,
    pub room_name: Option<String>,
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
/// Returns slots as pairs of RFC3339 strings in Europe/Zurich time zone.
#[allow(clippy::too_many_arguments)]
pub fn calculate_available_slots(
    query_start: DateTime<Utc>,
    query_end: DateTime<Utc>,
    busy_periods: &[(DateTime<Utc>, DateTime<Utc>)],
    duration: Duration,
    work_start_time: NaiveTime,
    work_end_time: NaiveTime,
    working_days: &[Weekday],
    buffer_time: Duration,
    step: Duration,
) -> Vec<(String, String)> {
    use chrono::{TimeZone, Timelike};

    fn merge_busy_periods(
        busy: &[(DateTime<Utc>, DateTime<Utc>)],
    ) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
        if busy.is_empty() {
            return vec![];
        }
        let mut sorted = busy.to_vec();
        sorted.sort_by_key(|(start, _)| *start);
        let mut merged = vec![sorted[0]];
        for &(start, end) in &sorted[1..] {
            let last = merged.last_mut().unwrap();
            if start <= last.1 {
                last.1 = last.1.max(end);
            } else {
                merged.push((start, end));
            }
        }
        merged
    }

    fn advance_to_next_working_time(
        current: DateTime<Utc>,
        work_start: NaiveTime,
        working_days: &[Weekday],
    ) -> DateTime<Utc> {
        let mut local = current.with_timezone(&Zurich);
        loop {
            let weekday = local.date_naive().weekday();
            if working_days.contains(&weekday) && local.time() <= work_start {
                return Utc
                    .with_ymd_and_hms(
                        local.year(),
                        local.month(),
                        local.day(),
                        work_start.hour(),
                        work_start.minute(),
                        0,
                    )
                    .unwrap();
            }
            local += chrono::Duration::days(1);
            local = Zurich
                .with_ymd_and_hms(
                    local.year(),
                    local.month(),
                    local.day(),
                    work_start.hour(),
                    work_start.minute(),
                    0,
                )
                .unwrap();
        }
    }

    let now_utc = Utc::now();
    // Only allow slots in the future, keep UTC type
    let adjusted_query_start = query_start.max(now_utc);

    debug!(
        "Calculating available slots for {} - {} Working Days:{:?}",
        query_start, query_end, working_days
    );
    let mut available_slots = Vec::new();
    let mut current_check_time = adjusted_query_start;
    // Round up to next step interval (e.g., next 15min)
    {
        let local_now = current_check_time.with_timezone(&Zurich);
        let step_minutes = step.num_minutes() as u32;
        let minute = local_now.minute();
        let rounded = minute % step_minutes;
        if rounded != 0 {
            let padding = step - chrono::Duration::minutes(rounded.into());
            current_check_time = (local_now + padding).with_timezone(&Utc);
        }
    }

    // Merge overlapping/adjacent busy periods for efficiency
    let merged_busy = merge_busy_periods(busy_periods);

    while current_check_time < query_end {
        let potential_start_time = current_check_time;
        let potential_end_time = match potential_start_time.checked_add_signed(duration) {
            Some(t) => t,
            None => break,
        };
        let potential_end_with_buffer = match potential_end_time.checked_add_signed(buffer_time) {
            Some(t) => t,
            None => potential_end_time,
        };
        let local_start = potential_start_time.with_timezone(&Zurich);
        let local_end = potential_end_time.with_timezone(&Zurich);

        if potential_start_time < query_start || potential_end_time > query_end {
            current_check_time += step;
            continue;
        }
        let day_of_week = local_start.date_naive().weekday();
        let time_of_day = local_start.time();
        let end_time_of_day = local_end.time();

        // If not in working day or outside working hours, advance smartly
        if !working_days.contains(&day_of_week)
            || time_of_day < work_start_time
            || time_of_day > work_end_time
            || end_time_of_day > work_end_time
            || (local_end.date_naive() != local_start.date_naive()
                && end_time_of_day > NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        {
            current_check_time = advance_to_next_working_time(
                current_check_time + step,
                work_start_time,
                working_days,
            );
            continue;
        }

        // Truncate if end-of-day is too close for slot
        let remaining_day = work_end_time - time_of_day;
        if remaining_day < duration {
            current_check_time = advance_to_next_working_time(
                current_check_time + chrono::Duration::days(1),
                work_start_time,
                working_days,
            );
            continue;
        }

        // Check for overlap with merged busy periods
        let mut overlaps = false;
        for (busy_start, busy_end) in &merged_busy {
            if potential_start_time < *busy_end && potential_end_with_buffer > *busy_start {
                overlaps = true;
                current_check_time = (*busy_end + buffer_time).max(current_check_time + step);
                break;
            }
        }

        if !overlaps {
            // Convert to Europe/Zurich local time and RFC3339 string
            let start_zurich = potential_start_time.with_timezone(&Zurich).to_rfc3339();
            let end_zurich = potential_end_time.with_timezone(&Zurich).to_rfc3339();
            available_slots.push((start_zurich, end_zurich));
            current_check_time = potential_end_with_buffer;
        }
    }
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
        // TODO move these values into Description / Summary fields as needed, for now they are skipped on serde serialize
        payment_method: request.payment_method.clone(),
        payment_id: request.payment_id.clone(),
        payment_amount: request.payment_amount,
        room_name: request.room_name.clone(),
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
    pub payment_method: Option<String>,
    pub payment_id: Option<String>,
    pub payment_amount: Option<i64>,
    pub room_name: Option<String>,
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
            payment_method: event.payment_method,
            payment_id: event.payment_id,
            payment_amount: event.payment_amount,
            room_name: event.room_name,
        })
        .collect();

    Ok(booked_events)
}
