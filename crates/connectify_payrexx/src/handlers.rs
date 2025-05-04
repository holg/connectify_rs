// --- File: crates/connectify_payrexx/src/handlers.rs ---

use axum::{
    extract::{State},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use std::sync::Arc;
use connectify_config::AppConfig; // Use the unified config from the config crate
use crate::logic::{
    create_gateway_request, CreateGatewayRequest, CreateGatewayResponse, PayrexxError
};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// --- Define the specific state needed for Payrexx handlers ---
#[derive(Clone)]
pub struct PayrexxState {
    pub config: Arc<AppConfig>,
    // REMOVED: pub http_client: Arc<Client>,
}

/// Axum handler to create a Payrexx payment gateway.
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path( // Keep OpenAPI docs if using that feature
    post,
    path = "/api/payrexx/create-gateway",
    request_body = CreateGatewayRequest,
    responses(
        (status = 200, description = "Gateway created successfully", body = CreateGatewayResponse),
        (status = 400, description = "Bad request (e.g., invalid input)"),
        (status = 500, description = "Internal server error or Payrexx API error")
    ),
    tag = "Payrexx"
))]
pub async fn create_gateway_handler(
    // Extract the Payrexx-specific state
    State(state): State<Arc<PayrexxState>>, // <-- Expect PayrexxState here
    Json(payload): Json<CreateGatewayRequest>,
) -> Result<Json<CreateGatewayResponse>, (StatusCode, String)> {

    // Check the runtime flag from the shared config inside PayrexxState
    if !state.config.use_payrexx {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Payrexx service is disabled.".to_string()));
    }

    // Extract the specific Payrexx config from the shared AppConfig
    // Use if let to handle the Option gracefully
    if let Some(payrexx_config) = state.config.payrexx.as_ref() {

        // REMOVED: let http_client_ref = &state.http_client; // No longer needed

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
            },
            Err(PayrexxError::WebhookSignatureError) => {
                Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Webhook processing error occurred unexpectedly")))
            },
            Err(PayrexxError::WebhookProcessingError(msg)) => {
                Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Webhook processing error occurred unexpectedly: {}", msg)))
            } // Should not happen here// Should not happen here
        }

    } else {
        // This case means use_payrexx was true, but config loading failed earlier
        Err((StatusCode::INTERNAL_SERVER_ERROR, "Payrexx configuration error (details missing).".to_string()))
    }
}

// --- Webhook Handler (Keep as before) ---
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/api/payrexx/webhook",
    request_body = crate::logic::PayrexxWebhookPayload, // Use logic struct
    responses(
        (status = 200, description = "Webhook received successfully"),
        (status = 400, description = "Bad request (e.g., invalid signature, bad payload)"),
        (status = 500, description = "Internal server error processing webhook")
    ),
    tag = "Payrexx"
))]
pub async fn payrexx_webhook_handler(
    State(state): State<Arc<PayrexxState>>, // Still needs config for secret
    headers: axum::http::HeaderMap, // Correct type for headers
    body: axum::body::Bytes, // Correct type for raw body
) -> axum::response::Response { // Return generic Response

    if !state.config.use_payrexx {
        return (StatusCode::SERVICE_UNAVAILABLE, "Payrexx service disabled.").into_response();
    }

    // --- Verify Signature ---
    let api_secret = match std::env::var("PAYREXX_API_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            eprintln!("🚨 PAYREXX_API_SECRET missing for webhook verification!");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let signature_header = headers.get("Webhook-Signature").and_then(|hv| hv.to_str().ok());

    if let Err(e) = crate::logic::verify_payrexx_signature(&api_secret, &body, signature_header) {
        eprintln!("Webhook signature verification failed: {:?}", e);
        return (StatusCode::BAD_REQUEST, "Invalid signature".to_string()).into_response();
    }
    println!("✅ Payrexx webhook signature verified.");

    // --- Process Payload ---
    let payload: crate::logic::PayrexxWebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to deserialize Payrexx webhook payload: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid payload format".to_string()).into_response();
        }
    };

    match crate::logic::process_webhook(payload.clone()).await { // Pass DB pool etc. if needed
        Ok(()) => {
            println!("Webhook processed successfully.:{:?}", payload.clone());
            match serde_json::to_string_pretty(&payload) {
                Ok(json) => println!("Webhook processed successfully:\n{}", json),
                Err(_) => println!("Webhook processed successfully. (Failed to format JSON payload)"),
            }

            StatusCode::OK.into_response()
        }
        Err(e) => {
            eprintln!("Error processing Payrexx webhook: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
