// --- File: crates/connectify_calendly/src/handlers.rs ---
#![cfg(feature = "calendly")]

use actix_web::{
    get, post, web, HttpRequest, HttpResponse, Responder, Result as ActixResult, error, http::header
};
use std::sync::Arc;
use oauth2::{CsrfToken, Scope, TokenResponse};
use cookie::{Cookie, CookieJar};

use crate::models::{
    CalendlyConfig, CalendlySlotsState, AuthCallbackQuery, SlotsQuery, BookSlotRequest
};
use crate::logic::{
    exchange_code_for_token, extract_csrf_state, extract_csrf_state_fallback,
    calculate_date_range, fetch_calendly_user_url, fetch_event_types,
    fetch_availability_for_event, get_default_user_token, CSRF_COOKIE_NAME
};
use crate::storage::TokenStore;
use crate::utils::crypto;

// --- OAuth Handlers ---

#[get("/auth/calendly/start")]
pub async fn start_calendly_auth(config: web::Data<CalendlyConfig>) -> ActixResult<impl Responder> {
    let Some(csrf_key) = config.csrf_state_key.clone().into() else {
        return Err(error::ErrorInternalServerError("Missing CSRF key in config"));
    };
    let client = config.oauth_client() else {
        return Err(error::ErrorInternalServerError("Calendly OAuth client init failed"));
    };

    let (authorize_url, csrf_token) = client.authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("default".to_string())).url();
    let state_value = csrf_token.secret().to_string();
    let encoded_state = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(state_value);
    let mut jar = CookieJar::new();

    jar.private_mut(&csrf_key).add(
        Cookie::build(CSRF_COOKIE_NAME, encoded_state.clone())
            .path("/").secure(false).http_only(true)
            .same_site(cookie::SameSite::Lax)
            .max_age(cookie::time::Duration::minutes(10))
            .finish()
    );

    let mut response_builder = HttpResponse::Found();
    response_builder.append_header((header::LOCATION, authorize_url.to_string()));
    for cookie in jar.delta() {
        response_builder.cookie(cookie.clone());
    }
    Ok(response_builder.finish())
}

#[get("/api/calendly/auth/")]
pub async fn calendly_auth_callback(
    req: HttpRequest,
    query: web::Query<AuthCallbackQuery>,
    config: web::Data<CalendlyConfig>,
    token_store: web::Data<Arc<dyn TokenStore>>,
) -> ActixResult<impl Responder> {
    let query = query.into_inner();
    let Some(csrf_key) = config.csrf_state_key.clone().into() else {
        return Err(error::ErrorBadRequest("Missing CSRF key in config"));
    };

    let (_jar, stored_state) = extract_csrf_state(&req, &csrf_key)
        .or_else(|| {
            eprintln!("⚠️ Using unsigned fallback CSRF cookie for dev/testing.");
            extract_csrf_state_fallback(&req, "trdt")
        })
        .ok_or_else(|| error::ErrorBadRequest("CSRF state not found or invalid"))?;

    if stored_state != query.state {
        return Err(error::ErrorBadRequest("CSRF state mismatch"));
    }
    let user_identifier = "default_calendly_user";
    let service_name = "calendly";
    let token_response = exchange_code_for_token(&query.code, &config).await?;
    let access_token = token_response.access_token();
    let encrypted_access_token = crypto::encrypt(&config.encryption_key.to_vec(), access_token.secret().as_bytes())
        .map_err(|_| error::ErrorInternalServerError("Encryption failed"))?;
    let refresh_token_opt = token_response.refresh_token();
    let expires_in_opt = token_response.expires_in();

    let encrypted_refresh_token = refresh_token_opt.map(|rt| {
        crypto::encrypt(&config.encryption_key.to_vec(), rt.secret().as_bytes())
    }).transpose().expect("expected refresh token to be present"); // converts Result<Option<Vec<u8>>> into Option<Result<Vec<u8>>> -> Result<Option<Vec<u8>>>

    let expires_at = expires_in_opt
        .and_then(|std_dur| chrono::Duration::from_std(std_dur).ok())
        .map(|chrono_dur| chrono::Utc::now() + chrono_dur)
        .map(|dt| dt.timestamp());

    token_store.save_token(
        user_identifier,
        service_name,
        &encrypted_access_token,
        encrypted_refresh_token.as_deref(),
        expires_at,
    ).await.map_err(|_| error::ErrorInternalServerError("Failed to save token"))?;

    Ok(HttpResponse::Ok().body("Calendly access token stored successfully"))
}

// --- Calendly Slots Handlers ---

#[get("/api/calendly/available_slots")]
pub async fn get_available_slots(
    config: web::Data<CalendlyConfig>,
    state: web::Data<CalendlySlotsState>,
    query: web::Query<SlotsQuery>,
) -> ActixResult<impl Responder> {
    let token = get_default_user_token(&config).await?;
    println!("Fetching available slots for user: {}", token);
    let (start_date, end_date) = calculate_date_range(&query);
    let user_uri = fetch_calendly_user_url(&state, &token).await?;
    let event_types = fetch_event_types(&state, &token, &user_uri).await?;
    for e in &event_types {
        println!("Event: {} (URI: {})", e.name, e.uri);
    }
    let mut all_slots = Vec::new();
    for event in event_types {
        let mut event_slots = fetch_availability_for_event(
            &state.client,
            &token,
            &event,
            &start_date,
            &end_date,
        )
            .await;
        all_slots.append(&mut event_slots);
    }

    Ok(HttpResponse::Ok().json(all_slots))
}

#[post("/api/calendly/book_slot")]
pub async fn book_slot(
    config: web::Data<CalendlyConfig>,
    state: web::Data<CalendlySlotsState>,
    payload: web::Json<BookSlotRequest>,
) -> ActixResult<impl Responder> {
    let token = get_default_user_token(&config).await?;

    let invitee_email = payload.invitee_email.clone().unwrap_or_else(|| "somebody@swissappgroup.ch".to_string());
    let event_type = payload.event_type.clone().ok_or_else(|| {
        actix_web::error::ErrorBadRequest("Missing event_type in request")
    })?;
    let start_time = payload.start_time.clone().ok_or_else(|| {
        actix_web::error::ErrorBadRequest("Missing start_time in request")
    })?;

    let body = serde_json::json!({
        "event_type": event_type,
        "invitee": { "email": invitee_email },
        "start_time": start_time
    });

    println!("📤 Booking with payload: {}", body);

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
            eprintln!("📥 Calendly response body: {}", body);
            if status.is_success() {
                Ok(HttpResponse::Ok().body(body))
            } else {
                Ok(HttpResponse::BadRequest()
                    .body(format!("Calendly API error: {}", body)))
            }
        }
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("Request failed: {e}"))),
    }
}

#[get("/calendly_test.html")]
pub async fn calendly_test_file() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("tests/e2e/calendly_test.html")?)
}