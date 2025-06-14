// --- File: crates/connectify_gcal/src/lib.rs ---
// Declare modules within this crate
pub mod auth;
#[cfg(test)]
mod auth_test;
pub mod doc;
pub mod handlers;
#[cfg(test)]
mod handlers_test;
pub mod logic;
#[cfg(test)]
mod logic_midnight_test;
#[cfg(test)]
mod logic_proptest;
#[cfg(test)]
mod logic_test;
pub mod routes;
pub mod service;
mod test;
