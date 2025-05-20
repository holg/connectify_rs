// --- File: crates/connectify_adhoc/src/lib.rs ---
pub mod handlers;
pub mod logic;
pub mod routes;
#[cfg(feature = "openapi")]
pub mod doc;

pub use routes::routes;
pub use handlers::AdhocState; // State for this crate's handlers
