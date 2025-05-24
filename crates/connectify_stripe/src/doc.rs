// --- File: crates/connectify_stripe/src/doc.rs ---
#![allow(dead_code)]
#![cfg(feature = "openapi")]
use utoipa::OpenApi;
// Import all relevant schemas from logic.rs and handlers.rs
use crate::handlers::{GetSessionDetailsQuery, StripeRedirectQuery};
use crate::logic::{
    CreateCheckoutSessionRequest, CreateCheckoutSessionResponse, ListSessionsAdminQuery,
    ListSessionsAdminResponse, StripeCheckoutSessionData, StripeCheckoutSessionObject,
    StripeCustomerDetails, StripeEvent, StripeEventData, StripeListObject,
};
#[utoipa::path(
    post,
    path = "/stripe/create-checkout-session", // Path relative to /api
    request_body(content = CreateCheckoutSessionRequest, example = json!({
        "product_name_override": "Premium Subscription",
        "amount_override": 2500, // e.g. 25.00 USD
        "currency_override": "USD",
        "client_reference_id": "my_order_ref_12345",
            "fulfillment_type": "gcal_booking", // Example fulfillment type
            "fulfillment_data": { // Example data for gcal_booking
                "start_time": "2025-08-01T14:00:00Z",
                "end_time": "2025-08-01T15:00:00Z",
                "summary": "Consultation (via Stripe)",
                "description": "Project discussion after payment.",
                "user_id_for_fulfillment": "user_abc_789" // Example, if needed by fulfillment
            },
            "metadata": { // Optional additional Stripe metadata
                "internal_tracking_id": "track_this_order_abc"
            }
    })),
    responses(
        (status = 200, description = "Stripe Checkout Session created successfully", body = CreateCheckoutSessionResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error or Stripe API error")
    ),
    tag = "Stripe"
)]
fn doc_create_checkout_session_handler() {}

#[utoipa::path(
    post,
    path = "/stripe/webhook", // Path relative to /api
    request_body = StripeEvent, // Main event object from logic.rs
    responses(
        (status = 200, description = "Webhook received and acknowledged"),
        (status = 400, description = "Bad Request (e.g., invalid signature, bad payload)"),
        (status = 500, description = "Internal Server Error processing webhook")
    ),
    tag = "Stripe Webhooks"
)]
fn doc_stripe_webhook_handler() {}

#[utoipa::path(
    get,
    path = "/stripe/checkout-success", // Path relative to /api
    params(StripeRedirectQuery), // Use the struct defined in handlers.rs
    responses((status = 200, description = "Checkout success page", content_type = "text/html")),
    tag = "Stripe Redirects"
)]
fn doc_stripe_checkout_success_handler() {}

#[utoipa::path(
    get,
    path = "/stripe/checkout-cancel", // Path relative to /api
    params(StripeRedirectQuery),
    responses((status = 200, description = "Checkout cancellation page", content_type = "text/html")),
    tag = "Stripe Redirects"
)]
fn doc_stripe_checkout_cancel_handler() {}

// --- Doc for Get Checkout Session Details (Public) ---
#[utoipa::path(
    get,
    path = "/stripe/order-confirmation-details", // Path relative to /api
    params(("session_id" = String, Query, description = "The ID of the Stripe checkout")), // Use full path if ambiguous
    // params(GetSessionDetailsQuery),
    responses(
        (status = 200, description = "Successfully retrieved checkout session details", body = StripeCheckoutSessionData),
        (status = 404, description = "Session not found or payment not completed"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Stripe"
)]
fn doc_get_checkout_session_details_handler() {}

// --- ADDED: Doc for Admin List Checkout Sessions ---
#[utoipa::path(
    get,
    path = "/admin/stripe/sessions", // Path relative to /api
    params(
        ("limit" = Option<u8>, Query, description = "A limit on the number of objects to be returned. Limit can range between 1 and 100, and the default is 10.", example = 10),
        ("starting_after" = Option<String>, Query, description = "A cursor for use in pagination. `starting_after` is an object ID that defines your place in the list.", example = "cs_test_a1b2c3..."),
        ("ending_before" = Option<String>, Query, description = "A cursor for use in pagination. `ending_before` is an object ID that defines your place in the list.", example = "cs_test_z0y9x8...")
        // Add other filter params from ListSessionsAdminQuery here if needed, e.g.:
        // ("customer" = Option<String>, Query, description = "Only return sessions for the given customer ID."),
        // ("payment_intent" = Option<String>, Query, description = "Only return sessions for the given PaymentIntent ID.")
    ),
    responses(
        (status = 200, description = "A list of Stripe Checkout Sessions", body = ListSessionsAdminResponse),
        (status = 500, description = "Internal server error or Stripe API error")
    ),
    // TODO: Add security definition once admin auth is implemented
    // security(("admin_auth" = [])), 
    tag = "Stripe Admin"
)]
fn doc_admin_list_checkout_sessions_handler() {}
#[derive(OpenApi)]
#[openapi(
    paths(
        doc_create_checkout_session_handler,
        doc_stripe_webhook_handler,
        doc_stripe_checkout_success_handler,
        doc_stripe_checkout_cancel_handler,
        doc_get_checkout_session_details_handler,
        doc_admin_list_checkout_sessions_handler
    ),
    components(
        schemas(
            CreateCheckoutSessionRequest, CreateCheckoutSessionResponse,
            StripeEvent, StripeEventData, StripeCheckoutSessionObject, StripeCustomerDetails,
            StripeRedirectQuery,
            crate::handlers::GetSessionDetailsQuery, // Use full path if ambiguous
            StripeCheckoutSessionData, // Response for single session details
            ListSessionsAdminQuery,    //  query schema for admin list
            ListSessionsAdminResponse, // response schema for admin list
            StripeListObject<StripeCheckoutSessionData>,// Ensure generic list object is in schema if used directly
            GetSessionDetailsQuery
        )
    ),
    tags(
        (name = "Stripe", description = "Stripe Payment Integration API"),
        (name = "Stripe Webhooks", description = "Stripe Server-to-Server Webhooks"), // ADDED tag
        (name = "Stripe Redirects", description = "User-facing redirect pages for Stripe Checkout") // ADDED tag
    )
)]
pub struct StripeApiDoc;
