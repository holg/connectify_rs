// --- File: crates/connectify_fulfillment/src/handlers.rs ---

#[cfg(feature = "gcal")]
use axum::{extract::State, http::StatusCode, response::Json};
use connectify_config::AppConfig;
use std::sync::Arc;
#[cfg(feature = "gcal")]
use tracing::{info, warn}; // To access shared configuration
                           // Import logic functions and request/response types
#[cfg(feature = "gcal")]
use crate::logic::{
    fulfill_adhoc_gcal_twilio_logic, fulfill_gcal_booking_logic, AdhocGcalTwilioFulfillmentRequest,
    GcalBookingFulfillmentRequest,
};
#[allow(unused_imports)]
use crate::logic::{
    FulfillmentError,
    // GCal specific, conditionally imported
    FulfillmentResponse,
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
    info!(
        "[Fulfillment Handler] Received GCal booking fulfillment request: {:?}",
        payload.summary
    );

    // Authentication is handled by the fulfillment_auth_middleware in auth.rs

    // Check if GCal is enabled in the main app config
    if !state.config.use_gcal {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "GCal feature is not enabled in config.".to_string(),
        ));
    }
    match fulfill_gcal_booking_logic(State(state), payload).await {
        Ok(response) => {
            info!(
                "[Fulfillment Handler] GCal booking fulfillment successful: {:?}",
                response.event_id
            );
            Ok(Json(response))
        }
        Err(e) => {
            info!(
                "[Fulfillment Handler] GCal booking fulfillment failed: {}",
                e
            );
            match e {
                FulfillmentError::GcalApiError(api_err_msg) => Err((
                    StatusCode::BAD_GATEWAY,
                    format!("Google Calendar API error: {}", api_err_msg),
                )),
                FulfillmentError::GcalBookingConflict => Err((
                    StatusCode::CONFLICT,
                    "Booking conflict in Google Calendar.".to_string(),
                )),
                FulfillmentError::ConfigError(msg) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Configuration error: {}", msg),
                )),
                FulfillmentError::FeatureDisabled(msg) => Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Required feature for fulfillment disabled: {}", msg),
                )),
                #[cfg(feature = "twilio")]
                FulfillmentError::TwilioError(msg) => Err((StatusCode::INTERNAL_SERVER_ERROR, msg)),
                FulfillmentError::InternalError(msg) => {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, msg))
                }
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
    info!(
        "[Fulfillment Handler] Received Adhoc GCal/Twilio fulfillment request for room: {}",
        payload.room_name
    );

    // Authentication is handled by the fulfillment_auth_middleware in auth.rs

    // Check if GCal is enabled in the main app config (needed for booking the slot)
    if !state.config.use_gcal {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "GCal feature is not enabled in config, cannot fulfill adhoc GCal booking.".to_string(),
        ));
    }
    // Also check if the adhoc feature itself is enabled at runtime
    if !state.config.use_adhoc {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Adhoc sessions feature is not enabled in config.".to_string(),
        ));
    }

    // Call the adhoc fulfillment logic
    match fulfill_adhoc_gcal_twilio_logic(State(state), payload).await {
        Ok(response) => {
            info!(
                "[Fulfillment Handler] Adhoc GCal booking successful: Event ID {:?}, Room: {:?}",
                response.event_id, response.room_name
            );
            Ok(Json(response))
        }
        Err(e) => {
            warn!("[Fulfillment Handler] Adhoc GCal booking failed: {}", e);
            match e {
                FulfillmentError::GcalApiError(api_err_msg) => Err((
                    StatusCode::BAD_GATEWAY,
                    format!("Google Calendar API error: {}", api_err_msg),
                )),
                FulfillmentError::GcalBookingConflict => Err((
                    StatusCode::CONFLICT,
                    "Booking conflict in Google Calendar for adhoc session.".to_string(),
                )),
                FulfillmentError::ConfigError(msg) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Configuration error: {}", msg),
                )),
                FulfillmentError::FeatureDisabled(msg) => Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Required feature for fulfillment disabled: {}", msg),
                )),
                #[cfg(feature = "twilio")]
                FulfillmentError::TwilioError(msg) => Err((StatusCode::INTERNAL_SERVER_ERROR, msg)),
                FulfillmentError::InternalError(msg) => {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, msg))
                }
            }
        }
    }
}
