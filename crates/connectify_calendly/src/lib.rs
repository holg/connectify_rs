// --- File: crates/connectify_calendly/src/lib.rs ---
#![cfg_attr(feature = "calendly", allow(unused_imports))]

// Declare modules within this crate
pub mod handlers; // HTTP request handlers
pub mod logic; // Core business logic
pub mod models; // Data structures and models
pub mod routes; // Route definitions

// Re-export the storage module from the parent crate
// This is needed because the Calendly module uses the TokenStore trait
pub use crate::storage;
pub use crate::utils;

// Re-export the routes function to be used by the main backend service
pub use routes::routes;

// Re-export key types and functions that might be needed by other crates
pub use logic::{print_calendly_oauth_url, refresh_calendly_token};
pub use models::{CalendlyConfig, CalendlySlotsState};
