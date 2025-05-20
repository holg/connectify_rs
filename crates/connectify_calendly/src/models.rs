// --- File: crates/connectify_calendly/src/models.rs ---
#![cfg(feature = "calendly")]

use cookie::Key as CookieSignKey;
use oauth2::{AuthUrl, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl, TokenUrl};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// --- Request/Response Types ---

#[derive(Deserialize)]
pub struct SlotsQuery {
    pub start_date: Option<String>, // e.g. 2025-04-15
    pub end_date: Option<String>,   // e.g. 2025-04-20
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BookSlotRequest {
    pub invitee_email: Option<String>,
    pub event_type: Option<String>,
    pub start_time: Option<String>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct AvailableSlot {
    pub start_time: String,
    pub end_time: String,
    pub uri: String,
}

#[derive(Deserialize, Debug)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CalendlyTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

// --- Calendly API Types ---

#[derive(Deserialize)]
pub struct EventType {
    pub uri: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct EventTypesResponse {
    pub collection: Option<Vec<EventType>>,
}

#[derive(Deserialize)]
pub struct SlotInterval {
    pub _start_time: String,
    pub _end_time: String,
}

#[derive(Deserialize)]
pub struct AvailabilityResponse {
    pub _available_times: Vec<SlotInterval>,
}

#[derive(Deserialize)]
pub struct TimeSlot {
    pub start_time: String,
    pub status: String,
    pub scheduling_url: String,
}

#[derive(Deserialize)]
pub struct AvailableTimesResponse {
    pub collection: Vec<TimeSlot>,
}

// --- Configuration ---

#[derive(Clone)]
pub struct CalendlyConfig {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub redirect_uri: RedirectUrl,
    pub auth_url: AuthUrl,
    pub token_url: TokenUrl,
    pub csrf_state_key: CookieSignKey,
    pub database_url: String,
    pub encryption_key: Vec<u8>,
    pub personal_token: Option<Vec<u8>>,
}

// --- State ---

#[derive(Clone)]
pub struct CalendlySlotsState {
    pub calendly_personal_token: String,
    pub calendly_event_urls: Arc<RwLock<Vec<String>>>,
    pub calendly_user_url: Arc<RwLock<String>>,
    pub client: reqwest::Client,
}

pub type SlotEventUrlCache = Arc<RwLock<Vec<String>>>;
