// --- File: crates/connectify_fulfillment/src/routes.rs ---

use crate::auth::{fulfillment_auth_middleware, FulfillmentAuthState};
use crate::handlers::FulfillmentState;

#[allow(unused_imports)]
use axum::{middleware, routing::post, Router};
use connectify_config::AppConfig;
use std::sync::Arc;
#[allow(unused_imports)]
// the warning is due to unused imports not recognized by rustfmt, but for features
use tracing::info;

// Conditionally import handlers based on features enabled for this crate
#[cfg(feature = "gcal")]
use crate::handlers::{handle_adhoc_gcal_twilio_fulfillment, handle_gcal_booking_fulfillment};
// #[cfg(feature = "twilio")]
// use crate::handlers::handle_twilio_fulfillment; // Example for another feature

/// Creates a router containing all routes for the fulfillment service.
pub fn routes(
    config: Arc<AppConfig>,
    #[cfg(feature = "gcal")] gcal_state_option: Option<Arc<connectify_gcal::handlers::GcalState>>,
) -> Router {
    let handler_state = Arc::new(FulfillmentState {
        config: config.clone(),
        #[cfg(feature = "gcal")]
        gcal_state_for_fulfillment: gcal_state_option,
    });

    let auth_middleware_state = Arc::new(FulfillmentAuthState {
        config: config.clone(),
    });

    #[allow(unused_mut)]
    let mut fulfillment_api_router = Router::new();

    // Conditionally add the GCal fulfillment routes
    #[cfg(feature = "gcal")]
    {
        // Check runtime config flags before adding routes
        if config.use_gcal && config.gcal.is_some() {
            // For standard GCal booking
            info!("💡 Fulfillment: Adding /fulfill/gcal-booking route.");
            fulfillment_api_router = fulfillment_api_router.route(
                "/fulfill/gcal-booking",
                post(handle_gcal_booking_fulfillment),
            );
        }
        // For adhoc GCal booking (which also relies on GCal config and use_adhoc_sessions flag)
        if config.use_adhoc
            && config
                .adhoc_settings
                .as_ref()
                .map_or(false, |s| s.admin_enabled)
            && config.gcal.is_some()
        {
            info!("💡 Fulfillment: Adding /fulfill/adhoc-gcal-twilio route.");
            fulfillment_api_router = fulfillment_api_router.route(
                "/fulfill/adhoc-gcal-twilio", // New route for adhoc
                post(handle_adhoc_gcal_twilio_fulfillment),
            );
        }
    }

    // TODO: Add other fulfillment routes here (e.g., for Twilio specific fulfillment)
    // #[cfg(feature = "twilio")]
    // {
    //     if config.use_twilio && config.twilio.is_some() {
    //         // router = router.route("/fulfill/twilio-something", post(handle_twilio_something_fulfillment));
    //     }
    // }

    fulfillment_api_router
        .layer(middleware::from_fn_with_state(
            auth_middleware_state,
            fulfillment_auth_middleware::<axum::body::Body>,
        ))
        .with_state(handler_state)
}
