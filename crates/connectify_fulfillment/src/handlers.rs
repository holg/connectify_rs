// --- File: crates/connectify_fulfillment/src/handlers.rs ---
use axum::{
    extract::{State},
    response::{Json},
    http::StatusCode,
};
use std::sync::Arc;
use connectify_config::AppConfig; // To access shared configuration
// use serde::Deserialize; // For request body deserialization

// Import logic functions and request/response types (we'll define these in logic.rs next)
use crate::logic::{
    GcalBookingFulfillmentRequest, FulfillmentResponse, FulfillmentError
};
#[cfg(feature = "gcal")]
pub(crate) use connectify_gcal::handlers::GcalState;
#[cfg(feature = "gcal")]
use crate::logic::fulfill_gcal_booking_logic;
// --- State for Fulfillment Handlers ---
// This state will be created in the fulfillment crate's routes.rs
// and will be provided when merging the fulfillment router into the main app.
#[derive(Clone)]
pub struct FulfillmentState {
    pub config: Arc<AppConfig>,
    // If fulfillment logic directly calls GCal or Twilio logic that requires their specific states
    // (like a CalendarHub or Twilio client), those Arcs would be included here.
    // For now, let's assume logic functions might take AppConfig and initialize clients as needed,
    // or that the calling service (e.g., Stripe webhook) provides all necessary data.
    // Example:
    #[cfg(feature = "gcal")]
    pub gcal_state: Option<Arc<GcalState>>, // If needed

}

// --- Handler for GCal Booking Fulfillment ---

/// Axum handler to fulfill a Google Calendar booking.
/// This endpoint is expected to be called internally (e.g., by another backend service
/// like a webhook handler after a payment is confirmed).
/// It should be protected by an authentication mechanism (e.g., shared secret).
#[cfg(feature = "gcal")]
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/fulfill/gcal-booking", // Path relative to where this router is nested
    request_body = GcalBookingFulfillmentRequest,
    responses(
        (status = 200, description = "Booking fulfilled successfully", body = FulfillmentResponse),
        (status = 400, description = "Bad Request - Invalid payload"),
        (status = 401, description = "Unauthorized - Missing or invalid auth"),
        (status = 500, description = "Internal Server Error - Fulfillment failed")
    ),
    tag = "Fulfillment"
))]
pub async fn handle_gcal_booking_fulfillment(
    State(state): State<Arc<FulfillmentState>>,
    Json(payload): Json<GcalBookingFulfillmentRequest>, // Request body with booking details
) -> Result<Json<FulfillmentResponse>, (StatusCode, String)> {

    println!("Received GCal booking fulfillment request: {:?}", payload);

    // TODO: Implement Authentication for this internal endpoint.
    // For example, check a shared secret passed in a header:
    // let auth_header = headers.get("X-Internal-Auth-Secret").and_then(|v| v.to_str().ok());
    // let expected_secret = &state.config.fulfillment.as_ref().expect("Fulfillment config missing").shared_secret;
    // if auth_header != Some(expected_secret) {
    //     return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    // }

    // Access GCal specific config if needed by the logic function
    let gcal_config = state.config.gcal.as_ref().ok_or_else(|| {
        eprintln!("GCal configuration missing in AppConfig for fulfillment.");
        (StatusCode::INTERNAL_SERVER_ERROR, "Server configuration error for GCal".to_string())
    })?;

    // Call the core logic function
    match fulfill_gcal_booking_logic(gcal_config, payload /*, &state.gcal_hub_if_any */).await {
        Ok(response) => {
            println!("GCal booking fulfillment successful: {:?}", response.event_id);
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("GCal booking fulfillment failed: {}", e);
            // Map FulfillmentError to an HTTP error response
            match e {
                FulfillmentError::GcalApiError(api_err_msg) => {
                    Err((StatusCode::BAD_GATEWAY, format!("Google Calendar API error: {}", api_err_msg)))
                }
                FulfillmentError::GcalBookingConflict => {
                    Err((StatusCode::CONFLICT, "Booking conflict in Google Calendar.".to_string()))
                }
                FulfillmentError::InternalError(msg) => {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, msg))
                }
                FulfillmentError::ConfigError(msg) => {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, "Missing e.g. GCal configuration.".to_string()))
                }
            }
        }
    }
}

// TODO: Add handlers for other fulfillment tasks, e.g.:
// pub async fn handle_twilio_adhoc_fulfillment(...) -> ... { ... }

