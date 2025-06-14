use axum::extract::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
#[allow(unused_imports)]
use tracing::{info, warn}; // To access GCal config
                           // use std::sync::Arc;

use crate::FulfillmentState;
/// Conditionally imports Google Calendar related functionality when the "gcal" feature is enabled.
///
/// This import block brings in the necessary types and functions from the `connectify_gcal` crate
/// that are required for Google Calendar integration:
///
/// # Imports
/// * `create_calendar_hub` - Function to create an authenticated Google Calendar API client
/// * `create_calendar_event` (aliased as `gcal_create_event`) - Function to create events in Google Calendar
/// * `BookSlotRequest` (aliased as `GcalBookSlotRequest`) - Request structure for booking calendar slots
/// * `GcalError` - Error type for Google Calendar operations
/// * `HubType` (aliased as `GcalHubType`) - The type representing the Google Calendar API client
#[cfg(feature = "gcal")]
use connectify_gcal::{
    auth::create_calendar_hub, // Function to create the GCal Hub
    logic::{
        create_calendar_event as gcal_create_event, BookSlotRequest as GcalBookSlotRequest,
        GcalError,
    }, // GCal's booking logic and request struct
};

// Import SmsRequest directly from connectify_twilio to avoid confusion
#[cfg(feature = "twilio")]
use connectify_twilio::twilio_sms::SmsRequest;

// --- Error Handling for Fulfillment ---
#[derive(Error, Debug)]
pub enum FulfillmentError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[cfg(feature = "gcal")] // GCal specific errors
    #[error("Google Calendar API error: {0}")]
    GcalApiError(String), // Store as String to avoid direct GcalError dependency if possible

    #[cfg(feature = "gcal")] // GCal specific errors
    #[error("Google Calendar booking conflict")]
    GcalBookingConflict,

    #[error("Internal feature disabled: {0}")]
    FeatureDisabled(String),
    // Add other specific errors for other fulfillment types (e.g., Twilio)
    #[cfg(feature = "twilio")]
    #[error("Twilio fulfillment error: {0}")]
    TwilioError(String),

    #[error("Internal fulfillment error: {0}")]
    InternalError(String),
}
// --- Request Structures for Fulfillment Tasks ---

/// Data needed to fulfill a Google Calendar booking.
/// This would typically be sent by another service (e.g., Stripe webhook handler)
/// after a payment is confirmed.
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GcalBookingFulfillmentRequest {
    // Fields needed by connectify_gcal::logic::create_calendar_event
    #[cfg_attr(feature = "openapi", schema(example = "2025-06-10T10:00:00Z"))]
    pub start_time: String,
    #[cfg_attr(feature = "openapi", schema(example = "2025-06-10T11:00:00Z"))]
    pub end_time: String,
    #[cfg_attr(
        feature = "openapi",
        schema(example = "Confirmed Booking via Connectify")
    )]
    pub summary: String,
    pub description: Option<String>,
    // Potentially other details like user_id, original_reference_id for logging/tracking
    pub original_reference_id: Option<String>,
    pub payment_id: Option<String>,     // e.g., Stripe payment ID
    pub payment_method: Option<String>, // e.g., "stripe"
    pub payment_amount: Option<i64>,    // e.g., 1000 (in cents)
    pub room_name: Option<String>,
}

// --- Response Structures for Fulfillment Tasks ---

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FulfillmentResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>, // e.g., GCal event ID
    #[serde(skip_serializing_if = "Option::is_none")] // Added for adhoc
    pub room_name: Option<String>,
}

// --- Core Fulfillment Logic Functions ---

/// Logic to fulfill a Google Calendar booking.
/// This function will be called by the corresponding handler.
#[cfg(feature = "gcal")] // Only compile if gcal feature is active for this crate
pub async fn fulfill_gcal_booking_logic(
    State(state): State<Arc<FulfillmentState>>, // Specific GCal config from AppConfig
    payload: GcalBookingFulfillmentRequest,
    // We need a CalendarHub. It can be created here or passed in if already initialized.
    // Creating it here makes this function more self-contained but might re-create clients.
    // Passing an Arc<GcalHubType> from a shared state (like GcalState) is often better.
    // For now, let's create it here for simplicity, assuming gcal_config has key_path.
) -> Result<FulfillmentResponse, FulfillmentError> {
    info!("Attempting to fulfill GCal booking: {:?}", payload.summary);
    let gcal_config = state
        .config
        .gcal
        .as_ref()
        .expect("GCal config not found in AppConfig");
    // 1. Create the CalendarHub instance
    // This requires the GcalConfig to have the necessary details (key_path)
    let hub = create_calendar_hub(gcal_config).await.map_err(|e| {
        FulfillmentError::GcalApiError(format!("Failed to create GCal client: {}", e))
    })?;

    // 2. Prepare the booking request for the connectify_gcal::logic module
    let gcal_book_request = GcalBookSlotRequest {
        start_time: payload.start_time.clone(),
        end_time: payload.end_time.clone(),
        summary: payload.summary.clone(),
        description: payload.description,
        payment_method: payload.payment_method,
        payment_amount: payload.payment_amount,
        payment_id: Some(
            payload
                .payment_id
                .unwrap_or_else(|| format!("gcal-booking-{}", chrono::Utc::now().timestamp())),
        ),
        room_name: payload.room_name.clone(),
        // Add other fields if GcalBookSlotRequest expects them
    };

    // 3. Call the booking function from connectify_gcal
    match gcal_create_event(
        &hub,
        gcal_config.calendar_id.as_ref().ok_or_else(|| {
            FulfillmentError::ConfigError("Missing GCal calendar_id in config".to_string())
        })?,
        gcal_book_request,
    )
    .await
    {
        Ok(created_event) => {
            let event_id = created_event.id;
            info!("Successfully booked GCal event. ID: {:?}", event_id);

            // Send SMS notification if Twilio is enabled - using modified approach
            #[cfg(feature = "twilio")]
            {
                info!("Twilio feature is enabled at compile time");
                if let Some(twilio_config) = state.config.twilio.as_ref() {
                    if state.config.use_twilio {
                        info!("Twilio is enabled in runtime config, preparing to send SMS");

                        // Create an explicit instance of SmsRequest from connectify_twilio
                        let sms_request = SmsRequest {
                            to: twilio_config.phone_number.to_string(),
                            message: format!(
                                "Appointment confirmed: start_time: {}, end_time: {}, summary: {}",
                                &payload.start_time, &payload.end_time, &payload.summary
                            ),
                        };

                        // Use the full path for the send_sms function
                        match connectify_twilio::twilio_sms::send_sms(
                            State(state.config.clone()),
                            axum::Json(sms_request),
                        )
                        .await
                        {
                            Ok(_) => {
                                info!("SMS notification sent successfully");
                            }
                            Err(e) => {
                                warn!("Failed to send SMS notification: {:?}", e);
                            }
                        }
                    } else {
                        info!("Twilio is disabled in runtime config");
                    }
                } else {
                    warn!("Twilio config section missing in AppConfig");
                }
            }

            #[cfg(not(feature = "twilio"))]
            {
                info!("Twilio feature is not enabled at compile time");
            }

            Ok(FulfillmentResponse {
                success: true,
                message: "Google Calendar event booked successfully.".to_string(),
                event_id,
                room_name: payload.room_name,
            })
        }
        Err(GcalError::Conflict) => {
            warn!("GCal booking conflict for summary: {}", payload.summary);
            Err(FulfillmentError::GcalBookingConflict)
        }
        Err(e) => {
            info!("Error booking GCal event: {}", e);
            Err(FulfillmentError::GcalApiError(e.to_string()))
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdhocGcalTwilioFulfillmentRequest {
    #[cfg_attr(feature = "openapi", schema(example = "2025-06-10T10:00:00Z"))]
    pub start_time: String,
    #[cfg_attr(feature = "openapi", schema(example = "2025-06-10T11:00:00Z"))]
    pub end_time: String,
    #[cfg_attr(
        feature = "openapi",
        schema(example = "Adhoc Session - Room adhoc-xyz")
    )]
    pub summary: String,
    pub description: Option<String>,
    #[cfg_attr(feature = "openapi", schema(example = "adhoc-xyz123-abc"))]
    pub room_name: String,
    pub original_reference_id: Option<String>,
    pub payment_method: Option<String>,
    pub payment_amount: Option<i64>,
    pub payment_id: Option<String>,
}

#[cfg(feature = "gcal")] // This fulfillment type also depends on GCal
pub async fn fulfill_adhoc_gcal_twilio_logic(
    State(state): State<Arc<FulfillmentState>>,
    payload: AdhocGcalTwilioFulfillmentRequest,
) -> Result<FulfillmentResponse, FulfillmentError> {
    let gcal_config = state
        .config
        .gcal
        .as_ref()
        .expect("GCal config not found in AppConfig");
    info!(
        "[Fulfillment Logic] Attempting Adhoc GCal booking for room: {}, summary: {}",
        payload.room_name, payload.summary
    );

    let calendar_id_to_use = gcal_config.calendar_id.as_ref().ok_or_else(|| {
        FulfillmentError::ConfigError(
            "Missing GCal calendar_id in config for adhoc booking".to_string(),
        )
    })?;

    let hub = create_calendar_hub(gcal_config).await.map_err(|e| {
        FulfillmentError::GcalApiError(format!("Failed to create GCal client for adhoc: {}", e))
    })?;

    let gcal_book_request = GcalBookSlotRequest {
        start_time: payload.start_time.clone(),
        end_time: payload.end_time.clone(),
        summary: payload.summary.clone(),
        description: payload.description.clone(),
        payment_method: Some(payload.payment_method.unwrap_or("stripe".to_string())),
        payment_amount: Some(payload.payment_amount.unwrap_or(0)),
        payment_id: Some(
            payload
                .original_reference_id
                .unwrap_or_else(|| format!("adhoc-booking-{}", chrono::Utc::now().timestamp())),
        ),
        room_name: Some(payload.room_name.clone()),
    };

    match gcal_create_event(&hub, calendar_id_to_use, gcal_book_request).await {
        Ok(created_event) => {
            let event_id = created_event.id;
            info!(
                "[Fulfillment Logic] Successfully booked Adhoc GCal event. ID: {:?}, Room: {}",
                event_id,
                payload.room_name.clone()
            );

            // Send SMS notification if Twilio is enabled - using modified approach
            #[cfg(feature = "twilio")]
            {
                info!("Twilio feature is enabled at compile time for adhoc");
                if let Some(twilio_config) = state.config.twilio.as_ref() {
                    if state.config.use_twilio {
                        info!("Twilio is enabled in runtime config, preparing to send adhoc SMS");

                        // Create an explicit instance of SmsRequest
                        let sms_request = SmsRequest {
                            to: twilio_config.phone_number.to_string(),
                            message: format!(
                                "Adhoc session booked: room: {}, start: {}, end: {}, summary: {}",
                                &payload.room_name,
                                &payload.start_time,
                                &payload.end_time,
                                &payload.summary
                            ),
                        };

                        // Use the full path for the send_sms function
                        match connectify_twilio::twilio_sms::send_sms(
                            State(state.config.clone()),
                            axum::Json(sms_request),
                        )
                        .await
                        {
                            Ok(_) => {
                                info!("Adhoc SMS notification sent successfully");
                            }
                            Err(e) => {
                                warn!("Failed to send adhoc SMS notification: {:?}", e);
                            }
                        }
                    } else {
                        info!("Twilio is disabled in runtime config");
                    }
                } else {
                    warn!("Twilio config section missing in AppConfig");
                }
            }

            #[cfg(not(feature = "twilio"))]
            {
                info!("Twilio feature is not enabled at compile time for adhoc");
            }

            Ok(FulfillmentResponse {
                success: true,
                message: "Adhoc Google Calendar event booked successfully.".to_string(),
                event_id,
                room_name: Some(payload.room_name),
            })
        }
        Err(GcalError::Conflict) => {
            info!(
                "[Fulfillment Logic] Adhoc GCal booking conflict for summary: {}",
                payload.summary
            );
            Err(FulfillmentError::GcalBookingConflict)
        }
        Err(e) => {
            info!("[Fulfillment Logic] Error booking Adhoc GCal event: {}", e);
            Err(FulfillmentError::GcalApiError(e.to_string()))
        }
    }
}

#[cfg(not(feature = "gcal"))]
pub async fn fulfill_adhoc_gcal_twilio_logic(
    _state: State<Arc<FulfillmentState>>,
    _payload: AdhocGcalTwilioFulfillmentRequest,
) -> Result<FulfillmentResponse, FulfillmentError> {
    info!("[Fulfillment Logic] GCal feature not enabled. Cannot fulfill Adhoc GCal booking.");
    Err(FulfillmentError::FeatureDisabled(
        "GCal feature not enabled".to_string(),
    ))
}

// TODO: Implement logic for other fulfillment tasks
// pub async fn fulfill_twilio_adhoc_session_logic(...) -> Result<FulfillmentResponse, FulfillmentError> { ... }
