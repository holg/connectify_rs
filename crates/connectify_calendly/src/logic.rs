// --- File: crates/connectify_calendly/src/logic.rs ---
#![cfg(feature = "calendly")]

use std::env;
use std::sync::{Arc, RwLock};
use chrono::{Duration, Utc, NaiveDate};
use reqwest::{Client as ReqwestClient, Error as ReqwestError};
use oauth2::{
    AuthorizationCode, CsrfToken, Scope, TokenResponse,
    basic::BasicTokenResponse,
    HttpRequest as OAuth2Request, HttpResponse as OAuth2Response,
    HttpClientError as OAuth2HttpClientError,
    EndpointSet, EndpointNotSet,
};
use http::{Response as HttpResponseBuilder};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as base64_engine, Engine};
use cookie::{Cookie, CookieJar, Key as CookieSignKey, SameSite};
use actix_web::{HttpRequest, error};
use uuid::Uuid;

use crate::models::{
    CalendlyConfig, CalendlySlotsState, CalendlyTokenResponse, EventType,
    SlotsQuery, AvailableSlot, SlotEventUrlCache, BookSlotRequest
};
use crate::storage::{TokenStore, create_sqlite_token_store};
use crate::utils::{crypto};

// --- Constants ---
pub const CSRF_COOKIE_NAME: &str = "calendly_csrf_state";

// --- OAuth Logic ---

pub async fn async_http_client(
    request: OAuth2Request,
) -> Result<OAuth2Response, OAuth2HttpClientError<ReqwestError>> {
    let client = ReqwestClient::new();
    let mut req_builder = client.request(request.method().clone(), request.uri().to_string()).headers(request.headers().clone());
    if !request.body().is_empty() {
        req_builder = req_builder.body(request.body().clone());
    }
    let response = req_builder.send().await.map_err(|e| OAuth2HttpClientError::Reqwest(Box::new(e)))?;
    let status = response.status();
    let resp_headers = response.headers().clone();
    let body_bytes = response.bytes().await.map_err(|e| OAuth2HttpClientError::Reqwest(Box::new(e)))?.to_vec();

    let mut builder = HttpResponseBuilder::builder().status(status);
    let Some(builder_headers) = builder.headers_mut() else {
        return Err(OAuth2HttpClientError::Other("Failed to access response builder headers".into()));
    };
    *builder_headers = resp_headers;

    builder.body(body_bytes).map_err(|e| OAuth2HttpClientError::Other(format!("Failed to build response: {e}").into()))
}

pub async fn exchange_code_for_token(
    code: &str,
    config: &CalendlyConfig,
) -> Result<BasicTokenResponse, actix_web::Error> {
    let client = config.oauth_client() else {
        return Err(error::ErrorInternalServerError("Calendly OAuth client init failed"));
    };
    let code = AuthorizationCode::new(code.to_owned());

    let token_response = client
        .exchange_code(code)
        .request_async(&async_http_client)
        .await
        .map_err(|e| {
            info!("Failed to exchange Calendly code: {:?}", e);
            error::ErrorInternalServerError("OAuth token exchange failed")
        })?;

    Ok(token_response)
}

pub fn extract_csrf_state(req: &HttpRequest, csrf_key: &CookieSignKey) -> Option<(CookieJar, String)> {
    let mut jar = CookieJar::new();
    if let Some(cookie_header) = req.headers().get(actix_web::http::header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for pair in cookie_str.split(';') {
                if let Ok(cookie) = Cookie::parse_encoded(pair.trim()) {
                    jar.add_original(cookie.into_owned());
                }
            }
        }
    }

    let stored_encoded_state = jar.private(&csrf_key).get(CSRF_COOKIE_NAME)?.value().to_string();
    let decoded = base64_engine.decode(&stored_encoded_state).ok()?;
    let decoded_str = String::from_utf8(decoded).ok()?;
    Some((jar, decoded_str))
}

pub fn extract_csrf_state_fallback(req: &HttpRequest, expected: &str) -> Option<(CookieJar, String)> {
    let mut jar = CookieJar::new();

    let cookie_header = req.headers().get(actix_web::http::header::COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;

    info!("🍪 Full raw Cookie header: {cookie_str}");

    for pair in cookie_str.split(';') {
        let trimmed = pair.trim();
        info!("➡️ Checking cookie pair: {trimmed}");

        if let Ok(cookie) = Cookie::parse_encoded(trimmed) {
            info!("🔍 Parsed cookie: {} = {}", cookie.name(), cookie.value());

            if cookie.name() == CSRF_COOKIE_NAME {
                let cookie_val = cookie.value().trim_matches('"').trim();
                if cookie_val == expected {
                    info!("✅ Fallback CSRF matched cookie == query param.");
                    jar.add_original(cookie.clone().into_owned());
                    return Some((jar, cookie_val.to_string()));
                } else {
                    info!("❌ Fallback CSRF mismatch: cookie_val != expected: {cookie_val} != {expected}");
                }
            }
        } else {
            info!("⚠️ Failed to parse cookie: {trimmed}");
        }
    }

    None
}

pub async fn refresh_calendly_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<CalendlyTokenResponse, reqwest::Error> {
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    let client = ReqwestClient::new();
    let res = client
        .post("https://auth.calendly.com/oauth/token")
        .form(&params)
        .send()
        .await?;

    res.json::<CalendlyTokenResponse>().await
}

pub fn print_calendly_oauth_url(config: &CalendlyConfig) {
    if let (client_id, redirect_uri) = (&config.client_id, &config.redirect_uri) {
        let state = Uuid::new_v4().to_string();
        let url = format!(
            "https://auth.calendly.com/oauth/authorize?client_id={}&response_type=code&redirect_uri={}&scope=default&state={}",
            &client_id.as_ref(), urlencoding::encode(redirect_uri.as_ref()), state
        );
        info!("🔐 Calendly OAuth Start URL:\n{}", url);
    } else {
        info!("Missing calendly_client_id or calendly_redirect_uri in config");
    }
}

// --- Calendly Slots Logic ---

pub fn calculate_date_range(query: &SlotsQuery) -> (String, String) {
    let today = Utc::now().date_naive();
    let start = query.start_date.clone()
        .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| today + Duration::days(1));

    let end = query.end_date.clone()
        .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| start + Duration::days(6)); // keep range ≤ 7 days

    let (start_date, end_date) = (start.to_string(), end.to_string());

    (start_date, end_date)
}

pub async fn fetch_calendly_user_url(
    state: &CalendlySlotsState,
    token: &str,
) -> Result<String, actix_web::Error> {
    #[derive(serde::Deserialize)]
    struct MeResponse {
        resource: MeUser,
    }

    #[derive(serde::Deserialize)]
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
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to fetch user: {e}")))?;

    let body = user_resp
        .text()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to read response: {e}")))?;
    info!("📦 Raw Calendly /me response:\n{}", body);
    let me: MeResponse = serde_json::from_str(&body)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse user: {e}")))?;
    info!("User URI: {}", me.resource.uri);
    {
        let mut write_guard = state.calendly_user_url.write().unwrap();
        *write_guard = me.resource.uri.clone();
    }

    Ok(me.resource.uri)
}

pub async fn fetch_event_types(
    state: &CalendlySlotsState,
    token: &str,
    user_uri: &str,
) -> Result<Vec<EventType>, actix_web::Error> {
    {
        // 1. Prüfen, ob gecachte URIs vorhanden sind
        let cached = state.calendly_event_urls.read().unwrap();
        if !cached.is_empty() {
            let events = cached.iter().map(|uri| EventType {
                uri: uri.clone(),
                name: "<cached>".to_string(), // Wir speichern Namen nicht im Cache
            }).collect();
            return Ok(events);
        }
    }
    // 2. Sonst API Call machen
    let url = format!("https://api.calendly.com/event_types?user={}", user_uri);
    let res = state.client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed request: {e}")))?;

    let body = res.text().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Body read error: {e}"))
    })?;

    let parsed: crate::models::EventTypesResponse = serde_json::from_str(&body).map_err(|e| {
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

pub async fn fetch_availability_for_event(
    client: &reqwest::Client,
    token: &str,
    event: &EventType,
    start_date: &str,
    end_date: &str,
) -> Vec<AvailableSlot> {
    let url = format!(
        "https://api.calendly.com/event_type_available_times?event_type={}&start_time={}T00:00:00Z&end_time={}T23:59:59Z&timezone=Europe/Berlin",
        event.uri, start_date, end_date
    );
    info!("API URL: {}", &url);

    let mut slots = Vec::new();
    let resp = client.get(&url).bearer_auth(token).send().await;
    info!("�� Raw Calendly /event_type_available_times response:\n{:?}", resp);
    if let Ok(resp) = resp {
        if resp.status().is_success() {
            if let Ok(parsed) = resp.json::<crate::models::AvailableTimesResponse>().await {
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

    slots
}

pub async fn get_default_user_token(
    config: &CalendlyConfig,
) -> Result<String, actix_web::Error> {
    let token_store = create_sqlite_token_store(
        &config.database_url,
        config.encryption_key.to_vec()
    ).await.unwrap();

    let (access_token, refresh_token_opt, expires_at_opt) =
        token_store.get_token_decrypted("default_calendly_user", "calendly").await
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
            config.client_id.as_ref(),
            config.client_secret.secret(),
            refresh_token_opt.as_deref().ok_or_else(|| {
                actix_web::error::ErrorInternalServerError("Missing refresh_token")
            })?,
        )
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Token refresh failed: {e}")))?;

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

pub fn get_personal_access_token(config: &CalendlyConfig) -> Option<String> {
    config.personal_token.as_ref().and_then(|bytes| {
        String::from_utf8(bytes.clone()).ok()
    })
}

// --- Configuration Logic ---

impl CalendlyConfig {
    pub fn load() -> Result<Self, String> {
        let client_id = env::var("CALENDLY_CLIENT_ID")
            .map(oauth2::ClientId::new)
            .map_err(|_| "Missing CALENDLY_CLIENT_ID".to_string())?;
        let client_secret = env::var("CALENDLY_CLIENT_SECRET")
            .map(oauth2::ClientSecret::new)
            .map_err(|_| "Missing CALENDLY_CLIENT_SECRET".to_string())?;
        let redirect_uri = env::var("CALENDLY_REDIRECT_URI")
            .map_err(|_| "Missing CALENDLY_REDIRECT_URI".to_string())
            .and_then(|u| oauth2::RedirectUrl::new(u).map_err(|_| "Invalid CALENDLY_REDIRECT_URI".to_string()))?;

        let csrf_key_raw = env::var("CSRF_STATE_SECRET").map_err(|_| "Missing CSRF_STATE_SECRET".to_string())?;
        if csrf_key_raw.len() < 32 {
            return Err("CSRF_STATE_SECRET must be at least 32 bytes.".into());
        }
        let csrf_state_key = CookieSignKey::from(csrf_key_raw.as_bytes());

        let auth_url = oauth2::AuthUrl::new("https://auth.calendly.com/oauth/authorize".to_string())
            .map_err(|_| "Invalid Auth URL".to_string())?;
        let token_url = oauth2::TokenUrl::new("https://auth.calendly.com/oauth/token".to_string())
            .map_err(|_| "Invalid Token URL".to_string())?;

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "Missing DATABASE_URL".to_string())?;
        let encryption_key_hex = env::var("ENCRYPTION_KEY")
            .map_err(|_| "Missing ENCRYPTION_KEY".to_string())?;
        let encryption_key = hex::decode(encryption_key_hex)
            .map_err(|_| "ENCRYPTION_KEY must be hex-encoded".to_string())?;

        let personal_token = env::var("CALENDLY_PERSONAL_TOKEN").ok().map(|s| s.into_bytes());

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
            auth_url,
            token_url,
            csrf_state_key,
            database_url,
            encryption_key,
            personal_token,
        })
    }

    pub fn oauth_client(&self) -> oauth2::basic::BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> {
        oauth2::basic::BasicClient::new(self.client_id.clone())
            .set_client_secret(self.client_secret.clone())
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_uri.clone())
    }
}

pub async fn create_token_store(config: &CalendlyConfig) -> Arc<dyn TokenStore> {
    Arc::new(create_sqlite_token_store(
        &config.database_url,
        config.encryption_key.clone(),
    )
    .await
    .expect("Failed to create token store"))
}

pub fn create_slots_state(config: &CalendlyConfig) -> CalendlySlotsState {
    let personal_token = get_personal_access_token(config).unwrap_or_default();
    
    CalendlySlotsState {
        calendly_personal_token: personal_token,
        calendly_event_urls: Arc::new(RwLock::new(Vec::new())),
        calendly_user_url: Arc::new(RwLock::new(String::new())),
        client: reqwest::Client::new(),
    }
}