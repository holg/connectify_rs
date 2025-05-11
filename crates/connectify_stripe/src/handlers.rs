// --- File: crates/connectify_stripe/src/handlers.rs ---
use crate::logic::{
    create_checkout_session, get_checkout_session_details, list_checkout_sessions_admin,
    process_stripe_webhook, verify_stripe_signature, CreateCheckoutSessionRequest,
    CreateCheckoutSessionResponse, ListSessionsAdminQuery, ListSessionsAdminResponse,
    StripeCheckoutSessionData, StripeError, StripeEvent,
};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json, Redirect, Response},
};
use connectify_config::AppConfig;
use connectify_config::StripeConfig;
use serde::Deserialize;
use std::sync::Arc;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

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
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Stripe service is disabled.".to_string(),
        ));
    }

    if let Some(stripe_config) = state.config.stripe.as_ref() {
        match create_checkout_session(stripe_config, payload).await {
            Ok(response) => Ok(Json(response)),
            Err(StripeError::ConfigError) => {
                eprintln!("Stripe configuration error.");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Stripe configuration error on server.".to_string(),
                ))
            }
            Err(StripeError::RequestError(e)) => {
                eprintln!("Stripe Reqwest Error: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to communicate with payment provider.".to_string(),
                ))
            }
            Err(StripeError::ParseError(e)) => {
                eprintln!("Stripe Parse Error: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to understand payment provider response.".to_string(),
                ))
            }
            Err(StripeError::ApiError {
                status_code,
                message,
            }) => {
                eprintln!("Stripe API Error ({}): {}", status_code, message);
                Err((
                    StatusCode::from_u16(status_code).unwrap_or(StatusCode::BAD_GATEWAY),
                    message,
                ))
            }
            Err(StripeError::InternalError(msg)) => {
                eprintln!("Stripe Internal Logic Error: {}", msg);
                Err((StatusCode::INTERNAL_SERVER_ERROR, msg))
            }
            // These errors are for webhook handling, not checkout session creation
            Err(StripeError::WebhookSignatureError(_))
            | Err(StripeError::WebhookProcessingError(_)) => {
                eprintln!("Unexpected webhook error during checkout session creation.");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unexpected server error".to_string(),
                ))
            }
            Err(StripeError::FulfillmentError(msg)) => {
                eprintln!(
                    "Fulfillment error during checkout session creation: {}",
                    msg
                );
                Err((StatusCode::INTERNAL_SERVER_ERROR, msg))
            }
            Err(StripeError::MissingFulfillmentData) => {
                eprintln!("Missing fulfillment data during checkout session creation.");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Missing fulfillment data".to_string(),
                ))
            }
            Err(StripeError::SessionNotFoundOrNotPaid) => {
                eprintln!("Session not found or not paid during checkout session creation.");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Session not found or not paid".to_string(),
                ))
            }
        }
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Stripe configuration not loaded.".to_string(),
        ))
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

    if !state.config.use_stripe {
        // Check if Stripe is enabled
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
    let sig_header = headers
        .get("Stripe-Signature")
        .and_then(|h| h.to_str().ok());

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
            return (
                StatusCode::BAD_REQUEST,
                "Invalid payload format".to_string(),
            )
                .into_response();
        }
    };
    let app_config = state.config.clone(); // Clone the AppConfig for processing the webhook
                                           // Call the processing logic from logic.rs
    match process_stripe_webhook(event, app_config.clone()).await {
        Ok(()) => {
            println!("Stripe webhook processed successfully.");
            StatusCode::OK.into_response() // Return 200 OK to Stripe
        }
        Err(e) => {
            eprintln!("Error processing Stripe webhook: {}", e);
            // Return 500 for internal processing errors
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Webhook processing error: {}", e),
            )
                .into_response()
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
    State(state): State<Arc<StripeState>>,
    Query(params): Query<StripeRedirectQuery>,
) -> Redirect {
    println!(
        "User redirected to Stripe success URL. Session ID: {:?}",
        params.session_id
    );
    let session_id = params
        .session_id
        .unwrap_or_else(|| "unknown_session".to_string());
    println!(
        "User redirected to Stripe success. Session ID: {}",
        session_id
    );

    // Construct the URL for your frontend success page
    // This page will be responsible for fetching and displaying details.

    let frontend_success_url = format!(
        "{}?session_id={}",
        <std::option::Option<StripeConfig> as Clone>::clone(&state.config.stripe)
            .unwrap()
            .payment_success_url,
        session_id
    );

    // Perform a redirect
    Redirect::to(&frontend_success_url)
    // Html("<h1>Payment Successful!</h1><p>Thank you for your payment. Your order is being processed.</p><a href='/'>Back to Home</a>")
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
    Query(params): Query<StripeRedirectQuery>,
) -> Html<&'static str> {
    println!(
        "User redirected to Stripe cancel URL. Session ID: {:?}",
        params.session_id
    );
    Html("<h1>Payment Cancelled</h1><p>Your payment process was cancelled. You have not been charged.</p><a href='/'>Back to Home</a>")
}

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams, ToSchema))] // Added ToSchema
pub struct GetSessionDetailsQuery {
    #[cfg_attr(feature = "openapi", param(example = "cs_test_a1..."))]
    pub session_id: String, // Made session_id mandatory for this endpoint
}
/// Handler to retrieve details of a Stripe Checkout Session using its ID.
/// This is called by the frontend success page.
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/stripe/order-confirmation-details",
    params(GetSessionDetailsQuery),
    responses(
        (status = 200, description = "Checkout session details retrieved", body = StripeCheckoutSessionData),
        (status = 404, description = "Session not found or payment not completed"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Stripe"
))]
pub async fn get_checkout_session_details_handler(
    State(state): State<Arc<StripeState>>, // Needs state to check if Stripe is enabled/configured
    Query(query): Query<GetSessionDetailsQuery>,
) -> Result<Json<StripeCheckoutSessionData>, (StatusCode, String)> {
    if !state.config.use_stripe || state.config.stripe.is_none() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Stripe service not configured or disabled.".to_string(),
        ));
    }

    match get_checkout_session_details(&query.session_id).await {
        Ok(session_data) => Ok(Json(session_data)),
        Err(StripeError::SessionNotFoundOrNotPaid) => Err((
            StatusCode::NOT_FOUND,
            "Checkout session not found or payment not completed.".to_string(),
        )),
        Err(e) => {
            eprintln!("Error retrieving Stripe session details: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve session details: {}", e),
            ))
        }
    }
}
#[axum::debug_handler]
// Add OpenAPI docs if needed for admin routes
pub async fn admin_get_checkout_session_details_handler(
    State(state): State<Arc<StripeState>>,
    Query(query): Query<GetSessionDetailsQuery>, // Assuming same query params
) -> Result<Json<StripeCheckoutSessionData>, (StatusCode, String)> {
    println!(
        "[ADMIN] Request to get Stripe session details: {:?}",
        query.session_id
    );
    // TODO: Implement admin-specific authorization/logging if needed

    if !state.config.use_stripe || state.config.stripe.is_none() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Stripe service not configured or disabled.".to_string(),
        ));
    }

    match crate::logic::get_checkout_session_details(&query.session_id).await {
        Ok(session_data) => Ok(Json(session_data)),
        Err(StripeError::SessionNotFoundOrNotPaid) => Err((
            StatusCode::NOT_FOUND,
            "Checkout session not found or payment not completed.".to_string(),
        )),
        Err(e) => {
            eprintln!("[ADMIN] Error retrieving Stripe session details: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve session details: {}", e),
            ))
        }
    }
}

// --- NEW: Admin Handler to list Checkout Sessions ---
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/admin/stripe/sessions", // Path relative to /api
    params(ListSessionsAdminQuery), // Use the query struct from logic.rs
    responses(
        (status = 200, description = "List of Stripe Checkout Sessions", body = ListSessionsAdminResponse),
        (status = 500, description = "Internal server error or Stripe API error")
    ),
    // TODO: Add security definition if admin routes are protected
    // security(("admin_auth" = [])),
    tag = "Stripe Admin"
))]
pub async fn admin_list_checkout_sessions_handler(
    State(state): State<Arc<StripeState>>,
    Query(query_params): Query<ListSessionsAdminQuery>,
) -> Result<Json<ListSessionsAdminResponse>, (StatusCode, String)> {
    println!(
        "[ADMIN] Listing Stripe Checkout Sessions. Params: {:?}",
        query_params
    );
    // TODO: Implement admin-specific authorization here

    if !state.config.use_stripe || state.config.stripe.is_none() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Stripe service not configured or disabled.".to_string(),
        ));
    }

    match list_checkout_sessions_admin(query_params).await {
        Ok(list_response) => Ok(Json(list_response)),
        Err(e) => {
            eprintln!("[ADMIN] Error listing Stripe sessions: {}", e);
            // Map StripeError to a tuple for Axum response
            match e {
                StripeError::ConfigError => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Stripe configuration error.".to_string(),
                )),
                StripeError::RequestError(re) => Err((
                    StatusCode::BAD_GATEWAY,
                    format!("Stripe API request error: {}", re),
                )),
                StripeError::ApiError {
                    status_code,
                    message,
                } => Err((
                    StatusCode::from_u16(status_code).unwrap_or(StatusCode::BAD_GATEWAY),
                    message,
                )),
                StripeError::ParseError(pe) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse Stripe response: {}", pe),
                )),
                _ => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to list Stripe sessions.".to_string(),
                )),
            }
        }
    }
}
