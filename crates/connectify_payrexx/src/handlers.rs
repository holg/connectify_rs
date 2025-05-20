// --- File: crates/connectify_payrexx/src/handlers.rs ---
use axum::{
    body::Bytes,
    extract::{Query, State}, // Added Query
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json, Response}, // Added Html, Response
};
use connectify_config::AppConfig;
use std::sync::Arc;
use tracing::info; // Use the unified config from the config crate
                   // Import logic functions and types
use crate::logic::{
    create_gateway_request,
    CreateGatewayRequest,
    CreateGatewayResponse,
    PayrexxError,
    // PayrexxWebhookPayload, verify_payrexx_signature, process_webhook
};
// Import serde::Deserialize for query params
use serde::Deserialize;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// --- State ---
// Contains only the AppConfig Arc, as the HTTP client is static in logic.rs
#[derive(Clone)]
pub struct PayrexxState {
    pub config: Arc<AppConfig>,
}

// --- Handlers ---

/// Axum handler to create a Payrexx payment gateway.
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/payrexx/create-gateway", // Path relative to /api
    request_body = CreateGatewayRequest,
    responses(
        (status = 200, description = "Gateway created successfully", body = CreateGatewayResponse),
        (status = 400, description = "Bad request (e.g., invalid input)"),
        (status = 500, description = "Internal server error or Payrexx API error")
    ),
    tag = "Payrexx"
))]
pub async fn create_gateway_handler(
    State(state): State<Arc<PayrexxState>>,
    Json(payload): Json<CreateGatewayRequest>,
) -> Result<Json<CreateGatewayResponse>, (StatusCode, String)> {
    // Check the runtime flag from the shared config inside PayrexxState
    if !state.config.use_payrexx {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Payrexx service is disabled.".to_string(),
        ));
    }

    // Extract the specific Payrexx config from the shared AppConfig
    if let Some(payrexx_config) = state.config.payrexx.as_ref() {
        // Call the logic function from logic.rs
        // It uses its own static client now
        match create_gateway_request(payrexx_config, payload).await {
            Ok(response) => Ok(Json(response)),
            Err(PayrexxError::ConfigError) => {
                // Log potentially sensitive config errors internally only
                info!("Payrexx configuration error during gateway creation.");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Server configuration error.".to_string(),
                ))
            }
            Err(PayrexxError::RequestError(e)) => {
                info!("Payrexx Reqwest Error: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to communicate with payment provider.".to_string(),
                ))
            }
            Err(PayrexxError::ParseError(e)) => {
                info!("Payrexx Parse Error: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to understand payment provider response.".to_string(),
                ))
            }
            Err(PayrexxError::ApiError { status, message }) => {
                // Log the actual error from Payrexx
                info!("Payrexx API Error ({}): {}", status, message);
                // Return a generic error to the client
                Err((
                    StatusCode::BAD_GATEWAY,
                    "Payment provider error.".to_string(),
                ))
            }
            Err(PayrexxError::EncodingError(msg)) => {
                info!("Payrexx Encoding Error: {}", msg);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to prepare payment request.".to_string(),
                ))
            }
            Err(PayrexxError::InternalError(msg)) => {
                info!("Payrexx Internal Logic Error: {}", msg);
                Err((StatusCode::INTERNAL_SERVER_ERROR, msg)) // Or a more generic message
            }
            // Webhook errors cannot originate from create_gateway_request
            Err(PayrexxError::WebhookSignatureError)
            | Err(PayrexxError::WebhookProcessingError(_)) => {
                // This case should be unreachable
                info!("Unexpected webhook error during gateway creation!");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unexpected server error.".to_string(),
                ))
            }
        }
    } else {
        // This case means use_payrexx was true, but config loading failed earlier
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Payrexx configuration error (details missing).".to_string(),
        ))
    }
}

/// Axum handler for incoming Payrexx webhooks (Server-to-Server).
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/payrexx/webhook", // Path relative to /api
    request_body = crate::logic::PayrexxWebhookPayload,
    responses(
        (status = 200, description = "Webhook received successfully"),
        (status = 400, description = "Bad request (e.g., invalid signature, bad payload)"),
        (status = 500, description = "Internal server error processing webhook")
    ),
    tag = "Payrexx"
))]
pub async fn payrexx_webhook_handler(
    State(state): State<Arc<PayrexxState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !state.config.use_payrexx {
        return (StatusCode::SERVICE_UNAVAILABLE, "Payrexx service disabled.").into_response();
    }

    // --- Verify Signature ---
    let api_secret = match std::env::var("PAYREXX_API_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            info!("🚨 PAYREXX_API_SECRET missing for webhook verification!");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    // Adjust header name if needed based on Payrexx documentation
    let signature_header = headers
        .get("Webhook-Signature")
        .and_then(|hv| hv.to_str().ok());

    // Call the verification logic function
    if let Err(e) = crate::logic::verify_payrexx_signature(&api_secret, &body, signature_header) {
        info!("Webhook signature verification failed: {:?}", e);
        // Check if it was specifically a signature error
        if matches!(e, PayrexxError::WebhookSignatureError) {
            return (StatusCode::BAD_REQUEST, "Invalid signature".to_string()).into_response();
        } else {
            // Handle other potential errors from verify_payrexx_signature if any
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error during signature check".to_string(),
            )
                .into_response();
        }
    }
    info!("✅ Payrexx webhook signature verified.");

    // --- Process Payload ---
    // Deserialize the raw body AFTER signature verification
    let payload: crate::logic::PayrexxWebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            info!("Failed to deserialize Payrexx webhook payload: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                "Invalid payload format".to_string(),
            )
                .into_response();
        }
    };

    // Call the processing logic
    match crate::logic::process_webhook(payload.clone()).await {
        // Pass DB pool etc. if needed
        Ok(()) => {
            // Acknowledge receipt to Payrexx with 200 OK
            info!("Webhook processed successfully.:{:?}", payload.clone()); // Debug
            StatusCode::OK.into_response()
        }
        Err(e) => {
            info!("Error processing Payrexx webhook: {}", e);
            // Determine response based on the specific error from process_webhook
            match e {
                PayrexxError::WebhookProcessingError(msg) => {
                    // Specific internal error during processing
                    info!("Webhook Processing Error: {}", msg);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Webhook processing failed".to_string(),
                    )
                        .into_response()
                }
                // Handle other potential errors returned by process_webhook if necessary
                _ => {
                    // Generic internal server error for other unexpected errors
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    )
                        .into_response()
                }
            }
        }
    }
}

// --- Redirect Handlers ---

// Define struct for potential query parameters Payrexx might add to redirects
#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams, ToSchema))] // Add IntoParams for docs
pub struct RedirectQuery {
    // Example: Payrexx might add transaction ID or your reference ID
    #[serde(rename = "transactionId")] // Use serde rename if needed
    #[cfg_attr(feature = "openapi", param(example = 12345))]
    pub transaction_id: Option<i64>,
    #[serde(rename = "referenceId")]
    #[cfg_attr(feature = "openapi", param(example = "connectify-xxx-123"))]
    pub reference_id: Option<String>,
    // Add other potential query params based on Payrexx docs
}

/// Handler for successful payment redirect. Shows a simple success message.
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/payrexx/webhook/success", // Path relative to /api
    params(RedirectQuery),
    responses( (status = 200, description = "Success page shown to user after payment", content_type = "text/html") ),
    tag = "Payrexx Redirects"
))]
pub async fn payrexx_success_handler(
    Query(params): Query<RedirectQuery>, // Extract query params
) -> Html<&'static str> {
    info!("User redirected to success URL. Params: {:?}", params);
    // TODO: Enhance this page - maybe show order details based on params?
    Html("<h1>Payment Successful!</h1><p>Thank you. Your booking is confirmed.</p><a href='/'>Back to Home</a>")
}

/// Handler for failed payment redirect. Shows a simple failure message.
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/payrexx/webhook/failure", // Path relative to /api
    params(RedirectQuery),
    responses( (status = 200, description = "Failure page shown to user after payment", content_type = "text/html") ),
    tag = "Payrexx Redirects"
))]
pub async fn payrexx_failure_handler(Query(params): Query<RedirectQuery>) -> Html<&'static str> {
    info!("User redirected to failure URL. Params: {:?}", params);
    // TODO: Enhance this page
    Html("<h1>Payment Failed</h1><p>Unfortunately, your payment could not be processed. Please try again or contact support.</p><a href='/'>Back to Home</a>")
}

/// Handler for cancelled payment redirect. Shows a cancellation message.
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/payrexx/webhook/cancel", // Path relative to /api
    params(RedirectQuery),
    responses( (status = 200, description = "Cancellation page shown to user", content_type = "text/html") ),
    tag = "Payrexx Redirects"
))]
pub async fn payrexx_cancel_handler(Query(params): Query<RedirectQuery>) -> Html<&'static str> {
    info!("User redirected to cancel URL. Params: {:?}", params);
    // TODO: Enhance this page
    Html("<h1>Payment Cancelled</h1><p>You have cancelled the payment process.</p><a href='/'>Home</a>")
}
