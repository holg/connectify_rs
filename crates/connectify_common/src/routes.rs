// --- File: crates/connectify_common/src/routes.rs ---

// This file will contain route definitions that are common across the application.
// Examples might include:
// - Health check routes
// - Common API endpoints
// - Shared route configuration

use axum::{routing::get, Router};

// Currently empty as no common routes have been defined yet.

/// Creates a router containing common routes that can be used across the application.
///
/// # Returns
/// A router configured with common routes.
pub fn routes() -> Router {
    Router::new().route("/common", get(|| async { "Common routes" }))
    // Add common routes here when needed
}
