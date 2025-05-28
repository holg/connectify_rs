// --- File: crates/connectify_twilio/src/twilio_sms.rs ---
use axum::{extract::State, http::StatusCode, response::Json};
use reqwest::Client;
use std::sync::Arc;

use connectify_config::AppConfig;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams, utoipa::ToSchema))]
pub struct SmsRequest {
    pub to: String,
    pub message: String,
}

#[derive(Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams, utoipa::ToSchema))]
pub struct SmsResponse {
    pub success: bool,
    pub message: String,
}

pub async fn send_sms(
    State(config): State<Arc<AppConfig>>,
    Json(request): Json<SmsRequest>,
) -> Result<Json<SmsResponse>, (StatusCode, String)> {
    let twilio_config = config.twilio.as_ref().unwrap();
    if !config.use_twilio {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Twilio disabled".into()));
    }

    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        twilio_config.account_sid
    );

    // Use lowercase keys
    let params = [
        ("To", request.to.as_str()),
        ("From", twilio_config.phone_number.as_str()),
        ("Body", request.message.as_str()),
    ];
    info!("Sending SMS to {}: {}", &request.to, &request.message);
    let resp = Client::new()
        // 👇 **account_sid** + **auth_token** here
        .post(&url)
        .basic_auth(&twilio_config.account_sid, Some(&twilio_config.auth_token))
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("HTTP error sending SMS: {}", e),
            )
        })?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        // Bubble up the Twilio JSON error so you can debug
        tracing::error!("Twilio returned {}: {}", status, body);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Twilio error {}: {}", status, body),
        ));
    }

    tracing::info!("SMS sent to {}: {}", request.to, request.message);
    Ok(Json(SmsResponse {
        success: true,
        message: "SMS sent successfully".into(),
    }))
}
