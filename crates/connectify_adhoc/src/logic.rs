// --- File: crates/connectify_adhoc/src/logic.rs ---

use chrono::{Duration, Utc};
#[allow(unused_imports)]
use chrono_tz::Tz;
use connectify_config::AppConfig; //, StripeConfig, GcalConfig, PriceTier, AdhocSessionSettings};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

// We need to call GCal logic and Stripe logic
#[cfg(feature = "gcal")]
use connectify_gcal::auth::HubType as GcalHubType;
#[cfg(feature = "gcal")]
use connectify_gcal::logic::{get_busy_times as gcal_get_busy_times, GcalError}; // If passing an existing hub

#[cfg(feature = "stripe")]
use connectify_stripe::error::StripeError;
#[cfg(feature = "stripe")]
use connectify_stripe::logic::{
    create_checkout_session as stripe_create_checkout_session,
    CreateCheckoutSessionRequest as StripeCreateCheckoutRequest,
    // CreateCheckoutSessionResponse as StripeCreateCheckoutResponse
};

#[derive(Error, Debug)]
pub enum AdhocSessionError {
    #[error("Adhoc sessions are currently disabled by admin.")]
    AdminDisabled,
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Google Calendar interaction failed: {0}")]
    GcalInteractionError(String),
    #[error("Requested time slot (now + preparation) is not available.")]
    SlotUnavailable,
    #[error("No matching price tier found for duration: {0} minutes.")]
    NoMatchingPriceTier(i64),
    #[error("Stripe session creation failed: {0}")]
    StripeError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[cfg(feature = "gcal")]
impl From<GcalError> for AdhocSessionError {
    fn from(err: GcalError) -> Self {
        AdhocSessionError::GcalInteractionError(err.to_string())
    }
}

#[cfg(feature = "stripe")]
impl From<StripeError> for AdhocSessionError {
    fn from(err: StripeError) -> Self {
        AdhocSessionError::StripeError(err.to_string())
    }
}

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct InitiateAdhocSessionRequest {
    #[cfg_attr(feature = "openapi", schema(example = 30))]
    pub duration_minutes: i64,
    // Potentially add user_identifier if you want to link this to a logged-in user
    // pub user_id: Option<String>,
}

#[derive(Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct InitiateAdhocSessionResponse {
    #[cfg_attr(
        feature = "openapi",
        schema(example = "https://checkout.stripe.com/pay/cs_test_...")
    )]
    pub stripe_checkout_url: String,
    #[cfg_attr(feature = "openapi", schema(example = "cs_test_..."))]
    pub stripe_session_id: String,
    #[cfg_attr(
        feature = "openapi",
        schema(example = "adhoc-room-123e4567-e89b-12d3-a456-426614174000")
    )]
    pub room_name: String,
    #[cfg_attr(feature = "openapi", schema(example = "2025-05-20T10:30:00Z"))]
    pub effective_start_time: String, // ISO 8601
    #[cfg_attr(feature = "openapi", schema(example = "2025-05-20T11:00:00Z"))]
    pub effective_end_time: String, // ISO 8601
}

pub async fn initiate_adhoc_session_logic(
    app_config: Arc<AppConfig>,
    request_data: InitiateAdhocSessionRequest,
    // If GCalState is managed centrally and passed in:
    #[cfg(feature = "gcal")] gcal_hub: Option<Arc<GcalHubType>>, // Pass the initialized GCal Hub if available
) -> Result<InitiateAdhocSessionResponse, AdhocSessionError> {
    let adhoc_settings = app_config.adhoc_settings.as_ref().ok_or_else(|| {
        AdhocSessionError::ConfigError("Adhoc session settings not configured.".to_string())
    })?;

    if !adhoc_settings.admin_enabled {
        return Err(AdhocSessionError::AdminDisabled);
    }

    let gcal_config = app_config
        .gcal
        .as_ref()
        .ok_or_else(|| AdhocSessionError::ConfigError("GCal configuration missing.".to_string()))?;
    let stripe_config = app_config.stripe.as_ref().ok_or_else(|| {
        AdhocSessionError::ConfigError("Stripe configuration missing.".to_string())
    })?;
    #[allow(unused_variables)]
    let calendar_id = gcal_config
        .calendar_id
        .as_ref()
        .ok_or_else(|| AdhocSessionError::ConfigError("GCal calendar_id missing.".to_string()))?;

    let now = Utc::now();
    let preparation_duration = Duration::minutes(adhoc_settings.preparation_time_minutes);
    let session_duration = Duration::minutes(request_data.duration_minutes);

    let effective_start_time = now + preparation_duration;
    let effective_end_time = effective_start_time + session_duration;

    // 1. Check GCal Availability
    #[cfg(feature = "gcal")]
    {
        // Create a hub if not provided (less efficient if called often)
        let hub_instance;
        let hub_to_use = match gcal_hub {
            Some(h_arc) => h_arc,
            None => {
                hub_instance = Arc::new(
                    connectify_gcal::auth::create_calendar_hub(gcal_config)
                        .await
                        .map_err(|e| {
                            AdhocSessionError::GcalInteractionError(format!(
                                "Failed to create GCal client: {}",
                                e
                            ))
                        })?,
                );
                hub_instance
            }
        };
        // Check if the requested time slot is available in GCal
        let timezone = gcal_config
            .time_zone
            .as_ref()
            .ok_or_else(|| AdhocSessionError::ConfigError("GCal timezone missing.".to_string()))?
            .parse::<Tz>()
            .map_err(|_| AdhocSessionError::ConfigError("Invalid timezone format.".to_string()))?;

        let busy_times = gcal_get_busy_times(
            &hub_to_use,
            calendar_id,
            effective_start_time.with_timezone(&timezone),
            effective_end_time.with_timezone(&timezone),
        )
        .await?;

        if !busy_times.is_empty() {
            // Check if any busy period overlaps with the desired slot
            for (busy_start, busy_end) in busy_times {
                if effective_start_time < busy_end && effective_end_time > busy_start {
                    return Err(AdhocSessionError::SlotUnavailable);
                }
            }
        }
        info!(
            "[Adhoc Logic] Slot from {} to {} is available.",
            effective_start_time, effective_end_time
        );
    }
    #[cfg(not(feature = "gcal"))]
    {
        info!("[Adhoc Logic] GCal feature not enabled, skipping availability check.");
        // If GCal is not enabled, you might want to always allow or have a different check.
        // For now, we'll proceed assuming it's available if GCal feature is off.
    }

    // 2. Find Price Tier
    let price_tier = stripe_config
        .price_tiers
        .iter()
        .find(|t| t.duration_minutes == request_data.duration_minutes)
        .ok_or(AdhocSessionError::NoMatchingPriceTier(
            request_data.duration_minutes,
        ))?;

    // 3. Generate unique room name
    let room_name = format!("adhoc-{}", uuid::Uuid::new_v4());

    // 4. Prepare data for Stripe Checkout Session
    let gcal_summary = price_tier.product_name.clone().unwrap_or_else(|| {
        format!(
            "Adhoc Session {} Min - {}",
            request_data.duration_minutes, room_name
        )
    });
    #[allow(unused_variables)]
    let fulfillment_data = serde_json::json!({
        "start_time": effective_start_time.to_rfc3339(),
        "end_time": effective_end_time.to_rfc3339(),
        "summary": gcal_summary,
        "description": format!("Adhoc session booked via Connectify. Room: {}", room_name),
        "room_name": room_name, // Pass room_name for fulfillment if needed
        "original_duration_request_minutes": request_data.duration_minutes, // For logging/verification
    });

    #[cfg(feature = "stripe")]
    let stripe_request = StripeCreateCheckoutRequest {
        product_name_override: None, // Price and product name determined by tier
        amount_override: None,       // Price determined by tier
        currency_override: None,     // Currency determined by tier or StripeConfig default
        fulfillment_type: "adhoc_gcal_twilio".to_string(), // New fulfillment type
        fulfillment_data,
        client_reference_id: Some("adhoc-{{CHECKOUT_SESSION_ID}}".to_string()), // Unique ref
    };

    // 5. Create Stripe Checkout Session
    // The success_url needs to include the room_name for the webcam page
    let mut dynamic_stripe_config = stripe_config.clone(); // Clone to modify success_url
    dynamic_stripe_config.success_url = format!(
        // "{}?room_name={}&session_id={{CHECKOUT_SESSION_ID}}", // Assuming base success_url doesn't have query params
        "{}&room_name=adhoc_{{CHECKOUT_SESSION_ID}}", // Assuming base success_url doesn't have query params
        stripe_config.success_url.trim_end_matches('/'), // Ensure no double slash
    );
    // If your base success_url already has query params, append with &
    // Example: "https://.../success.html?someparam=value&room_name=...&session_id=..."

    #[cfg(feature = "stripe")]
    let stripe_session_response =
        stripe_create_checkout_session(&dynamic_stripe_config, stripe_request).await?;

    #[cfg(feature = "stripe")]
    {
        Ok(InitiateAdhocSessionResponse {
            stripe_checkout_url: stripe_session_response.url,
            stripe_session_id: stripe_session_response.session_id,
            room_name,
            effective_start_time: effective_start_time.to_rfc3339(),
            effective_end_time: effective_end_time.to_rfc3339(),
        })
    }
    #[cfg(not(feature = "stripe"))]
    {
        Err(AdhocSessionError::ConfigError(
            "Stripe feature not enabled.".to_string(),
        ))
    }
}
