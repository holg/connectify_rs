// --- File: crates/connectify_fulfillment/src/lib.rs ---

// Declare modules within this crate
pub mod auth;     // For secure endpoint authentication
pub mod handlers; // Axum handlers for fulfillment tasks
pub mod logic;    // Core fulfillment logic (calling GCal, Twilio, etc.)
pub mod routes;   // Axum router definition for this crate
#[cfg(feature = "openapi")]
pub mod doc;
// OpenAPI documentation specific to fulfillment API

// Re-export the routes function to be used by the main backend service
pub use routes::routes;

// Re-export state if main.rs needs to construct it (following GCal/Payrexx pattern)
pub use handlers::FulfillmentState;

// Potentially re-export request/response structs if they are part of a public API
// defined by this crate (e.g., if other services call this crate's API directly).
// For internal fulfillment triggered by webhooks, this might not be necessary.
