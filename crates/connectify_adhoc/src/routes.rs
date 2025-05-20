// --- File: crates/connectify_adhoc/src/routes.rs ---
use axum::{routing::post, Router};
use std::sync::Arc;
use connectify_config::AppConfig;
use crate::handlers::{AdhocState, initiate_adhoc_session_handler};

// Define two separate functions based on the feature flag
#[cfg(feature = "gcal")]
pub fn routes(
    config: Arc<AppConfig>,
    gcal_hub_option: Option<Arc<connectify_gcal::auth::HubType>>,
) -> Router {
    let adhoc_state = Arc::new(AdhocState {
        config,
        gcal_hub: gcal_hub_option,
    });

    Router::new()
        .route("/adhoc/initiate-session", post(initiate_adhoc_session_handler))
        .with_state(adhoc_state)
}

#[cfg(not(feature = "gcal"))]
pub fn routes(
    config: Arc<AppConfig>,
) -> Router {
    let adhoc_state = Arc::new(AdhocState {
        config,
    });

    Router::new()
        .route("/adhoc/initiate-session", post(initiate_adhoc_session_handler))
        .with_state(adhoc_state)
}