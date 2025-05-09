// --- File: crates/connectify_payrexx/src/lib.rs ---
// Declare modules within this crate
// pub mod auth; // Remove if not used
mod auth;
pub mod doc;
pub mod handlers;
pub mod logic;
pub mod routes;
// mod test; // Make test module private
pub use logic::{CreateGatewayRequest, CreateGatewayResponse};
pub use routes::routes;
