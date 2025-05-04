// --- File: crates/connectify_payrexx/src/handlers.rs ---

use axum::{
    extract::{State},
    response::{Json},
    http::StatusCode,
};
use std::sync::Arc;
use connectify_config::AppConfig; // Use the unified config from the config crate
// Removed: use reqwest::Client; // No longer needed in state
// ** Corrected Imports from logic module **
use crate::logic::{
    create_gateway_request, // Function is pub
    CreateGatewayRequest,   // Struct is pub
    CreateGatewayResponse,  // Struct is pub
    PayrexxError            // Enum is pub
    // Removed non-existent HTTPP_CLIENT_REF
};

// --- Define the specific state needed for Payrexx handlers ---
// Contains only the AppConfig Arc, as the HTTP client is static in logic.rs
#[derive(Clone)]
pub struct PayrexxState {
    pub config: Arc<AppConfig>,
}

/// Axum handler to create a Payrexx payment gateway.
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path( /* ... OpenAPI attributes ... */ ))]
pub async fn create_gateway_handler(
    // Extract the Payrexx-specific state
    State(state): State<Arc<PayrexxState>>,
    Json(payload): Json<CreateGatewayRequest>,
) -> Result<Json<CreateGatewayResponse>, (StatusCode, String)> {

    // Check the runtime flag from the shared config inside PayrexxState
    if !state.config.use_payrexx {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Payrexx service is disabled.".to_string()));
    }

    // Extract the specific Payrexx config from the shared AppConfig
    // Use if let to handle the Option gracefully
    if let Some(payrexx_config) = state.config.payrexx.as_ref() {

        // ** REMOVED: Unnecessary access to static client **
        // let http_client_ref = &HTPP_CLIENT_REF; // This was incorrect

        // Call the logic function from logic.rs
        // It uses its own static client now
        match create_gateway_request(payrexx_config, payload).await {
            Ok(response) => Ok(Json(response)),
            Err(PayrexxError::ConfigError) => {
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Payrexx configuration error on server.".to_string()))
            }
            Err(PayrexxError::RequestError(e)) => {
                eprintln!("Payrexx Reqwest Error: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to communicate with payment provider.".to_string()))
            }
            Err(PayrexxError::ParseError(e)) => {
                eprintln!("Payrexx Parse Error: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to understand payment provider response.".to_string()))
            }
            Err(PayrexxError::ApiError { status, message }) => {
                Err((StatusCode::BAD_GATEWAY, format!("Payment provider error: {} - {}", status, message)))
            }
            Err(PayrexxError::InternalError(msg)) => {
                Err((StatusCode::INTERNAL_SERVER_ERROR, msg))
            }
        }

    } else {
        // This case means use_payrexx was true, but config loading failed earlier
        Err((StatusCode::INTERNAL_SERVER_ERROR, "Payrexx configuration error (details missing).".to_string()))
    }
}
