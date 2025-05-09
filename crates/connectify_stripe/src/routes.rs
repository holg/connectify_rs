// --- File: crates/connectify_stripe/src/routes.rs ---

use crate::handlers::{
    create_checkout_session_handler,
    stripe_checkout_cancel_handler,
    stripe_checkout_success_handler,
    stripe_webhook_handler,
    StripeState,
};
use axum::{routing::{post, get}, Router};
use connectify_config::AppConfig;
use std::sync::Arc;

/// Creates a router containing all routes for the Stripe feature.
pub fn routes(config: Arc<AppConfig>) -> Router {
    let stripe_state = Arc::new(StripeState { config });

    Router::new()
        .route(
            "/stripe/create-checkout-session",
            post(create_checkout_session_handler),
        )
        // User-facing redirect endpoints (GET)
        .route("/stripe/webhook", post(stripe_webhook_handler))
        .route("/stripe/checkout-success", get(stripe_checkout_success_handler))
        .route("/stripe/checkout-cancel", get(stripe_checkout_cancel_handler))
        .with_state(stripe_state)
}
