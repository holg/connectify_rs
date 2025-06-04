// File: crates/connectify_gcal/src/handlers.rs
// use google_calendar3::api::Event;
use crate::logic::{
    calculate_available_slots, create_calendar_event, delete_calendar_event, get_booked_events,
    mark_event_cancelled, AvailabilityQuery, AvailableSlotsResponse, BookSlotRequest,
    BookedEventsQuery, BookedEventsResponse, BookingResponse, CancelBookingRequest,
    CancellationResponse, GcalError, PricedSlot,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use connectify_config::{AppConfig, PriceTier}; // Use the unified config
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

use crate::auth::HubType; // Import the Hub type alias

// Define shared state needed by GCal handlers
#[derive(Clone)]
pub struct GcalState {
    pub config: Arc<AppConfig>,
    pub calendar_hub: Arc<HubType>, // Share the authenticated Calendar client
}

/// Handler to get available time slots.
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/gcal/availability", // Path relative to /api
    params(AvailabilityQuery),
    responses(
        (status = 200, description = "Available time slots with pricing", body = AvailableSlotsResponse),
        (status = 400, description = "Bad request (e.g., invalid date format, no matching price tier)"),
        (status = 500, description = "Internal error")
    ),
    tag = "GCal"
))]
pub async fn get_availability_handler(
    State(state): State<Arc<GcalState>>,
    Query(query): Query<AvailabilityQuery>,
) -> Result<Json<AvailableSlotsResponse>, (StatusCode, String)> {
    // Ensure GCal feature is enabled via runtime config
    if !state.config.use_gcal {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "GCal service is disabled.".to_string(),
        ));
    }

    let gcal_config = state.config.gcal.as_ref().ok_or_else(|| {
        info!("GCal configuration missing in AppConfig.");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Server configuration error: GCal config missing.".to_string(),
        )
    })?;
    let calendar_id = gcal_config.calendar_id.as_ref().ok_or_else(|| {
        info!("GCal calendar_id missing in GcalConfig.");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Server configuration error: GCal calendar ID missing.".to_string(),
        )
    })?;

    // --- Find Price Tier ---
    // Price tiers are assumed to be in StripeConfig for now.
    // This could be moved to a more generic "ServicePricingConfig" if needed.
    let stripe_config = state.config.stripe.as_ref().ok_or_else(|| {
        info!("Stripe configuration (for price tiers) missing in AppConfig.");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Pricing configuration error on server.".to_string(),
        )
    })?;

    let price_tier: &PriceTier = stripe_config
        .price_tiers
        .iter()
        .find(|tier| tier.duration_minutes == query.duration_minutes)
        .ok_or_else(|| {
            let err_msg = format!(
                "No service offered for {} minute duration.",
                query.duration_minutes
            );
            info!("{}", err_msg);
            (StatusCode::BAD_REQUEST, err_msg)
        })?;

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
    info!(
        "Parsed dates: start={}, end={}",
        start_naive_date, end_naive_date
    );
    // Compute Tz boundaries for the query range
    let start_naive_datetime = start_naive_date.and_hms_opt(0, 0, 0).unwrap();
    // Set end time to end of day (23:59:59) to ensure the time range is not empty
    let end_naive_datetime = end_naive_date.and_hms_opt(23, 59, 59).unwrap();
    let time_zone = &gcal_config
        .time_zone
        .clone()
        .unwrap_or("Zurich".to_string());
    let time_zone = Tz::from_str(time_zone).unwrap_or(Tz::Europe__Zurich);
    // Get current time in the configured timezone
    let now_tz = Utc::now().with_timezone(&time_zone);
    let query_start_tz = time_zone
        .from_local_datetime(&start_naive_datetime)
        .unwrap();
    let query_end_tz = time_zone.from_local_datetime(&end_naive_datetime).unwrap();

    let preparation_time = gcal_config.preparation_time_minutes.unwrap_or(120);
    // don’t allow slots before now + prep time
    let effective_start_tz = {
        // now in TZ already computed as `now_tz`
        let min_start = now_tz + Duration::minutes(preparation_time);
        if query_start_tz < min_start {
            min_start
        } else {
            query_start_tz
        }
    };
    let appointment_duration_chrono = Duration::minutes(query.duration_minutes);
    if appointment_duration_chrono <= Duration::zero() {
        return Err((
            StatusCode::BAD_REQUEST,
            "duration_minutes must be positive".to_string(),
        ));
    }

    // --- Fetch Busy Times ---
    let busy_periods = match crate::logic::get_busy_times(
        &state.calendar_hub,
        calendar_id,
        query_start_tz,
        query_end_tz,
    )
    .await
    {
        Ok(periods) => periods,
        Err(e) => {
            info!("Error fetching GCal free/busy: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query calendar availability".to_string(),
            ));
        }
    };

    // --- Calculate Slots using configuration values or defaults ---
    // For tests, use hardcoded values to avoid configuration issues
    #[cfg(test)]
    let work_start = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    #[cfg(test)]
    let work_end = NaiveTime::from_hms_opt(23, 59, 59).unwrap();
    #[cfg(test)]
    let working_days = vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    // For non-test environments, try to use configuration values
    #[cfg(not(test))]
    let work_start = match &gcal_config.work_start_time {
        Some(time_str) => NaiveTime::parse_from_str(time_str, "%H:%M")
            .unwrap_or_else(|_| NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
        None => NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    };

    #[cfg(not(test))]
    let work_end = match &gcal_config.work_end_time {
        Some(time_str) => NaiveTime::parse_from_str(time_str, "%H:%M")
            .unwrap_or_else(|_| NaiveTime::from_hms_opt(23, 59, 59).unwrap()),
        None => NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
    };

    // Parse working days from config
    #[cfg(not(test))]
    let working_days: Vec<Weekday> = match &gcal_config.working_days {
        Some(days) => days
            .iter()
            .filter_map(|day| match day.as_str() {
                "Mon" => Some(Weekday::Mon),
                "Tue" => Some(Weekday::Tue),
                "Wed" => Some(Weekday::Wed),
                "Thu" => Some(Weekday::Thu),
                "Fri" => Some(Weekday::Fri),
                "Sat" => Some(Weekday::Sat),
                "Sun" => Some(Weekday::Sun),
                _ => None,
            })
            .collect(),
        None => vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ],
    };

    let buffer = Duration::minutes(0); // No buffer by default
    let step = Duration::minutes(15); // Check every 15 minutes

    let available_datetime_slots = calculate_available_slots(
        effective_start_tz, // which is the
        query_end_tz,
        &busy_periods,
        appointment_duration_chrono,
        work_start,
        work_end,
        &working_days,
        buffer,
        step,
    )
    // Ensure all returned slots are UTC for consistency
    .into_iter()
    .collect::<Vec<_>>();

    // Convert query start and end to local time for date comparison
    let tz = chrono_tz::Tz::from_str(
        &gcal_config
            .time_zone
            .clone()
            .unwrap_or("Europe/Zurich".to_string()),
    )
    .unwrap();

    // Convert query start and end to local time for date comparison
    let start_local = query_start_tz;
    let end_local = query_end_tz;

    // --- Transform to PricedSlots, filtering and rounding based on local time (zero out seconds/nanos) ---
    let priced_slots: Vec<PricedSlot> = available_datetime_slots
        .iter()
        .filter_map(|slot_start_tz| {
            let slot_local = chrono::DateTime::parse_from_rfc3339(slot_start_tz.0.as_str())
                .ok()
                .map(|dt| dt.with_timezone(&tz))?;

            // Keep the full minute-resolution, only zero out seconds and nanoseconds
            let rounded_local = slot_local.with_second(0)?.with_nanosecond(0)?;

            let local_date = rounded_local.date_naive();
            let start_date = start_local.date_naive();
            let end_date = end_local.date_naive();
            if local_date < start_date || local_date >= end_date {
                return None;
            }

            let floored_tz = rounded_local.with_timezone(&time_zone);
            let slot_end_tz = floored_tz + appointment_duration_chrono;

            tracing::debug!(
                "🕒 Slot interpreted locally as: {} ({:?})",
                slot_local,
                slot_local.weekday()
            );

            Some(PricedSlot {
                start_time: floored_tz.to_rfc3339(),
                end_time: slot_end_tz.to_rfc3339(),
                duration_minutes: query.duration_minutes,
                price: price_tier.unit_amount,
                currency: price_tier.currency.clone().unwrap_or_else(|| {
                    stripe_config
                        .default_currency
                        .clone()
                        .unwrap_or_else(|| "USD".to_string())
                }),
                product_name: price_tier.product_name.clone(),
            })
        })
        .collect();

    Ok(Json(AvailableSlotsResponse {
        slots: priced_slots,
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
    let time_zone = chrono_tz::Tz::from_str(
        gcal_config
            .time_zone
            .clone()
            .unwrap_or("Europe/Zurich".to_string())
            .as_str(),
    )
    .unwrap();
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
        slot_start.with_timezone(&time_zone),
        slot_end.with_timezone(&time_zone),
    )
    .await
    .map_err(|e| {
        info!("Error checking availability: {}", e);
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
            info!("Successfully created event: {:?}", created_event.id);
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
            info!("Error booking slot: {}", e);
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
            info!("Error deleting event: {}", e);
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
            info!("Error marking event as cancelled: {}", e);
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

/// Handler for OPTIONS requests to support CORS preflight
pub async fn options_handler() -> impl axum::response::IntoResponse {
    // Return appropriate CORS headers for preflight requests
    (
        StatusCode::OK,
        [
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Methods", "GET, OPTIONS"),
            ("Access-Control-Allow-Headers", "Content-Type"),
            ("Access-Control-Max-Age", "86400"),
        ],
    )
}

/// Handler to get booked time slots.
#[axum::debug_handler]
pub async fn get_booked_events_handler(
    State(state): State<Arc<GcalState>>,
    Query(query): Query<BookedEventsQuery>,
) -> Result<Json<BookedEventsResponse>, (StatusCode, String)> {
    // Get GCal specific config
    let gcal_config = state.config.gcal.as_ref().expect("GCal config missing");
    let tz = chrono_tz::Tz::from_str(
        gcal_config
            .time_zone
            .as_ref()
            .map_or("Europe/Zurich", |v| v),
    )
    .unwrap();
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

    let query_start_tz = Utc
        .from_utc_datetime(&start_naive_datetime)
        .with_timezone(&tz);
    let query_end_tz = Utc
        .from_utc_datetime(&end_naive_datetime)
        .with_timezone(&tz);

    // Get include_cancelled parameter, default to false if not provided
    let include_cancelled = query.include_cancelled.unwrap_or(false);

    // Fetch booked events
    match get_booked_events(
        &state.calendar_hub,
        gcal_config
            .calendar_id
            .as_ref()
            .expect("Calendar ID is required"),
        query_start_tz,
        query_end_tz,
        include_cancelled,
    )
    .await
    {
        Ok(events) => Ok(Json(BookedEventsResponse { events })),
        Err(e) => {
            info!("Error fetching booked events: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch booked events".to_string(),
            ))
        }
    }
}
