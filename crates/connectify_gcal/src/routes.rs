// --- File: crates/connectify_gcal/src/routes.rs ---

use crate::handlers::get_booked_events_handler;
use crate::handlers::{
    book_slot_handler, delete_event_handler, get_availability_handler,
    mark_booking_cancelled_handler, GcalState,
};
use axum::{
    routing::{delete, get, patch, post}, // Add delete here
    Router,
};

use crate::auth::create_calendar_hub;
use connectify_config::AppConfig; // Implement this function
                                  // Import handlers from the handlers module
use std::sync::Arc; // Needed for State type hint if not using AppState directly

// Define the type for the shared state expected by these routes
// If using a nested AppState struct in main.rs, adjust this accordingly
#[allow(dead_code)]
type SharedGcalState = Arc<GcalState>;

/// Creates a router containing all routes for the Google Calendar feature.
/// Requires GcalState to be available in the application state.
pub async fn routes(config: Arc<AppConfig>) -> Router {
    // Create GcalState internally using the config
    let calendar_hub =
        create_calendar_hub(config.clone().gcal.as_ref().expect("GCal config missing"))
            .await
            .unwrap(); // Implement this function
    let gcal_state = Arc::new(GcalState {
        config,
        calendar_hub: Arc::new(calendar_hub),
    });

    Router::new()
        .route("/availability", get(get_availability_handler))
        .route("/available-slots", get(get_availability_handler))
        .route("/gcal/available-slots", get(get_availability_handler))
        .route("/book", post(book_slot_handler))
        .route("/gcal/book", post(book_slot_handler))
        .route("/admin/delete/{event_id}", delete(delete_event_handler))
        .route(
            "/admin/gcal/delete/{event_id}",
            delete(delete_event_handler),
        )
        .route(
            "/admin/mark_cancelled/{event_id}",
            patch(mark_booking_cancelled_handler),
        )
        .route("/admin/bookings", get(get_booked_events_handler))
        // Add this new route
        .with_state(gcal_state)
}
