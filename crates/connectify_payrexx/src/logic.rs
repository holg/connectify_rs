// --- File: crates/connectify_payrexx/src/logic.rs ---

use reqwest::Client;
use serde::{Deserialize, Serialize};
use connectify_config::{PayrexxConfig}; // Use config types
use chrono::Utc;
use thiserror::Error;
use once_cell::sync::Lazy;

// --- Error Handling ---
#[derive(Error, Debug)]
pub enum PayrexxError {
    #[error("Payrexx API request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Payrexx API returned an error: Status={status}, Message='{message}'")]
    ApiError { status: String, message: String },
    #[error("Failed to parse Payrexx API response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Payrexx configuration missing or incomplete")]
    ConfigError,
    #[error("Internal processing error: {0}")]
    InternalError(String),
}

// --- Static HTTP Client ---
// Initialize reqwest client lazily and store it statically
// This client will be reused for all Payrexx API calls within this crate
static HTTP_CLIENT: Lazy<Client> = Lazy::new(Client::new);

// --- Data Structures ---
/// Represents a request received by our backend to create a Payrexx Gateway.
#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateGatewayRequest {
    pub amount_override: Option<i64>,
    pub currency_override: Option<String>,
    pub purpose_override: Option<String>,
    pub user_email: Option<String>,
    // pub reference_id: Option<String>, // Example if needed from frontend
}

/// Represents the payload structure sent TO the Payrexx Gateway API.
#[derive(Serialize, Debug)]
struct PayrexxApiRequest<'a> {
    amount: i64,
    currency: &'a str,
    purpose: &'a str,
    #[serde(rename = "referenceId")]
    reference_id: String,
    #[serde(rename = "successRedirectUrl")]
    success_redirect_url: String,
    #[serde(rename = "failedRedirectUrl")]
    failed_redirect_url: String,
    #[serde(rename = "cancelRedirectUrl")]
    cancel_redirect_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<PayrexxFields<'a>>,
}

// Optional fields to send to Payrexx
#[derive(Serialize, Debug)]
struct PayrexxFields<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<PayrexxEmailField<'a>>,
}
#[derive(Serialize, Debug)]
struct PayrexxEmailField<'a> {
    value: &'a str,
}


/// Represents the `data` object within the Payrexx API success response.
#[derive(Deserialize, Debug)]
struct PayrexxApiResponseData {
    link: String,
}

/// Represents the overall structure of the Payrexx API response (success or error).
#[derive(Deserialize, Debug)]
struct PayrexxApiResponse {
    status: String,
    #[serde(default)]
    data: Vec<PayrexxApiResponseData>,
    message: Option<String>,
}

/// Represents the successful response sent back TO our frontend.
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateGatewayResponse {
    pub url: String,
}


// --- Core Logic Function ---

/// Makes a request to the Payrexx API to create a payment gateway (payment link).
// ** REMOVED http_client parameter **
pub async fn create_gateway_request(
    config: &PayrexxConfig, // Pass specific Payrexx config
    request_data: CreateGatewayRequest, // Data from frontend request
) -> Result<CreateGatewayResponse, PayrexxError> {

    println!("Initiating Payrexx gateway creation...");

    // Determine final values, using overrides or defaults from config
    let amount = request_data.amount_override.unwrap_or(config.unit_amount.unwrap_or(1000));
    let currency = request_data.currency_override.as_deref().unwrap_or(&config.currency.as_deref().unwrap_or("CHF"));
    let purpose = request_data.purpose_override.as_deref().unwrap_or(&config.product_name.as_deref().unwrap_or("Payment"));

    // Generate a unique reference ID for this transaction
    let reference_id = format!("connectify-{}-{}", SERVICE_NAME, Utc::now().timestamp_millis()); // TODO: Define SERVICE_NAME or use better ID

    // Construct Payrexx API payload
    let api_payload = PayrexxApiRequest {
        amount,
        currency,
        purpose,
        reference_id,
        success_redirect_url: config.success_url.clone(),
        failed_redirect_url: config.failed_url.clone(),
        cancel_redirect_url: config.cancel_url.clone(),
        fields: request_data.user_email.as_deref().map(|email| PayrexxFields {
            email: Some(PayrexxEmailField { value: email }),
        }),
    };

    // Construct Payrexx API URL
    let api_url = format!(
        "https://api.payrexx.com/v1.0/Gateway/?instance={}",
        config.instance_name
    );

    println!("Sending request to Payrexx API: {}", api_url);

    // --- Make the API Call ---
    // Get API Secret securely from environment
    let api_secret = std::env::var("PAYREXX_API_SECRET")
        .map_err(|_| PayrexxError::ConfigError)?;

    // ** Use the static HTTP_CLIENT **
    let response = HTTP_CLIENT // Use the static client directly
        .post(&api_url)
        .header("Api-Secret", api_secret)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&api_payload)
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    println!("Payrexx API response status: {}", status);

    // --- Parse Response ---
    if status.is_success() {
        let payrexx_response: PayrexxApiResponse = serde_json::from_str(&body_text)?;

        if payrexx_response.status == "success" {
            if let Some(gateway_data) = payrexx_response.data.first() {
                println!("Payrexx gateway created successfully. Link: {}", gateway_data.link);
                Ok(CreateGatewayResponse { url: gateway_data.link.clone() })
            } else {
                eprintln!("Payrexx API success status but missing data/link in response.");
                Err(PayrexxError::InternalError("Payrexx response missing gateway link".to_string()))
            }
        } else {
            let error_message = payrexx_response.message.unwrap_or_else(|| "Unknown Payrexx API error".to_string());
            eprintln!("Payrexx API reported error. Status: {}, Message: {}", payrexx_response.status, error_message);
            Err(PayrexxError::ApiError { status: payrexx_response.status, message: error_message })
        }
    } else {
        eprintln!("Payrexx API request failed with HTTP status: {}. Body: {}", status, body_text);
        Err(PayrexxError::ApiError { status: status.to_string(), message: body_text })
    }
}

// Placeholder for service name or better reference ID generation
const SERVICE_NAME: &str = "connectify_payrexx";

