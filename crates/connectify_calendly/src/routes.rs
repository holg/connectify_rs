// --- File: crates/connectify_calendly/src/routes.rs ---
#![cfg(feature = "calendly")]

use actix_web::{web, Scope};
use std::sync::Arc;

use crate::handlers::{
    book_slot, calendly_auth_callback, calendly_test_file, get_available_slots, start_calendly_auth,
};
use crate::logic::create_slots_state;
use crate::models::CalendlyConfig;
use crate::storage::TokenStore;

/// Creates a router containing all routes for the Calendly integration.
///
/// # Arguments
/// * `config` - The Calendly configuration
/// * `token_store` - The token store for OAuth tokens
///
/// # Returns
/// A Scope containing all Calendly routes
pub fn routes(
    config: web::Data<CalendlyConfig>,
    token_store: web::Data<Arc<dyn TokenStore>>,
) -> Scope {
    // Create the state for the Calendly slots handlers
    let slots_state = create_slots_state(&config);

    web::scope("")
        .app_data(web::Data::new(slots_state))
        .service(start_calendly_auth)
        .service(calendly_auth_callback)
        .service(get_available_slots)
        .service(book_slot)
        .service(calendly_test_file)
}
