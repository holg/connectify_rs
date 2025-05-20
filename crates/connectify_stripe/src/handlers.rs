// --- File: crates/connectify_stripe/src/handlers.rs ---
// use crate::error::StripeError;
use crate::logic::{
    create_checkout_session, get_checkout_session_details, list_checkout_sessions_admin,
    process_stripe_webhook, verify_stripe_signature, CreateCheckoutSessionRequest,
    CreateCheckoutSessionResponse, ListSessionsAdminQuery, ListSessionsAdminResponse,
    StripeCheckoutSessionData, StripeEvent,
};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json, Redirect, Response},
};
use connectify_common::{
    config_error,
    // external_service_error,
    // validation_error,
    // not_found,
    // IntoHttpResponse,
    // handle_json_result,
    // map_error,
    map_json_error,
    ConnectifyError,
};
use connectify_config::AppConfig;
use connectify_config::StripeConfig;
use serde::Deserialize;
use std::{sync::Arc};
use tracing::info;
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
) -> Result<Json<CreateCheckoutSessionResponse>, Response> {
    if !state.config.use_stripe {
        return Err(
            ConnectifyError::ConfigError("Stripe service is disabled".to_string()).into_response(),
        );
    }

    if let Some(stripe_config) = state.config.stripe.as_ref() {
        // Use map_json_error to convert StripeError to ConnectifyError and then to a Response
        map_json_error(
            create_checkout_session(stripe_config, payload).await,
            |err| err.into(), // Convert StripeError to ConnectifyError using the From implementation
        ).map_err(|boxed| *boxed)

    } else {
        Err(config_error("Stripe configuration not loaded").into_response())
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
    info!("Received Stripe webhook...");

    if !state.config.use_stripe {
        // Check if Stripe is enabled
        return ConnectifyError::ConfigError("Stripe service disabled".to_string()).into_response();
    }

    // --- Verify Signature ---
    // Get webhook signing secret from environment variables
    let webhook_secret = match std::env::var("STRIPE_WEBHOOK_SECRET") {
        Ok(s) => s,
        Err(_) => {
            info!("🚨 STRIPE_WEBHOOK_SECRET environment variable not set!");
            return ConnectifyError::ConfigError(
                "STRIPE_WEBHOOK_SECRET environment variable not set".to_string(),
            )
            .into_response();
        }
    };

    // Get the 'Stripe-Signature' header from the request
    let sig_header = headers
        .get("Stripe-Signature")
        .and_then(|h| h.to_str().ok());

    // Call the verification function from logic.rs
    if let Err(e) = verify_stripe_signature(body.as_bytes(), sig_header, &webhook_secret) {
        info!("Stripe webhook signature verification failed: {:?}", e);
        // Return 400 Bad Request for signature errors
        return ConnectifyError::from(e).into_response();
    }

    info!("✅ Stripe webhook signature verified.");

    // --- Process Payload ---
    // Deserialize the raw body into StripeEvent AFTER signature verification
    let event: StripeEvent = match serde_json::from_str(&body) {
        Ok(ev) => ev,
        Err(e) => {
            info!("Failed to deserialize Stripe webhook event: {}", e);
            return ConnectifyError::ParseError(format!("Invalid webhook payload: {}", e))
                .into_response();
        }
    };

    let app_config = state.config.clone(); // Clone the AppConfig for processing the webhook

    // Call the processing logic from logic.rs
    match process_stripe_webhook(event, app_config.clone()).await {
        Ok(()) => {
            info!("Stripe webhook processed successfully.");
            StatusCode::OK.into_response() // Return 200 OK to Stripe
        }
        Err(e) => {
            info!("Error processing Stripe webhook: {}", e);
            // Convert StripeError to ConnectifyError and then to a Response
            ConnectifyError::from(e).into_response()
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
    info!(
        "User redirected to Stripe success URL. Session ID: {:?}",
        params.session_id
    );
    let session_id = params
        .session_id
        .unwrap_or_else(|| "unknown_session".to_string());
    info!(
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
    info!(
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
) -> Result<Json<StripeCheckoutSessionData>, Response> {
    if !state.config.use_stripe || state.config.stripe.is_none() {
        return Err(ConnectifyError::ConfigError(
            "Stripe service not configured or disabled".to_string(),
        )
        .into_response());
    }

    // Use map_json_error to convert StripeError to ConnectifyError and then to a Response
    map_json_error(
        get_checkout_session_details(&query.session_id).await,
        |err| {
            info!("Error retrieving Stripe session details: {}", err);
            err.into() // Convert StripeError to ConnectifyError using the From implementation
        },
    ).map_err(|boxed| *boxed)

}
#[axum::debug_handler]
// Add OpenAPI docs if needed for admin routes
pub async fn admin_get_checkout_session_details_handler(
    State(state): State<Arc<StripeState>>,
    Query(query): Query<GetSessionDetailsQuery>, // Assuming same query params
) -> Result<Json<StripeCheckoutSessionData>, Response> {
    info!(
        "[ADMIN] Request to get Stripe session details: {:?}",
        query.session_id
    );
    // TODO: Implement admin-specific authorization/logging if needed

    if !state.config.use_stripe || state.config.stripe.is_none() {
        return Err(ConnectifyError::ConfigError(
            "Stripe service not configured or disabled".to_string(),
        )
        .into_response());
    }

    // Use map_json_error to convert StripeError to ConnectifyError and then to a Response
    map_json_error(
        crate::logic::get_checkout_session_details(&query.session_id).await,
        |err| {
            info!("[ADMIN] Error retrieving Stripe session details: {}", err);
            err.into() // Convert StripeError to ConnectifyError using the From implementation
        },
    ).map_err(|boxed| *boxed)

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
) -> Result<Json<ListSessionsAdminResponse>, Response> {
    info!(
        "[ADMIN] Listing Stripe Checkout Sessions. Params: {:?}",
        query_params
    );
    // TODO: Implement admin-specific authorization here

    if !state.config.use_stripe || state.config.stripe.is_none() {
        return Err(ConnectifyError::ConfigError(
            "Stripe service not configured or disabled".to_string(),
        )
        .into_response());
    }

    // Use map_json_error to convert StripeError to ConnectifyError and then to a Response
    map_json_error(list_checkout_sessions_admin(query_params).await, |err| {
        info!("[ADMIN] Error listing Stripe sessions: {}", err);
        err.into() // Convert StripeError to ConnectifyError using the From implementation
    }).map_err(|boxed| *boxed)

}
