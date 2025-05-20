// --- File: crates/connectify_adhoc/src/handlers.rs ---
use crate::logic::{
    initiate_adhoc_session_logic, AdhocSessionError, InitiateAdhocSessionRequest,
    InitiateAdhocSessionResponse,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json, //, IntoResponse}, // Added Html, Response
};
use connectify_config::AppConfig;
use std::sync::Arc;

// State for Adhoc handlers
#[derive(Clone)]
pub struct AdhocState {
    pub config: Arc<AppConfig>,
    #[cfg(feature = "gcal")]
    pub gcal_hub: Option<Arc<connectify_gcal::auth::HubType>>, // Pass initialized GCal Hub
}

#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/adhoc/initiate-session", // Relative to /api
    request_body = InitiateAdhocSessionRequest,
    responses(
        (status = 200, description = "Adhoc session initiated, Stripe URL returned", body = InitiateAdhocSessionResponse),
        (status = 400, description = "Invalid request (e.g., bad duration)"),
        (status = 403, description = "Adhoc sessions admin-disabled"),
        (status = 409, description = "Slot unavailable"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Adhoc Sessions"
))]
pub async fn initiate_adhoc_session_handler(
    State(state): State<Arc<AdhocState>>,
    Json(payload): Json<InitiateAdhocSessionRequest>,
) -> Result<Json<InitiateAdhocSessionResponse>, (StatusCode, String)> {
    // Check main feature flag for adhoc sessions
    if !state.config.use_adhoc {
        return Err((
            StatusCode::NOT_FOUND,
            "Adhoc session feature not enabled.".to_string(),
        ));
    }

    match initiate_adhoc_session_logic(
        state.config.clone(),
        payload,
        #[cfg(feature = "gcal")]
        state.gcal_hub.clone(), // Pass the GCal Hub from state
    )
    .await
    {
        Ok(response) => Ok(Json(response)),
        Err(AdhocSessionError::AdminDisabled) => Err((
            StatusCode::FORBIDDEN,
            AdhocSessionError::AdminDisabled.to_string(),
        )),
        Err(AdhocSessionError::ConfigError(msg)) => Err((StatusCode::INTERNAL_SERVER_ERROR, msg)),
        Err(AdhocSessionError::GcalInteractionError(msg)) => Err((StatusCode::BAD_GATEWAY, msg)),
        Err(AdhocSessionError::SlotUnavailable) => Err((
            StatusCode::CONFLICT,
            AdhocSessionError::SlotUnavailable.to_string(),
        )),
        Err(AdhocSessionError::NoMatchingPriceTier(duration)) => Err((
            StatusCode::BAD_REQUEST,
            format!("No price for duration: {} minutes.", duration),
        )),
        Err(AdhocSessionError::StripeError(msg)) => {
            Err((StatusCode::BAD_GATEWAY, format!("Stripe error: {}", msg)))
        }
        Err(AdhocSessionError::InternalError(msg)) => Err((StatusCode::INTERNAL_SERVER_ERROR, msg)),
    }
}
