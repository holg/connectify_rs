// --- File: src/calendly/calendly_oauth.rs ---
#![cfg(feature = "calendly")]

use actix_web::{
    error, get, http::header, web, HttpRequest, HttpResponse, Responder, Result as ActixResult,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as base64_engine, Engine};
use chrono::{Duration as ChronoDuration, Utc};
use cookie::{Cookie, CookieJar, Key as CookieSignKey, SameSite};
use http::Response as HttpResponseBuilder; // , HeaderMap, StatusCode};
use oauth2::{
    basic::BasicTokenResponse, AuthorizationCode, CsrfToken,
    HttpClientError as OAuth2HttpClientError, HttpRequest as OAuth2Request,
    HttpResponse as OAuth2Response, Scope, TokenResponse,
};
use reqwest::{Client as ReqwestClient, Error as ReqwestError};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::AppConfig,
    storage::TokenStore, //, StorageError},
    utils::crypto,       //sqlx_helper},
};

#[derive(Deserialize, Debug)]
struct AuthCallbackQuery {
    code: String,
    state: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct CalendlyTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

const CSRF_COOKIE_NAME: &str = "calendly_csrf_state";

async fn async_http_client(
    request: OAuth2Request,
) -> Result<OAuth2Response, OAuth2HttpClientError<ReqwestError>> {
    let client = ReqwestClient::new();
    let mut req_builder = client
        .request(request.method().clone(), request.uri().to_string())
        .headers(request.headers().clone());
    if !request.body().is_empty() {
        req_builder = req_builder.body(request.body().clone());
    }
    let response = req_builder
        .send()
        .await
        .map_err(|e| OAuth2HttpClientError::Reqwest(Box::new(e)))?;
    let status = response.status();
    let resp_headers = response.headers().clone();
    let body_bytes = response
        .bytes()
        .await
        .map_err(|e| OAuth2HttpClientError::Reqwest(Box::new(e)))?
        .to_vec();

    let mut builder = HttpResponseBuilder::builder().status(status);
    let Some(builder_headers) = builder.headers_mut() else {
        return Err(OAuth2HttpClientError::Other(
            "Failed to access response builder headers".into(),
        ));
    };
    *builder_headers = resp_headers;

    builder
        .body(body_bytes)
        .map_err(|e| OAuth2HttpClientError::Other(format!("Failed to build response: {e}").into()))
}

pub async fn exchange_code_for_token(
    code: &str,
    config: &AppConfig,
) -> Result<BasicTokenResponse, actix_web::Error> {
    let client = config.calendly_config.oauth_client() else {
        return Err(error::ErrorInternalServerError(
            "Calendly OAuth client init failed",
        ));
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
fn extract_csrf_state(req: &HttpRequest, csrf_key: &CookieSignKey) -> Option<(CookieJar, String)> {
    let mut jar = CookieJar::new();
    if let Some(cookie_header) = req.headers().get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for pair in cookie_str.split(';') {
                if let Ok(cookie) = Cookie::parse_encoded(pair.trim()) {
                    jar.add_original(cookie.into_owned());
                }
            }
        }
    }

    let stored_encoded_state = jar
        .private(&csrf_key)
        .get(CSRF_COOKIE_NAME)?
        .value()
        .to_string();
    let decoded = base64_engine.decode(&stored_encoded_state).ok()?;
    let decoded_str = String::from_utf8(decoded).ok()?;
    Some((jar, decoded_str))
}

fn extract_csrf_state_fallback(req: &HttpRequest, expected: &str) -> Option<(CookieJar, String)> {
    let mut jar = CookieJar::new();

    let cookie_header = req.headers().get(header::COOKIE)?;
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
#[get("/auth/calendly/start")]
pub async fn start_calendly_auth(config: web::Data<AppConfig>) -> ActixResult<impl Responder> {
    let Some(csrf_key) = config.calendly_config.csrf_state_key.clone().into() else {
        return Err(error::ErrorInternalServerError(
            "Missing CSRF key in config",
        ));
    };
    let client = config.calendly_config.oauth_client() else {
        return Err(error::ErrorInternalServerError(
            "Calendly OAuth client init failed",
        ));
    };

    let (authorize_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("default".to_string()))
        .url();
    let state_value = csrf_token.secret().to_string();
    let encoded_state = base64_engine.encode(state_value);
    let mut jar = CookieJar::new();

    jar.private_mut(&csrf_key).add(
        Cookie::build(CSRF_COOKIE_NAME, encoded_state.clone())
            .path("/")
            .secure(false)
            .http_only(true)
            .same_site(SameSite::Lax)
            .max_age(cookie::time::Duration::minutes(10))
            .finish(),
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
    config: web::Data<AppConfig>,
    token_store: web::Data<Arc<dyn TokenStore>>,
) -> ActixResult<impl Responder> {
    let query = query.into_inner();
    let Some(csrf_key) = config.calendly_config.csrf_state_key.clone().into() else {
        return Err(error::ErrorBadRequest("Missing CSRF key in config"));
    };

    let (_jar, stored_state) = extract_csrf_state(&req, &csrf_key)
        .or_else(|| {
            info!("⚠️ Using unsigned fallback CSRF cookie for dev/testing.");
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
    let encrypted_access_token = crypto::encrypt(
        &config.calendly_config.encryption_key.to_vec(),
        access_token.secret().as_bytes(),
    )
    .map_err(|_| error::ErrorInternalServerError("Encryption failed"))?;
    let refresh_token_opt = token_response.refresh_token();
    let expires_in_opt = token_response.expires_in();

    let encrypted_refresh_token = refresh_token_opt
        .map(|rt| {
            crypto::encrypt(
                &config.calendly_config.encryption_key.to_vec(),
                rt.secret().as_bytes(),
            )
        })
        .transpose()
        .expect("expected refresh token to be present"); // converts Result<Option<Vec<u8>>> into Option<Result<Vec<u8>>> -> Result<Option<Vec<u8>>>

    let expires_at = expires_in_opt
        .and_then(|std_dur| ChronoDuration::from_std(std_dur).ok())
        .map(|chrono_dur| Utc::now() + chrono_dur)
        .map(|dt| dt.timestamp());

    token_store
        .save_token(
            user_identifier,
            service_name,
            &encrypted_access_token,
            encrypted_refresh_token.as_deref(),
            expires_at,
        )
        .await
        .map_err(|_| error::ErrorInternalServerError("Failed to save token"))?;

    Ok(HttpResponse::Ok().body("Calendly access token stored successfully"))
}

pub fn print_calendly_oauth_url(config: &AppConfig) {
    if let (client_id, redirect_uri) = (
        &config.calendly_config.client_id,
        &config.calendly_config.redirect_uri,
    ) {
        let state = Uuid::new_v4().to_string();
        let url = format!(
            "https://auth.calendly.com/oauth/authorize?client_id={}&response_type=code&redirect_uri={}&scope=default&state={}",
            &client_id.as_str(), urlencoding::encode(redirect_uri), state
        );
        info!("🔐 Calendly OAuth Start URL:\n{}", url);
    } else {
        info!("Missing calendly_client_id or calendly_redirect_uri in config");
    }
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
