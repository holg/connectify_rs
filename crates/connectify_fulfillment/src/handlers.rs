// --- File: crates/connectify_fulfillment/src/handlers.rs ---

use axum::{
    extract::{State},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use std::sync::Arc;
use connectify_config::AppConfig; // To access shared configuration
use serde::Deserialize; // For request body deserialization

// Import logic functions and request/response types
use crate::logic::{
    FulfillmentResponse, FulfillmentError,
    // GCal specific, conditionally imported
};
#[cfg(feature = "gcal")]
use crate::logic::{
    GcalBookingFulfillmentRequest,
    fulfill_gcal_booking_logic,
    AdhocGcalTwilioFulfillmentRequest,
    fulfill_adhoc_gcal_twilio_logic,
};

// Conditionally import GcalState if the 'gcal' feature is enabled for this crate
#[cfg(feature = "gcal")]
use connectify_gcal::handlers::GcalState as ConnectifyGcalState; // Alias to avoid name clash if any

// --- State for Fulfillment Handlers ---
#[derive(Clone)] // Added Debug for logging in routes.rs
pub struct FulfillmentState {
    pub config: Arc<AppConfig>,
    // If fulfillment logic directly calls GCal logic that requires GcalState (e.g., a pre-initialized Hub)
    #[cfg(feature = "gcal")]
    pub gcal_state_for_fulfillment: Option<Arc<ConnectifyGcalState>>, // Renamed for clarity
}

// --- Handler for Standard GCal Booking Fulfillment ---
#[cfg(feature = "gcal")]
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/fulfill/gcal-booking",
    request_body = GcalBookingFulfillmentRequest,
    responses(
        (status = 200, description = "Booking fulfilled successfully", body = FulfillmentResponse),
        (status = 400, description = "Bad Request - Invalid payload"),
        (status = 401, description = "Unauthorized - Missing or invalid auth"),
        (status = 409, description = "Booking conflict in Google Calendar"),
        (status = 500, description = "Internal Server Error - Fulfillment failed")
    ),
    tag = "Fulfillment"
))]
pub async fn handle_gcal_booking_fulfillment(
    State(state): State<Arc<FulfillmentState>>,
    Json(payload): Json<GcalBookingFulfillmentRequest>,
) -> Result<Json<FulfillmentResponse>, (StatusCode, String)> {

    println!("[Fulfillment Handler] Received GCal booking fulfillment request: {:?}", payload.summary);

    // TODO: Implement Authentication for this internal endpoint.
    // Check shared secret from state.config.fulfillment.shared_secret

    // Check if GCal is enabled in the main app config
    if !state.config.use_gcal {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "GCal feature is not enabled in config.".to_string()));
    }

    let gcal_config = state.config.gcal.as_ref().ok_or_else(|| {
        eprintln!("[Fulfillment Handler] GCal configuration missing in AppConfig for fulfillment.");
        (StatusCode::INTERNAL_SERVER_ERROR, "Server configuration error for GCal".to_string())
    })?;

    // If you decide to pass the GcalState (and thus the pre-initialized Hub) to the logic function:
    // let gcal_hub_to_use = state.gcal_state_for_fulfillment.as_ref()
    //     .ok_or_else(|| {
    //         eprintln!("[Fulfillment Handler] GcalState (and Hub) not available for fulfillment.");
    //         (StatusCode::INTERNAL_SERVER_ERROR, "GCal client not initialized for fulfillment.".to_string())
    //     })?.calendar_hub.clone(); // Clone the Arc<HubType>

    // For now, fulfill_gcal_booking_logic creates its own hub from gcal_config
    match fulfill_gcal_booking_logic(gcal_config, payload).await {
        Ok(response) => {
            println!("[Fulfillment Handler] GCal booking fulfillment successful: {:?}", response.event_id);
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("[Fulfillment Handler] GCal booking fulfillment failed: {}", e);
            match e {
                FulfillmentError::GcalApiError(api_err_msg) => Err((StatusCode::BAD_GATEWAY, format!("Google Calendar API error: {}", api_err_msg))),
                FulfillmentError::GcalBookingConflict => Err((StatusCode::CONFLICT, "Booking conflict in Google Calendar.".to_string())),
                FulfillmentError::ConfigError(msg) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Configuration error: {}", msg))),
                FulfillmentError::FeatureDisabled(msg) => Err((StatusCode::SERVICE_UNAVAILABLE, format!("Required feature for fulfillment disabled: {}", msg))),
                FulfillmentError::InternalError(msg) => Err((StatusCode::INTERNAL_SERVER_ERROR, msg)),
            }
        }
    }
}

// --- NEW: Handler for Adhoc GCal & Twilio Fulfillment ---
#[cfg(feature = "gcal")] // This handler also depends on the 'gcal' feature of this crate
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/fulfill/adhoc-gcal-twilio",
    request_body = AdhocGcalTwilioFulfillmentRequest,
    responses(
        (status = 200, description = "Adhoc session fulfilled (GCal booked)", body = FulfillmentResponse),
        (status = 400, description = "Bad Request - Invalid payload"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Booking conflict in Google Calendar"),
        (status = 500, description = "Internal Server Error - Fulfillment failed")
    ),
    tag = "Fulfillment"
))]
pub async fn handle_adhoc_gcal_twilio_fulfillment(
    State(state): State<Arc<FulfillmentState>>,
    Json(payload): Json<AdhocGcalTwilioFulfillmentRequest>,
) -> Result<Json<FulfillmentResponse>, (StatusCode, String)> {

    println!("[Fulfillment Handler] Received Adhoc GCal/Twilio fulfillment request for room: {}", payload.room_name);

    // Check if GCal is enabled in the main app config (needed for booking the slot)
    if !state.config.use_gcal {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "GCal feature is not enabled in config, cannot fulfill adhoc GCal booking.".to_string()));
    }
    // Also check if the adhoc feature itself is enabled at runtime
    if !state.config.use_adhoc {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Adhoc sessions feature is not enabled in config.".to_string()));
    }


    let gcal_config = state.config.gcal.as_ref().ok_or_else(|| {
        eprintln!("[Fulfillment Handler] GCal configuration missing for adhoc fulfillment.");
        (StatusCode::INTERNAL_SERVER_ERROR, "Server configuration error for GCal (adhoc)".to_string())
    })?;

    // Call the adhoc fulfillment logic
    match fulfill_adhoc_gcal_twilio_logic(gcal_config, payload).await {
        Ok(response) => {
            println!("[Fulfillment Handler] Adhoc GCal booking successful: Event ID {:?}, Room: {:?}", response.event_id, response.room_name);
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("[Fulfillment Handler] Adhoc GCal booking failed: {}", e);
            match e {
                FulfillmentError::GcalApiError(api_err_msg) => Err((StatusCode::BAD_GATEWAY, format!("Google Calendar API error: {}", api_err_msg))),
                FulfillmentError::GcalBookingConflict => Err((StatusCode::CONFLICT, "Booking conflict in Google Calendar for adhoc session.".to_string())),
                FulfillmentError::ConfigError(msg) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Configuration error: {}", msg))),
                FulfillmentError::FeatureDisabled(msg) => Err((StatusCode::SERVICE_UNAVAILABLE, format!("Required feature for fulfillment disabled: {}", msg))),
                FulfillmentError::InternalError(msg) => Err((StatusCode::INTERNAL_SERVER_ERROR, msg)),
            }
        }
    }
}
