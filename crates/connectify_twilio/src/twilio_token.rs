// --- File: crates/connectify_twilio/src/twilio_token.rs ---
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use connectify_config::AppConfig;

#[derive(Debug, Serialize, Deserialize)]
struct VideoGrant {
    room: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Grants {
    identity: String,
    video: VideoGrant,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // Twilio Account SID
    iss: String, // Twilio API Key SID
    exp: usize,  // Expiration timestamp (Unix epoch seconds)
    jti: String, // Unique identifier for the token
    // aud: String, // Usually not needed for Video Grant
    grants: Grants,
}

#[derive(Deserialize, Debug)] // Added Debug
pub struct TokenRequestQuery {
    pub identity: String,
    #[serde(rename = "roomName")]
    pub room_name: String,
}

#[derive(Serialize, Debug)] // Added Debug
pub struct TokenResponse {
    pub token: String,
}

// --- Axum Handler Function ---

/// Generates a Twilio access token for video services using Axum.
///
/// Expects shared AppConfig state and query parameters.
/// Loads the API secret directly from environment variables.
/// Returns a JSON token on success or a status code + error message on failure.
/// Generates a Twilio access token for video services using Axum.
///
/// Expects shared AppConfig state and query parameters.
/// Loads the API secret directly from environment variables.
/// Returns a JSON token on success or a status code + error message on failure.
#[axum::debug_handler] // Useful Axum macro for debugging handler type issues
pub async fn generate_token(
    State(config): State<Arc<AppConfig>>, // Axum state extractor (Arc<AppConfig>)
    Query(query): Query<TokenRequestQuery>, // Axum query extractor
) -> Result<Json<TokenResponse>, (StatusCode, String)> {
    // Axum Result type

    // --- Access Config & Check Runtime Flag ---
    // Check if the optional twilio config section exists
    let Some(twilio_conf) = config.twilio.as_ref() else {
        let err_msg = "Twilio configuration section missing in server config.".to_string();
        eprintln!("{}", err_msg);
        // Return Axum error tuple
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err_msg));
    };

    // Check the runtime flag
    if !config.use_twilio {
        let err_msg = "Twilio service is disabled by configuration.".to_string();
        eprintln!("{}", err_msg);
        return Err((StatusCode::SERVICE_UNAVAILABLE, err_msg));
    }

    let api_key_secret = match &config.twilio {
        Some(twilio_conf) => twilio_conf.api_key_secret.clone(),
        None => {
            let err_msg = "Missing TWILIO Config.".to_string();
            eprintln!("{}", err_msg);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, err_msg));
        }
    };

    // --- Token Generation Logic ---
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    // Use fields from twilio_conf loaded via AppConfig state
    let jti = format!("{}-{}", twilio_conf.api_key_sid, now_secs);

    let expiry_seconds: i64 = env::var("TOKEN_EXPIRY") // Reading helper env var is fine
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(3600); // Default to 1 hour

    let expiration = (Utc::now() + Duration::seconds(expiry_seconds)).timestamp() as usize;

    // Construct claims using data from config and query
    let claims = Claims {
        sub: twilio_conf.account_sid.clone(),
        iss: twilio_conf.api_key_sid.clone(),
        exp: expiration,
        jti,
        grants: Grants {
            identity: query.identity.clone(),
            video: VideoGrant {
                room: Some(query.room_name.clone()),
            },
        },
    };

    println!(
        "DEBUG: Generating token with iss(SK): {}, sub(AC): {}",
        claims.iss, claims.sub
    );

    // Standard Twilio JWT headers for Video Grant tokens
    let mut header = Header::new(Algorithm::HS256);
    header.cty = Some("twilio-fpa;v=1".to_string());
    header.typ = Some("JWT".to_string());

    // Encode the token using the secret loaded from the environment
    match encode(
        &header,
        &claims,
        &EncodingKey::from_secret(api_key_secret.as_ref()),
    ) {
        Ok(token) => {
            // On success, return JSON response
            Ok(Json(TokenResponse { token }))
        }
        Err(e) => {
            // On failure, log error and return Axum error tuple
            let err_msg = format!("Error generating token: {}", e);
            eprintln!("{}", err_msg);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate token".to_string(),
            ))
        }
    }
}
