//! Firebase Cloud Messaging integration for Connectify
//!
//! This crate provides functionality to send push notifications
//! using Firebase Cloud Messaging (FCM) HTTP v1 API.
//!
//! # Features
//!
//! - Authentication with Firebase using service account credentials
//! - Sending push notifications to specific devices using FCM tokens
//! - Sending push notifications to topics
//! - Support for notification payload (title and body)
//! - Support for custom data payload
//! - Integration with Axum for HTTP API endpoints
//! - OpenAPI/Swagger documentation (with the `openapi` feature)
//!
//! # Usage
//!
//! Add the crate to your dependencies:
//!
//! ```toml
//! [dependencies]
//! connectify-firebase = { version = "0.1.0" }
//! ```
//!
//! To enable OpenAPI documentation:
//!
//! ```toml
//! [dependencies]
//! connectify-firebase = { version = "0.1.0", features = ["openapi"] }
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use connectify_config::AppConfig;
//! use connectify_firebase::routes;
//! use std::sync::Arc;
//!
//! async fn setup_app() {
//!     let config = Arc::new(AppConfig::default());
//!     let app = routes(config);
//!     // Use the app with your Axum server
//! }
//! ```
//!
//! # API Endpoints
//!
//! - `POST /send-notification` - Send a push notification to a device or topic

pub mod auth;
pub mod client;
#[cfg(feature = "openapi")]
pub mod doc;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod repository_factory;
pub mod routes;
pub mod service;

// Re-export the routes function to be used by the main backend service
pub use routes::routes;
// Re-export the service factory
pub use service::FirebaseServiceFactory;
// Re-export the repository factory
pub use repository_factory::DeviceRegistrationRepositoryFactory;

#[cfg(feature = "openapi")]
pub mod openapi {
    pub use crate::doc::FirebaseApiDoc;
}
