#![cfg(feature = "calendly")]
#![allow(dead_code)]
use crate::calendly::{refresh_calendly_token, CalendlySlotsState};
use crate::config::AppConfig;
use crate::storage::{create_sqlite_token_store, TokenStore};
use actix_web::{get, post, web, HttpResponse, Responder, Result as ActixResult};
use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

pub type SlotEventUrlCache = Arc<RwLock<Vec<String>>>;
// --- Request Query / Payload Types ---
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

// --- Response Types ---
#[derive(Serialize, Debug, Deserialize)]
pub struct AvailableSlot {
    pub start_time: String,
    pub end_time: String,
    pub uri: String,
}

// --- Calendly API Types ---
#[derive(Deserialize)]
struct EventType {
    uri: String,
    name: String,
}

#[derive(Deserialize)]
struct EventTypesResponse {
    collection: Option<Vec<EventType>>,
}

#[derive(Deserialize)]
struct SlotInterval {
    _start_time: String,
    _end_time: String,
}

#[derive(Deserialize)]
struct AvailabilityResponse {
    _available_times: Vec<SlotInterval>,
}

#[derive(Deserialize)]
struct TimeSlot {
    start_time: String,
    status: String,
    scheduling_url: String,
}

#[derive(Deserialize)]
struct AvailableTimesResponse {
    collection: Vec<TimeSlot>,
}
// --- Public Routes ---

#[get("/api/calendly/available_slots")]
pub async fn get_available_slots(
    config: web::Data<AppConfig>,
    state: web::Data<CalendlySlotsState>,
    query: web::Query<SlotsQuery>,
) -> ActixResult<impl Responder> {
    let token = get_default_user_token(&config).await?;
    info!("Fetching available slots for user: {}", token);
    let (start_date, end_date) = calculate_date_range(&query);
    let user_uri = fetch_calendly_user_url(&state, &token).await?;
    let event_types = fetch_event_types(&state, &token, &user_uri).await?;
    for e in &event_types {
        info!("Event: {} (URI: {})", e.name, e.uri);
    }
    let mut all_slots = Vec::new();
    for event in event_types {
        let mut event_slots =
            fetch_availability_for_event(&state.client, &token, &event, &start_date, &end_date)
                .await;
        all_slots.append(&mut event_slots);
    }

    Ok(HttpResponse::Ok().json(all_slots))
}

#[post("/api/calendly/book_slot")]
pub async fn book_slot(
    config: web::Data<AppConfig>,
    state: web::Data<CalendlySlotsState>,
    payload: web::Json<BookSlotRequest>,
) -> ActixResult<impl Responder> {
    let token = get_default_user_token(&config).await?;

    let invitee_email = payload
        .invitee_email
        .clone()
        .unwrap_or_else(|| "somebody@swissappgroup.ch".to_string());
    let event_type = payload
        .event_type
        .clone()
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing event_type in request"))?;
    let start_time = payload
        .start_time
        .clone()
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing start_time in request"))?;

    let body = serde_json::json!({
        "event_type": event_type,
        "invitee": { "email": invitee_email },
        "start_time": start_time
    });

    info!("📤 Booking with payload: {}", body);

    let res = state
        .client
        .post("https://api.calendly.com/scheduled_events")
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await;

    match res {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            info!("📥 Calendly response body: {}", body);
            if status.is_success() {
                Ok(HttpResponse::Ok().body(body))
            } else {
                Ok(HttpResponse::BadRequest().body(format!("Calendly API error: {}", body)))
            }
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Request failed: {e}"))),
    }
}

// --- Helpers ---

fn calculate_date_range(query: &web::Query<SlotsQuery>) -> (String, String) {
    let today = Utc::now().date_naive();
    let start = query
        .start_date
        .clone()
        .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| today + Duration::days(1));

    let end = query
        .end_date
        .clone()
        .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| start + Duration::days(6)); // keep range ≤ 7 days

    let (start_date, end_date) = (start.to_string(), end.to_string());

    (start_date, end_date)
}

async fn fetch_calendly_user_url(
    state: &CalendlySlotsState,
    token: &str,
) -> Result<String, actix_web::Error> {
    #[derive(Deserialize)]
    struct MeResponse {
        resource: MeUser,
    }

    #[derive(Deserialize)]
    struct MeUser {
        uri: String,
    }

    {
        let read_guard = state.calendly_user_url.read().unwrap();
        if !read_guard.is_empty() {
            return Ok(read_guard.clone());
        }
    }

    let user_resp = state
        .client
        .get("https://api.calendly.com/users/me")
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to fetch user: {e}"))
        })?;

    let body = user_resp.text().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to read response: {e}"))
    })?;
    info!("📦 Raw Calendly /me response:\n{}", body);
    let me: MeResponse = serde_json::from_str(&body).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to parse user: {e}"))
    })?;
    info!("User URI: {}", me.resource.uri);
    {
        let mut write_guard = state.calendly_user_url.write().unwrap();
        *write_guard = me.resource.uri.clone();
    }

    Ok(me.resource.uri)
}

async fn fetch_event_types(
    state: &CalendlySlotsState,
    token: &str,
    user_uri: &str,
) -> Result<Vec<EventType>, actix_web::Error> {
    {
        // 1. Prüfen, ob gecachte URIs vorhanden sind
        let cached = state.calendly_event_urls.read().unwrap();
        if !cached.is_empty() {
            let events = cached
                .iter()
                .map(|uri| EventType {
                    uri: uri.clone(),
                    name: "<cached>".to_string(), // Wir speichern Namen nicht im Cache
                })
                .collect();
            return Ok(events);
        }
    }
    // 2. Sonst API Call machen
    let url = format!("https://api.calendly.com/event_types?user={}", user_uri);
    let res = state
        .client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed request: {e}")))?;

    let body = res
        .text()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Body read error: {e}")))?;

    let parsed: EventTypesResponse = serde_json::from_str(&body).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("JSON parse error: {e}"))
    })?;

    let events = parsed.collection.unwrap_or_default();

    {
        // 3. In den Cache schreiben
        let mut writer = state.calendly_event_urls.write().unwrap();
        *writer = events.iter().map(|e| e.uri.clone()).collect();
    }

    Ok(events)
}

async fn fetch_availability_for_event(
    client: &reqwest::Client,
    token: &str,
    event: &EventType,
    start_date: &str,
    end_date: &str,
) -> Vec<AvailableSlot> {
    // Outdated, will not work soon ?(May 2025)?
    // let event_id = event.uri.split('/').last().unwrap_or("");
    // let url = format!(
    //     "https://api.calendly.com/event_types/{}/availability?start_time={}T00:00:00Z&end_time={}T23:59:59Z",
    //     event_id, start_date, end_date
    // );
    let url = format!(
        "https://api.calendly.com/event_type_available_times?event_type={}&start_time={}T00:00:00Z&end_time={}T23:59:59Z&timezone=Europe/Berlin",
        event.uri, start_date, end_date
    );
    info!("API URL: {}", &url);

    let mut slots = Vec::new();
    let resp = client.get(&url).bearer_auth(token).send().await;
    info!(
        "�� Raw Calendly /event_type_available_times response:\n{:?}",
        resp
    );
    if let Ok(resp) = resp {
        if resp.status().is_success() {
            if let Ok(parsed) = resp.json::<AvailableTimesResponse>().await {
                for slot in parsed.collection {
                    if slot.status == "available" {
                        slots.push(AvailableSlot {
                            start_time: slot.start_time,
                            end_time: "".to_string(), // or calculate from duration?
                            uri: event.uri.clone(),
                        });
                    }
                }
            }
        }
    }
    /*if let Ok(resp) = client.get(&url).bearer_auth(token).send().await {
        if resp.status().is_success() {
            if let Ok(parsed) = resp.json::<AvailabilityResponse>().await {
                for slot in parsed.available_times {
                    slots.push(AvailableSlot {
                        start_time: slot.start_time,
                        end_time: slot.end_time,
                        uri: event.uri.clone(),
                    });
                }
            }
        }
    }*/

    slots
}

pub async fn get_default_user_token(config: &AppConfig) -> Result<String, actix_web::Error> {
    let token_store = create_sqlite_token_store(
        &config.calendly_config.database_url,
        config.calendly_config.encryption_key.to_vec(),
    )
    .await
    .unwrap();

    let (access_token, refresh_token_opt, expires_at_opt) = token_store
        .get_token_decrypted("default_calendly_user", "calendly")
        .await
        .map_err(|e| {
            info!("❌ Failed to load token: {e}");
            actix_web::error::ErrorInternalServerError("DB error")
        })?
        .ok_or_else(|| {
            info!("❌ Token not found for default_calendly_user");
            actix_web::error::ErrorUnauthorized("Calendly token missing")
        })?;

    let now = chrono::Utc::now().timestamp();
    let is_expired = expires_at_opt.map(|exp| exp <= now).unwrap_or(true);

    if is_expired {
        info!("🔁 Access token expired — refreshing via refresh_token...");
        let new_token = refresh_calendly_token(
            config.calendly_config.client_id.as_ref(),
            config.calendly_config.client_secret.secret(),
            refresh_token_opt.as_deref().ok_or_else(|| {
                actix_web::error::ErrorInternalServerError("Missing refresh_token")
            })?,
        )
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Token refresh failed: {e}"))
        })?;

        let new_access = new_token.access_token.clone();
        let new_refresh = new_token.refresh_token.clone();
        let new_expiry = chrono::Utc::now().timestamp() + new_token.expires_in;

        token_store
            .save_token_encrypted(
                "default_calendly_user",
                "calendly",
                &new_access,
                new_refresh.as_deref(),
                Some(new_expiry),
            )
            .await
            .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to save new token"))?;

        Ok(new_access)
    } else {
        String::from_utf8(access_token.into()).map_err(|_| {
            actix_web::error::ErrorInternalServerError("Access token is not valid UTF-8")
        })
    }
}

fn get_personal_access_token(config: &AppConfig) -> Option<String> {
    config
        .calendly_config
        .personal_token
        .as_ref()
        .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
}
