// --- File: crates/connectify_payrexx/src/routes.rs ---

use axum::{
    routing::post,
    Router,
};
use std::sync::Arc;
use connectify_config::AppConfig; // Need AppConfig for state
// Removed: use reqwest::Client; // No longer needed as parameter
// Import the handler function and the specific state struct it needs
use crate::handlers::{create_gateway_handler, PayrexxState, payrexx_webhook_handler};

/// Creates a router containing all routes for the Payrexx feature.
/// Initializes and applies the necessary PayrexxState.
///
/// # Arguments
/// * `config` - Shared application configuration (`Arc<AppConfig>`).
///
/// # Returns
/// An Axum Router configured with Payrexx routes and state.
// Function now only takes AppConfig
pub fn routes(
    config: Arc<AppConfig>,
    // Removed: http_client: Arc<Client>
) -> Router { // Return concrete Router, state applied internally

    // Create the specific state needed for Payrexx handlers
    // It only needs AppConfig now, as the client is static in logic.rs
    let payrexx_state = Arc::new(PayrexxState {
        config, // Move Arc<AppConfig> into state
        // Removed http_client field
    });

    Router::new()
        // Register the handler for creating a payment gateway
        .route("/payrexx/create-gateway", post(create_gateway_handler))
        .route("/payrexx/webhook", post(payrexx_webhook_handler))
        // Add other Payrexx routes here later (e.g., webhook handler)
        .with_state(payrexx_state) // Apply the specific state to this router fragment
}
