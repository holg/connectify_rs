// --- File: crates/connectify_payrexx/src/lib.rs ---

// Declare modules within this crate
// pub mod auth; // Remove if not used
pub mod doc;
pub mod handlers;
pub mod logic;
pub mod routes;
mod auth;
// mod test; // Make test module private
pub use routes::routes;
pub use logic::{CreateGatewayRequest, CreateGatewayResponse};
