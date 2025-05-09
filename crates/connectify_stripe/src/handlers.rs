// --- File: crates/connectify_stripe/src/handlers.rs ---
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Json, Response},
    http::{StatusCode, HeaderMap},
};
use std::sync::Arc;
use connectify_config::AppConfig;
use serde::Deserialize;
use utoipa::ToSchema;
use crate::logic::{
    create_checkout_session, CreateCheckoutSessionRequest, CreateCheckoutSessionResponse, StripeError,
    StripeEvent, verify_stripe_signature, process_stripe_webhook
};

// --- State for Stripe Handlers ---
// Only needs AppConfig as reqwest::Client is static in logic.rs
#[derive(Clone)]
pub struct StripeState {
    pub config: Arc<AppConfig>,
}

/// Axum handler to create a Stripe Checkout Session.
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/stripe/create-checkout-session", // Path relative to /api
    request_body = CreateCheckoutSessionRequest,
    responses(
        (status = 200, description = "Stripe Checkout Session created", body = CreateCheckoutSessionResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error or Stripe API error")
    ),
    tag = "Stripe"
))]
pub async fn create_checkout_session_handler(
    State(state): State<Arc<StripeState>>,
    Json(payload): Json<CreateCheckoutSessionRequest>,
) -> Result<Json<CreateCheckoutSessionResponse>, (StatusCode, String)> {

    if !state.config.use_stripe {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Stripe service is disabled.".to_string()));
    }

    if let Some(stripe_config) = state.config.stripe.as_ref() {
        match create_checkout_session(stripe_config, payload).await {
            Ok(response) => Ok(Json(response)),
            Err(StripeError::ConfigError) => {
                eprintln!("Stripe configuration error.");
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Stripe configuration error on server.".to_string()))
            }
            Err(StripeError::RequestError(e)) => {
                eprintln!("Stripe Reqwest Error: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to communicate with payment provider.".to_string()))
            }
            Err(StripeError::ParseError(e)) => {
                eprintln!("Stripe Parse Error: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to understand payment provider response.".to_string()))
            }
            Err(StripeError::ApiError { status_code, message }) => {
                eprintln!("Stripe API Error ({}): {}", status_code, message);
                Err((StatusCode::from_u16(status_code).unwrap_or(StatusCode::BAD_GATEWAY), message))
            }
            Err(StripeError::InternalError(msg)) => {
                eprintln!("Stripe Internal Logic Error: {}", msg);
                Err((StatusCode::INTERNAL_SERVER_ERROR, msg))
            }
            // These errors are for webhook handling, not checkout session creation
            Err(StripeError::WebhookSignatureError(_)) | Err(StripeError::WebhookProcessingError(_)) => {
                eprintln!("Unexpected webhook error during checkout session creation.");
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Unexpected server error".to_string()))
            }
        }
    } else {
        Err((StatusCode::INTERNAL_SERVER_ERROR, "Stripe configuration not loaded.".to_string()))
    }
}

// --- Placeholder for Stripe Webhook Handler ---
// This is where Stripe sends server-to-server notifications.
// You need to configure this endpoint URL in your Stripe Dashboard.
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/stripe/webhook", // Path relative to /api
    // request_body = StripeEvent, // Describe the event payload from logic.rs
    responses(
        (status = 200, description = "Webhook received and acknowledged"),
        (status = 400, description = "Bad Request (e.g., invalid signature, bad payload)"),
        (status = 500, description = "Internal Server Error processing webhook")
    ),
    tag = "Stripe Webhooks"
))]
pub async fn stripe_webhook_handler(
    State(state): State<Arc<StripeState>>, // Needs AppConfig for webhook secret (via env usually)
    headers: HeaderMap,
    body: String, // Raw body for signature verification
) -> Response {
    println!("Received Stripe webhook...");

    if !state.config.use_stripe { // Check if Stripe is enabled
        return (StatusCode::SERVICE_UNAVAILABLE, "Stripe service disabled.").into_response();
    }

    // --- Verify Signature ---
    // Get webhook signing secret from environment variables
    let webhook_secret = match std::env::var("STRIPE_WEBHOOK_SECRET") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("🚨 STRIPE_WEBHOOK_SECRET environment variable not set!");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Get the 'Stripe-Signature' header from the request
    let sig_header = headers.get("Stripe-Signature").and_then(|h| h.to_str().ok());

    // Call the verification function from logic.rs
    match verify_stripe_signature(&body.as_bytes(), sig_header, &webhook_secret) {
        Ok(_) => {
            println!("✅ Stripe webhook signature verified.");
        }
        Err(e) => {
            eprintln!("Stripe webhook signature verification failed: {:?}", e);
            // Return 400 Bad Request for signature errors
            return (StatusCode::BAD_REQUEST, format!("Invalid signature: {}", e)).into_response();
        }
    }

    // --- Process Payload ---
    // Deserialize the raw body into StripeEvent AFTER signature verification
    let event: StripeEvent = match serde_json::from_str(&body) {
        Ok(ev) => ev,
        Err(e) => {
            eprintln!("Failed to deserialize Stripe webhook event: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid payload format".to_string()).into_response();
        }
    };

    // Call the processing logic from logic.rs
    match process_stripe_webhook(event).await {
        Ok(()) => {
            println!("Stripe webhook processed successfully.");
            StatusCode::OK.into_response() // Return 200 OK to Stripe
        }
        Err(e) => {
            eprintln!("Error processing Stripe webhook: {}", e);
            // Return 500 for internal processing errors
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Webhook processing error: {}", e)).into_response()
        }
    }
}


// --- Redirect Handlers (Client-Side) ---
// These are the success_url and cancel_url you provide to Stripe

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams, ToSchema))]
pub struct StripeRedirectQuery {
    // Stripe appends the checkout session ID to the success URL
    // e.g., ?session_id={CHECKOUT_SESSION_ID}
    #[cfg_attr(feature = "openapi", param(example = "cs_test_a1..."))]
    pub session_id: Option<String>,
}

#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/stripe/checkout-success", // Path relative to /api
    params(StripeRedirectQuery),
    responses((status = 200, description = "Checkout success page", content_type = "text/html")),
    tag = "Stripe Redirects"
))]
pub async fn stripe_checkout_success_handler(
    Query(params): Query<StripeRedirectQuery>
) -> Html<&'static str> {
    println!("User redirected to Stripe success URL. Session ID: {:?}", params.session_id);
    // TODO: You can use the session_id to fetch session details from Stripe API
    // and display more specific information to the user.
    // For now, just a generic success message.
    Html("<h1>Payment Successful!</h1><p>Thank you for your payment. Your order is being processed.</p><a href='/'>Back to Home</a>")
}

#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/stripe/checkout-cancel", // Path relative to /api
    params(StripeRedirectQuery), // Stripe might not add params to cancel URL
    responses((status = 200, description = "Checkout cancellation page", content_type = "text/html")),
    tag = "Stripe Redirects"
))]
pub async fn stripe_checkout_cancel_handler(
    Query(params): Query<StripeRedirectQuery>
) -> Html<&'static str> {
    println!("User redirected to Stripe cancel URL. Session ID: {:?}", params.session_id);
    Html("<h1>Payment Cancelled</h1><p>Your payment process was cancelled. You have not been charged.</p><a href='/'>Back to Home</a>")
}
