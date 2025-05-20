// --- File: crates/connectify_common/src/lib.rs ---

// Declare modules within this crate
pub mod error; // Error handling
pub mod features;
pub mod handlers; // HTTP request handlers
pub mod http; // HTTP utilities
pub mod logging; // Logging utilities
pub mod logic; // Core business logic
pub mod models; // Data structures and models
pub mod routes; // Route definitions
pub mod services; // Service abstractions // Feature flag handling

// Re-export the routes function to be used by the main backend service
pub use routes::routes;

// Re-export error types and utilities for easier access
pub use error::{
    config_error, conflict, external_service_error, internal_error, not_found, validation_error,
    ConnectifyError, Context, HttpStatusCode,
};

// Re-export HTTP utilities for easier access
pub use http::{
    client::{create_client, delete, get, patch, post, put, HTTP_CLIENT},
    handle_json_result, handle_result, map_error, map_json_error, IntoHttpResponse,
};

// Re-export logging utilities for easier access
pub use logging::{
    debug_log, error_log, info_log, init, init_with_level, log_error, log_result, trace_log,
    warn_log,
};

// Re-export feature flag handling utilities for easier access
pub use features::is_feature_enabled;

// Conditionally re-export feature-specific functions
#[cfg(feature = "gcal")]
pub use features::is_gcal_enabled;

#[cfg(feature = "stripe")]
pub use features::is_stripe_enabled;

#[cfg(feature = "twilio")]
pub use features::is_twilio_enabled;

#[cfg(feature = "payrexx")]
pub use features::is_payrexx_enabled;

#[cfg(feature = "fulfillment")]
pub use features::is_fulfillment_enabled;

// This crate provides common functionality that can be used across the application.
// It includes shared models, logic, handlers, routes, error handling, and HTTP utilities.
