use axum::{routing::post, Router};
use connectify_config::AppConfig;
use std::sync::Arc;

use crate::client::FirebaseClient;
use crate::handlers::{send_notification_handler, FirebaseState};

/// Create Firebase routes for the API
///
/// This function creates a router with the Firebase Cloud Messaging API endpoints.
/// It initializes a Firebase client with the provided configuration and sets up
/// the necessary routes for sending push notifications.
///
/// # Arguments
///
/// * `config` - A reference to the application configuration, which includes Firebase settings
///
/// # Returns
///
/// An Axum router with the Firebase API endpoints
///
pub fn routes(config: Arc<AppConfig>) -> Router {
    let firebase_config = config.firebase.clone().unwrap_or_default();
    let firebase_client = FirebaseClient::new(firebase_config);

    let state = Arc::new(FirebaseState {
        client: Arc::new(firebase_client),
    });

    Router::new()
        .route(
            "/firebase/send-notification",
            post(send_notification_handler),
        )
        .with_state(state)
}
