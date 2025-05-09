// --- File: crates/connectify_stripe/src/doc.rs ---
#![allow(dead_code)]
#![cfg(feature = "openapi")]

use utoipa::OpenApi;
// Import all relevant schemas from logic.rs and handlers.rs
use crate::logic::{
    CreateCheckoutSessionRequest, CreateCheckoutSessionResponse,
    StripeEvent, StripeEventData, StripeCheckoutSessionObject, StripeCustomerDetails
};
use crate::handlers::StripeRedirectQuery;
#[utoipa::path(
    post,
    path = "/stripe/create-checkout-session", // Path relative to /api
    request_body(content = CreateCheckoutSessionRequest, example = json!({
        "product_name_override": "Premium Subscription",
        "amount_override": 2500, // e.g. 25.00 USD
        "currency_override": "USD",
        "metadata": {
            "user_id": "user_xyz789",
            "order_id": "order_abc123"
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


#[derive(OpenApi)]
#[openapi(
    paths(
        doc_create_checkout_session_handler,
        doc_stripe_webhook_handler,
        doc_stripe_checkout_success_handler,
        doc_stripe_checkout_cancel_handler
    ),
    components(
        schemas(
            CreateCheckoutSessionRequest, CreateCheckoutSessionResponse,
            // Add all webhook related structs that are part of the StripeEvent schema
            StripeEvent, StripeEventData, StripeCheckoutSessionObject, StripeCustomerDetails,
            StripeRedirectQuery // ADDED redirect query schema
        )
    ),
    tags(
        (name = "Stripe", description = "Stripe Payment Integration API"),
        (name = "Stripe Webhooks", description = "Stripe Server-to-Server Webhooks"), // ADDED tag
        (name = "Stripe Redirects", description = "User-facing redirect pages for Stripe Checkout") // ADDED tag
    )
)]
pub struct StripeApiDoc;
