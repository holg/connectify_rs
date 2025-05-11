// --- File: crates/connectify_stripe/src/routes.rs ---

use crate::handlers::{
    create_checkout_session_handler,
    stripe_checkout_cancel_handler,
    stripe_checkout_success_handler,
    stripe_webhook_handler,
    get_checkout_session_details_handler,
    admin_get_checkout_session_details_handler,
    admin_list_checkout_sessions_handler,
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
        .route("/stripe/webhook", post(stripe_webhook_handler))
        .route("/stripe/checkout-success", get(stripe_checkout_success_handler))
        .route("/stripe/checkout-cancel", get(stripe_checkout_cancel_handler))
        .route("/stripe/order-confirmation-details", get(get_checkout_session_details_handler))
        .route("/admin/stripe/order-details", get(admin_get_checkout_session_details_handler))
        .route("/admin/stripe/sessions", get(admin_list_checkout_sessions_handler))
        .with_state(stripe_state)
}
