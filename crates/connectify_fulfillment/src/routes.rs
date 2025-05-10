// --- File: crates/connectify_fulfillment/src/routes.rs ---

use axum::{
    body::Body as AxumBody,
    middleware,
    routing::post, // Assuming fulfillment tasks are triggered by POST
    Router,
};
use std::sync::Arc;
// Import the AppConfig from the config crate, as it will be part of the state
use connectify_config::AppConfig;
// Import the specific state struct for fulfillment handlers
use crate::handlers::FulfillmentState;
use crate::auth::{fulfillment_auth_middleware, FulfillmentAuthState};
#[cfg(feature = "gcal")]
use crate::handlers::{handle_gcal_booking_fulfillment, GcalState};

/// Creates a router containing all routes for the fulfillment service.
/// Initializes and applies the necessary FulfillmentState.
///
/// # Arguments
/// * `config`: The shared application configuration (`Arc<AppConfig>`).
/// * `gcal_state_option`: An Option containing the GcalState if GCal is enabled.
///                        This is needed if fulfillment directly interacts with GCal.
/// * `twilio_config_option`: An Option containing TwilioConfig if Twilio is enabled.
///                           This is needed if fulfillment directly interacts with Twilio.
///
/// # Returns
/// An Axum Router configured with fulfillment routes and state.
pub fn routes(
    config: Arc<AppConfig>,
    #[cfg(feature = "gcal")] gcal_state_option: Option<Arc<GcalState>>,
    // Pass other necessary states if fulfillment logic needs them directly
    // For example, if GCal booking happens here, we need GcalState
) -> Router {
    // Return concrete Router, state applied internally
    // Create the state for the fulfillment handlers
    let handler_state = Arc::new(FulfillmentState {
        config: config.clone(), // Clone Arc for handler state
        #[cfg(feature = "gcal")]
        gcal_state: gcal_state_option.clone(), // Store the GcalState if provided for handlers
    });
    // Create the specific state needed for Fulfillment handlers
    // This state will hold what the fulfillment logic needs,
    // which might include parts of AppConfig or other feature states.
    let fulfillment_state = Arc::new(FulfillmentState {
        config: config.clone(), // The main AppConfig
        #[cfg(feature = "gcal")]
        gcal_state: gcal_state_option, // Store the GcalState if provided
                // Add other states as needed, e.g., for Twilio
    });
    // Create the state for the authentication middleware
    let auth_middleware_state = Arc::new(FulfillmentAuthState {
        config: config.clone(), // Clone Arc for auth middleware state
    });
    // Define the core fulfillment API routes
    let mut fulfillment_api_router = Router::new();

    #[cfg(feature = "gcal")]
    {

        // Ensure the main config also enables gcal for runtime check
        if config.use_gcal && config.gcal.is_some() {
            println!("💡 Fulfillment: GCal feature is enabled, adding /fulfill/gcal-booking route.");
            fulfillment_api_router = fulfillment_api_router.route(
                "/fulfill/gcal-booking",
                post(handle_gcal_booking_fulfillment)
            );
        }
    }
    // Apply the authentication middleware to all routes in fulfillment_api_router,
    // then apply the state for the handlers themselves.
    fulfillment_api_router
        .layer(middleware::from_fn_with_state(auth_middleware_state, fulfillment_auth_middleware::<AxumBody>))
        .with_state(handler_state)
}