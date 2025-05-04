// File: crates/connectify_gcal/src/handlers.rs
use crate::logic::get_booked_events;
use crate::logic::BookedEventsQuery;
use crate::logic::BookedEventsResponse;
// use google_calendar3::api::Event;
use crate::logic::{
    calculate_available_slots, create_calendar_event, delete_calendar_event, mark_event_cancelled,
    AvailabilityQuery, AvailableSlotsResponse, BookSlotRequest, BookingResponse,
    CancelBookingRequest, CancellationResponse, GcalError,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use connectify_config::AppConfig; // Use the unified config
use std::sync::Arc;

use crate::auth::HubType; // Import the Hub type alias

// Define shared state needed by GCal handlers
#[derive(Clone)]
pub struct GcalState {
    pub config: Arc<AppConfig>,
    pub calendar_hub: Arc<HubType>, // Share the authenticated Calendar client
}

/// Handler to get available time slots.
#[axum::debug_handler]
pub async fn get_availability_handler(
    State(state): State<Arc<GcalState>>, // Extract shared GCal state
    Query(query): Query<AvailabilityQuery>, // Extract query params
) -> Result<Json<AvailableSlotsResponse>, (StatusCode, String)> {
    // Get GCal specific config (safe unwrap assuming route guard checks this)
    let gcal_config = state.config.gcal.as_ref().expect("GCal config missing");

    // --- Parse Dates & Validate ---
    let start_naive_date =
        NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d").map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid start_date format (YYYY-MM-DD)".to_string(),
            )
        })?;
    let end_naive_date = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid end_date format (YYYY-MM-DD)".to_string(),
        )
    })?;

    if end_naive_date < start_naive_date {
        return Err((
            StatusCode::BAD_REQUEST,
            "end_date must be after start_date".to_string(),
        ));
    }
    // Add 1 day to end_date to include the full end day in the range
    let end_naive_date_inclusive = end_naive_date + Duration::days(1);

    // First create NaiveDateTime objects from your NaiveDate objects
    let start_naive_datetime = start_naive_date.and_hms_opt(0, 0, 0).unwrap();
    let end_naive_datetime = end_naive_date_inclusive.and_hms_opt(0, 0, 0).unwrap();

    // Then use from_utc_datetime
    let query_start_utc = Utc.from_utc_datetime(&start_naive_datetime);
    let query_end_utc = Utc.from_utc_datetime(&end_naive_datetime);

    let appointment_duration = Duration::minutes(query.duration_minutes);
    if appointment_duration <= Duration::zero() {
        return Err((
            StatusCode::BAD_REQUEST,
            "duration_minutes must be positive".to_string(),
        ));
    }

    // --- Fetch Busy Times ---
    let busy_periods = match crate::logic::get_busy_times(
        &state.calendar_hub,
        gcal_config
            .calendar_id
            .as_ref()
            .expect("Calendar ID is required"),
        query_start_utc,
        query_end_utc,
    )
    .await
    {
        Ok(periods) => periods,
        Err(e) => {
            eprintln!("Error fetching free/busy: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query calendar availability".to_string(),
            ));
        }
    };

    // --- Calculate Slots (using placeholder parameters for now) ---
    // TODO: Load these from config or define constants
    let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
    let working_days = [
        chrono::Weekday::Mon,
        chrono::Weekday::Tue,
        chrono::Weekday::Wed,
        chrono::Weekday::Thu,
        chrono::Weekday::Fri,
    ];
    let buffer = Duration::minutes(0); // Example: no buffer
    let step = Duration::minutes(15); // Example: check every 15 mins

    let available_slots_dt = calculate_available_slots(
        query_start_utc,
        query_end_utc,
        &busy_periods,
        appointment_duration,
        work_start,
        work_end,
        &working_days,
        buffer,
        step,
    );

    // --- Format Response ---
    let response_slots = available_slots_dt
        .iter()
        .map(|dt| dt.to_rfc3339()) // Format as ISO 8601 string e.g., "2025-04-25T10:00:00Z"
        .collect();

    Ok(Json(AvailableSlotsResponse {
        slots: response_slots,
    }))
}

/// Handler to book a time slot.
#[axum::debug_handler]
pub async fn book_slot_handler(
    State(state): State<Arc<GcalState>>,  // Extract shared GCal state
    Json(payload): Json<BookSlotRequest>, // Extract JSON body
) -> Result<Json<BookingResponse>, (StatusCode, String)> {
    // Get GCal specific config
    let gcal_config = state.config.gcal.as_ref().expect("GCal config missing");

    // Validate time slot availability
    let slot_start = chrono::DateTime::parse_from_rfc3339(&payload.start_time).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid start_time format".to_string(),
        )
    })?;
    let slot_end = chrono::DateTime::parse_from_rfc3339(&payload.end_time).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid end_time format".to_string(),
        )
    })?;

    // Check current availability
    let busy_periods = crate::logic::get_busy_times(
        &state.calendar_hub,
        gcal_config
            .calendar_id
            .as_ref()
            .expect("Calendar ID is required"),
        slot_start.with_timezone(&Utc),
        slot_end.with_timezone(&Utc),
    )
    .await
    .map_err(|e| {
        eprintln!("Error checking availability: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to check slot availability".to_string(),
        )
    })?;

    // Check if there are any overlapping busy periods
    for busy in &busy_periods {
        if !(busy.1 <= slot_start.with_timezone(&Utc) || busy.0 >= slot_end.with_timezone(&Utc)) {
            return Err((
                StatusCode::CONFLICT,
                "Requested time slot is no longer available".to_string(),
            ));
        }
    }

    // TODO: Add payment validation here once payment system is integrated
    // if let Some(payment_info) = payload.payment_info {
    //     validate_payment(payment_info).await?;
    // }

    match create_calendar_event(
        &state.calendar_hub,
        gcal_config
            .calendar_id
            .as_ref()
            .expect("Calendar ID is required"),
        payload,
    )
    .await
    {
        Ok(created_event) => {
            println!("Successfully created event: {:?}", created_event.id);
            Ok(Json(BookingResponse {
                success: true,
                event_id: created_event.id, // Send back the Google Calendar event ID
                message: "Appointment booked successfully.".to_string(),
            }))
        }
        Err(GcalError::Conflict) => {
            // Example specific error handling
            Err((
                StatusCode::CONFLICT,
                "Requested time slot is no longer available.".to_string(),
            ))
        }
        Err(e) => {
            eprintln!("Error booking slot: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to book appointment.".to_string(),
            ))
        }
    }
}

/// Handler to delete a booking completely from the calendar.
#[axum::debug_handler]
pub async fn delete_event_handler(
    State(state): State<Arc<GcalState>>,
    axum::extract::Path(event_id): axum::extract::Path<String>,
    Query(params): Query<CancelBookingRequest>,
) -> Result<Json<CancellationResponse>, (StatusCode, String)> {
    // Get GCal specific config
    let gcal_config = state.config.gcal.as_ref().expect("GCal config missing");
    let calendar_id = gcal_config
        .calendar_id
        .as_ref()
        .expect("Calendar ID is required");

    // Use notify_attendees parameter if provided, or default to true
    let notify_attendees = params.notify_attendees.unwrap_or(true);

    match delete_calendar_event(
        &state.calendar_hub,
        calendar_id,
        &event_id,
        notify_attendees,
    )
    .await
    {
        Ok(_) => Ok(Json(CancellationResponse {
            success: true,
            message: "Event deleted successfully.".to_string(),
        })),
        Err(e) => {
            eprintln!("Error deleting event: {}", e);
            match e {
                GcalError::ApiError(error) => {
                    // Handle specific error codes if needed
                    if error.to_string().contains("404") {
                        return Err((StatusCode::NOT_FOUND, "Event not found.".to_string()));
                    }
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to delete event.".to_string(),
                    ))
                }
                _ => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to delete event.".to_string(),
                )),
            }
        }
    }
}

/// Handler to mark a booking as cancelled without deleting it.
#[axum::debug_handler]
pub async fn mark_booking_cancelled_handler(
    State(state): State<Arc<GcalState>>,
    axum::extract::Path(event_id): axum::extract::Path<String>,
    Query(params): Query<CancelBookingRequest>,
) -> Result<Json<CancellationResponse>, (StatusCode, String)> {
    // Get GCal specific config
    let gcal_config = state.config.gcal.as_ref().expect("GCal config missing");
    let calendar_id = gcal_config
        .calendar_id
        .as_ref()
        .expect("Calendar ID is required");

    // Use notify_attendees parameter if provided, or default to true
    let notify_attendees = params.notify_attendees.unwrap_or(true);

    match mark_event_cancelled(
        &state.calendar_hub,
        calendar_id,
        &event_id,
        notify_attendees,
    )
    .await
    {
        Ok(_) => Ok(Json(CancellationResponse {
            success: true,
            message: "Appointment marked as cancelled successfully.".to_string(),
        })),
        Err(e) => {
            eprintln!("Error marking event as cancelled: {}", e);
            match e {
                GcalError::ApiError(error) => {
                    // Handle specific error codes if needed
                    if error.to_string().contains("404") {
                        return Err((StatusCode::NOT_FOUND, "Event not found.".to_string()));
                    }
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to mark appointment as cancelled.".to_string(),
                    ))
                }
                _ => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to mark appointment as cancelled.".to_string(),
                )),
            }
        }
    }
}

/// Handler to get booked time slots.
#[axum::debug_handler]
pub async fn get_booked_events_handler(
    State(state): State<Arc<GcalState>>,
    Query(query): Query<BookedEventsQuery>,
) -> Result<Json<BookedEventsResponse>, (StatusCode, String)> {
    // Get GCal specific config
    let gcal_config = state.config.gcal.as_ref().expect("GCal config missing");

    // Parse dates
    let start_naive_date =
        NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d").map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid start_date format (YYYY-MM-DD)".to_string(),
            )
        })?;
    let end_naive_date = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid end_date format (YYYY-MM-DD)".to_string(),
        )
    })?;

    if end_naive_date < start_naive_date {
        return Err((
            StatusCode::BAD_REQUEST,
            "end_date must be after start_date".to_string(),
        ));
    }

    // Add 1 day to end_date to include the full end day
    let end_naive_date_inclusive = end_naive_date + Duration::days(1);

    // Convert to UTC DateTime
    let start_naive_datetime = start_naive_date.and_hms_opt(0, 0, 0).unwrap();
    let end_naive_datetime = end_naive_date_inclusive.and_hms_opt(0, 0, 0).unwrap();

    let query_start_utc = Utc.from_utc_datetime(&start_naive_datetime);
    let query_end_utc = Utc.from_utc_datetime(&end_naive_datetime);

    // Get include_cancelled parameter, default to false if not provided
    let include_cancelled = query.include_cancelled.unwrap_or(false);

    // Fetch booked events
    match get_booked_events(
        &state.calendar_hub,
        gcal_config
            .calendar_id
            .as_ref()
            .expect("Calendar ID is required"),
        query_start_utc,
        query_end_utc,
        include_cancelled,
    )
    .await
    {
        Ok(events) => Ok(Json(BookedEventsResponse { events })),
        Err(e) => {
            eprintln!("Error fetching booked events: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch booked events".to_string(),
            ))
        }
    }
}
