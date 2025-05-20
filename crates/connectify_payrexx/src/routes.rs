// --- File: crates/connectify_payrexx/src/routes.rs ---

use axum::{
    routing::{get, post},
    Router,
};
use connectify_config::AppConfig;
use std::sync::Arc; // Need AppConfig for state
                    // Removed: use reqwest::Client; // No longer needed as parameter
                    // Import the handler function and the specific state struct it needs
use crate::handlers::{
    create_gateway_handler, payrexx_cancel_handler, payrexx_failure_handler,
    payrexx_success_handler, payrexx_webhook_handler, PayrexxState,
};

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
) -> Router {
    // Return concrete Router, state applied internally

    // Create the specific state needed for Payrexx handlers
    // It only needs AppConfig now, as the client is static in logic.rs
    let payrexx_state = Arc::new(PayrexxState {
        config, // Move Arc<AppConfig> into state
                // Removed http_client field
    });

    Router::new()
        // API endpoint called by our frontend to create the payment link
        .route("/payrexx/create-gateway", post(create_gateway_handler))
        // API endpoint called by Payrexx SERVER for webhook notifications
        .route("/payrexx/webhook", post(payrexx_webhook_handler))
        // Routes for USER BROWSER redirects (typically GET)
        .route("/payrexx/webhook/success", get(payrexx_success_handler)) // <-- Use GET and correct handler
        .route("/payrexx/webhook/failure", get(payrexx_failure_handler)) // <-- Use GET and correct handler
        .route("/payrexx/webhook/cancel", get(payrexx_cancel_handler)) // <-- Add route for cancel handler
        .with_state(payrexx_state) // Apply the specific state to this router fragment
}
