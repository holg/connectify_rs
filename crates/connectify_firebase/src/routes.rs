use axum::{routing::post, Router};
use connectify_config::AppConfig;
use std::sync::Arc;
use tracing::info;

use crate::handlers::{
    register_device_handler, send_notification_handler, send_notification_to_user_handler,
    FirebaseState,
};
use crate::service::FirebaseServiceFactory;

/// Create Firebase routes for the API
///
/// This function creates a router with the Firebase Cloud Messaging API endpoints.
/// It initializes a Firebase service factory with the provided configuration
/// and sets up the necessary routes for sending push notifications and registering devices.
///
/// Note: Database initialization is now performed at application startup in the ConnectifyServiceFactory.
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
    // Create the service factory
    let service_factory = FirebaseServiceFactory::new(config.clone());

    // Get a client with the repository set if database is enabled
    let firebase_client = service_factory.client();

    info!("Firebase routes initialized");

    let state = Arc::new(FirebaseState {
        client: Arc::new(firebase_client),
    });

    Router::new()
        .route(
            "/firebase/send-notification",
            post(send_notification_handler),
        )
        .route("/firebase/register-device", post(register_device_handler))
        .route(
            "/firebase/send-notification-to-user",
            post(send_notification_to_user_handler),
        )
        .with_state(state)
}
