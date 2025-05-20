// --- File: crates/connectify_stripe/src/lib.rs ---

pub mod logic;
pub mod handlers;
pub mod routes;
pub mod doc;
pub mod error;
pub mod service;

// Re-export for main backend
pub use routes::routes;
pub use logic::{CreateCheckoutSessionRequest, CreateCheckoutSessionResponse}; // For OpenAPI
pub use handlers::StripeState; // If main needs to construct it (not with current routes.rs pattern)
pub use error::StripeError; // Re-export the error type
pub use service::StripePaymentService; // Re-export the payment service
