// --- File: crates/connectify_gcal/src/lib.rs ---
// Declare modules within this crate
pub mod auth;
pub mod doc;
pub mod handlers;
pub mod logic;
pub mod routes;
pub mod service;
mod test;
#[cfg(test)]
mod logic_test;
#[cfg(test)]
mod handlers_test;
#[cfg(test)]
mod routes_test;
#[cfg(test)]
mod auth_test;
#[cfg(test)]
mod logic_proptest;
