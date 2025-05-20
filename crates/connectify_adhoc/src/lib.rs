// --- File: crates/connectify_adhoc/src/lib.rs ---
#[cfg(feature = "openapi")]
pub mod doc;
pub mod handlers;
pub mod logic;
pub mod routes;

pub use handlers::AdhocState;
pub use routes::routes; // State for this crate's handlers
