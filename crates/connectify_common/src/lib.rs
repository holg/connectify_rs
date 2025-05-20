

// --- File: crates/connectify_common/src/lib.rs ---

// Declare modules within this crate
pub mod models;    // Data structures and models
pub mod logic;     // Core business logic
pub mod handlers;  // HTTP request handlers
pub mod routes;    // Route definitions
pub mod error;     // Error handling
pub mod http;      // HTTP utilities
pub mod services;  // Service abstractions
pub mod logging;   // Logging utilities
pub mod features;  // Feature flag handling

// Re-export the routes function to be used by the main backend service
pub use routes::routes;

// Re-export error types and utilities for easier access
pub use error::{
    ConnectifyError, 
    HttpStatusCode, 
    Context,
    config_error,
    validation_error,
    not_found,
    conflict,
    external_service_error,
    internal_error,
};

// Re-export HTTP utilities for easier access
pub use http::{
    IntoHttpResponse,
    handle_result,
    handle_json_result,
    map_error,
    map_json_error,
    client::{
        HTTP_CLIENT,
        create_client,
        get,
        post,
        put,
        delete,
        patch,
    },
};

// Re-export logging utilities for easier access
pub use logging::{
    init,
    init_with_level,
    trace_log,
    debug_log,
    info_log,
    warn_log,
    error_log,
    log_error,
    log_result,
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
