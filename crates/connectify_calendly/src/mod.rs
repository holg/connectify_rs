// --- File: crates/connectify_calendly/src/mod.rs
pub mod calendly_oauth;
pub mod calendly_slots;
pub mod config;

use actix_files::NamedFile;
use actix_web::get;
use std::sync::{Arc, RwLock};
// Re-export the public handlers for easier use from outside this module (e.g., from main.rs)
// This makes `calendly::start_calendly_auth` and `calendly::calendly_auth_callback` valid paths.
pub use calendly_oauth::{start_calendly_auth, calendly_auth_callback,
                         print_calendly_oauth_url, refresh_calendly_token, CalendlyTokenResponse};
pub use calendly_slots::{get_available_slots, book_slot, AvailableSlot, BookSlotRequest};
#[get("/calendly_test.html")]
async fn calendly_test_file() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("tests/e2e/calendly_test.html")?)
}
#[derive(Clone)]
pub struct CalendlySlotsState {
    pub calendly_personal_token: String,
    pub calendly_event_urls: Arc<RwLock<Vec<String>>>,
    pub calendly_user_url: Arc<RwLock<String>>,
    pub client: reqwest::Client,
}