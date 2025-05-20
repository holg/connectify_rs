// src/calendly/config.rs
#![cfg(feature = "calendly")]

use cookie::Key as CookieSignKey;
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
    AuthUrl, ClientId, ClientSecret, EmptyExtraTokenFields, EndpointNotSet, EndpointSet,
    RedirectUrl, RevocationErrorResponseType, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenUrl,
};
use std::env;
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

impl CalendlyConfig {
    pub fn load() -> Result<Self, String> {
        let client_id = env::var("CALENDLY_CLIENT_ID")
            .map(ClientId::new)
            .map_err(|_| "Missing CALENDLY_CLIENT_ID".to_string())?;
        let client_secret = env::var("CALENDLY_CLIENT_SECRET")
            .map(ClientSecret::new)
            .map_err(|_| "Missing CALENDLY_CLIENT_SECRET".to_string())?;
        let redirect_uri = env::var("CALENDLY_REDIRECT_URI")
            .map_err(|_| "Missing CALENDLY_REDIRECT_URI".to_string())
            .and_then(|u| {
                RedirectUrl::new(u).map_err(|_| "Invalid CALENDLY_REDIRECT_URI".to_string())
            })?;

        let csrf_key_raw =
            env::var("CSRF_STATE_SECRET").map_err(|_| "Missing CSRF_STATE_SECRET".to_string())?;
        if csrf_key_raw.len() < 32 {
            return Err("CSRF_STATE_SECRET must be at least 32 bytes.".into());
        }
        let csrf_state_key = CookieSignKey::from(csrf_key_raw.as_bytes());

        let auth_url = AuthUrl::new("https://auth.calendly.com/oauth/authorize".to_string())
            .map_err(|_| "Invalid Auth URL".to_string())?;
        let token_url = TokenUrl::new("https://auth.calendly.com/oauth/token".to_string())
            .map_err(|_| "Invalid Token URL".to_string())?;

        let database_url =
            env::var("DATABASE_URL").map_err(|_| "Missing DATABASE_URL".to_string())?;
        let encryption_key_hex =
            env::var("ENCRYPTION_KEY").map_err(|_| "Missing ENCRYPTION_KEY".to_string())?;
        let encryption_key = hex::decode(encryption_key_hex)
            .map_err(|_| "ENCRYPTION_KEY must be hex-encoded".to_string())?;

        let personal_token = env::var("CALENDLY_PERSONAL_TOKEN")
            .ok()
            .map(|s| s.into_bytes());

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

    pub fn oauth_client(
        &self,
    ) -> BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> {
        BasicClient::new(self.client_id.clone())
            .set_client_secret(self.client_secret.clone())
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_uri.clone())
    }
}

use crate::calendly::{calendly_slots::SlotEventUrlCache, CalendlySlotsState};
use crate::storage::{TokenStore, TokenStoreData};
use crate::utils::sqlx_helper::create_sqlite_token_store;
use std::sync::Arc;

pub async fn create_token_store(config: &CalendlyConfig) -> TokenStore {
    create_sqlite_token_store(&config.database_url, config.encryption_key)
        .await
        .expect("Failed to create token store")
}

pub fn create_slots_state(config: &CalendlyConfig) -> CalendlySlotsState {
    CalendlySlotsState {
        token_store: Arc::new(TokenStore::InMemory),
        event_cache: SlotEventUrlCache::default(),
        user_url: None,
        client_id: config.client_id.clone(),
        redirect_uri: config.redirect_uri.clone(),
    }
}
