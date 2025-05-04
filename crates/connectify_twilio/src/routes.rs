// --- File: crates/connectify_twilio/src/routes.rs ---
use axum::{routing::get, Router};
use std::sync::Arc;
// Import the handler function from the sibling module
use crate::twilio_token::generate_token;
use connectify_config::AppConfig;

/// Creates a router containing all routes for the Twilio feature.
pub fn routes(config: Arc<AppConfig>) -> Router {
    Router::new()
        .route("/generate-token", get(generate_token))
        .with_state(config)
    // Add any other Twilio-specific routes here later
}
