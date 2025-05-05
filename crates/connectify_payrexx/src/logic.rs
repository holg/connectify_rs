// --- File: crates/connectify_payrexx/src/logic.rs ---

use chrono::Utc;
use connectify_config::PayrexxConfig; // Use config types from connectify_config
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error; // Use BTreeMap for ordered params for signing

// Signature generation imports
use base64::{engine::general_purpose::STANDARD as base64_engine, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

// URL encoding import
use serde_urlencoded;

// Conditionally import ToSchema if openapi feature is enabled
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

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
    #[error("Webhook signature verification failed")]
    WebhookSignatureError,
    #[error("Webhook processing error: {0}")]
    WebhookProcessingError(String),
    #[error("Failed to encode request body: {0}")]
    EncodingError(String),
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
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateGatewayRequest {
    #[cfg_attr(feature = "openapi", schema(example = 5000))]
    pub amount_override: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(example = "CHF"))]
    pub currency_override: Option<String>,
    #[cfg_attr(feature = "openapi", schema(example = "Booking Fee - Room XYZ"))]
    pub purpose_override: Option<String>,
    #[cfg_attr(feature = "openapi", schema(example = "customer@example.com"))]
    pub user_email: Option<String>,
}

// --- Structures for Payrexx API Payload (for Form Encoding) ---

/// Represents an item within the 'basket' array for Payrexx API.
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))] // Add schema if needed
struct PayrexxBasketItem<'a> {
    name: &'a str,
    description: Option<&'a str>,
    quantity: i64,
    amount: i64, // Amount for one item in cents/rappen
    #[serde(rename = "vatRate", skip_serializing_if = "Option::is_none")]
    vat_rate: Option<f64>,
}

// The top-level 'fields' object
#[derive(Serialize, Debug, Default)]
struct PayrexxFields<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<PayrexxEmailField<'a>>,
    // Add other 'fields' like firstname, lastname if needed
}

// Structure for fields that just have a 'value'
#[derive(Serialize, Debug)]
struct PayrexxValueField<'a> {
    value: &'a str,
}
// Specific struct for email for clarity (same as ValueField)
type PayrexxEmailField<'a> = PayrexxValueField<'a>;

// --- Structures for Payrexx API Response (Gateway Creation) ---
#[derive(Deserialize, Debug)]
struct PayrexxApiResponseData {
    link: String,
}
#[derive(Deserialize, Debug)]
struct PayrexxApiResponse {
    status: String,
    #[serde(default)]
    data: Vec<PayrexxApiResponseData>,
    message: Option<String>,
}

// --- Structure for Response to our Frontend ---
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateGatewayResponse {
    #[cfg_attr(
        feature = "openapi",
        schema(example = "https://INSTANCE.payrexx.com/pay?tid=XYZ123")
    )]
    pub url: String,
}

// --- Webhook Payload Structures ---
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PayrexxWebhookContact {
    pub id: Option<i64>,
    pub zip: Option<String>,
    pub uuid: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub place: Option<String>,
    pub title: Option<String>,
    pub street: Option<String>,
    pub company: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "countryISO")]
    pub country_iso: Option<String>,
    pub lastname: Option<String>,
    pub firstname: Option<String>,
    pub date_of_birth: Option<String>,
    #[serde(default)]
    pub delivery_zip: Option<String>,
    #[serde(default)]
    pub delivery_place: Option<String>,
    #[serde(default)]
    pub delivery_title: Option<String>,
    #[serde(default)]
    pub delivery_street: Option<String>,
    #[serde(default)]
    pub delivery_company: Option<String>,
    #[serde(default)]
    pub delivery_country: Option<String>,
    #[serde(default)]
    pub delivery_lastname: Option<String>,
    #[serde(default)]
    pub delivery_firstname: Option<String>,
    #[serde(default, rename = "delivery_countryISO")]
    pub delivery_country_iso: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PayrexxWebhookInvoiceProduct {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub price: Option<i64>,
    #[serde(rename = "vatRate")]
    pub vat_rate: Option<f64>,
    pub quantity: Option<i64>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PayrexxWebhookCustomField {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub field_type: Option<String>,
    pub value: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PayrexxWebhookInvoice {
    pub test: Option<i64>,
    pub number: Option<String>,
    pub currency: Option<String>,
    #[serde(default)]
    pub discount: Option<serde_json::Value>,
    #[serde(default)]
    pub products: Vec<PayrexxWebhookInvoiceProduct>,
    #[serde(rename = "paymentLink")]
    pub payment_link: Option<String>,
    #[serde(rename = "referenceId")]
    pub reference_id: Option<String>,
    #[serde(default, rename = "custom_fields")]
    pub custom_fields: Vec<PayrexxWebhookCustomField>,
    #[serde(rename = "originalAmount")]
    pub original_amount: Option<i64>,
    #[serde(rename = "refundedAmount")]
    pub refunded_amount: Option<i64>,
    #[serde(rename = "shippingAmount")]
    pub shipping_amount: Option<i64>,
    #[serde(rename = "paymentRequestId")]
    pub payment_request_id: Option<i64>,
    #[serde(rename = "googleAnalyticProducts", default)]
    pub google_analytic_products: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PayrexxWebhookPayment {
    pub brand: Option<String>,
    pub wallet: Option<String>,
    #[serde(rename = "purchaseOnInvoiceInformation")]
    pub purchase_on_invoice_information: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PayrexxWebhookInstance {
    pub name: Option<String>,
    pub uuid: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PayrexxWebhookTransaction {
    pub id: Option<i64>,
    pub psp: Option<serde_json::Value>,
    pub lang: Option<String>,
    pub mode: Option<String>,
    pub time: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: Option<String>,
    pub uuid: Option<String>,
    #[serde(rename = "pspId")]
    pub psp_id: Option<i64>,
    pub amount: Option<i64>,
    pub status: Option<String>,
    pub contact: Option<PayrexxWebhookContact>,
    pub invoice: Option<PayrexxWebhookInvoice>,
    pub payment: Option<PayrexxWebhookPayment>,
    pub instance: Option<PayrexxWebhookInstance>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(rename = "pageUuid")]
    pub page_uuid: Option<String>,
    #[serde(rename = "payoutUuid")]
    pub payout_uuid: Option<String>,
    #[serde(rename = "payrexxFee")]
    pub payrexx_fee: Option<i64>,
    pub refundable: Option<bool>,
    #[serde(rename = "referenceId")]
    pub reference_id: Option<String>,
    pub subscription: Option<serde_json::Value>,
    #[serde(rename = "posSerialNumber", default)]
    pub pos_serial_number: String,
    #[serde(rename = "posTerminalName", default)]
    pub pos_terminal_name: String,
    #[serde(rename = "partiallyRefundable")]
    pub partially_refundable: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PayrexxWebhookPayload {
    #[serde(default)]
    pub transaction: Option<PayrexxWebhookTransaction>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
}

// --- Core Logic Functions ---

/// Generates the HMAC-SHA256 signature required by Payrexx API.
fn generate_payrexx_signature(query_string: &str, api_secret: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    // Ensure secret is treated as bytes
    let mut mac =
        HmacSha256::new_from_slice(api_secret.as_bytes()).expect("HMAC can take key of any size"); // Handle error in production
    mac.update(query_string.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    // Encode signature using standard base64
    base64_engine.encode(code_bytes)
}

/// Makes a request to the Payrexx API to create a payment gateway using form encoding and signature.
pub async fn create_gateway_request(
    config: &PayrexxConfig,
    request_data: CreateGatewayRequest,
) -> Result<CreateGatewayResponse, PayrexxError> {
    println!("Initiating Payrexx gateway creation (Form Encoded)...");

    // Determine final values
    let amount = request_data
        .amount_override
        .unwrap_or(config.unit_amount.unwrap_or(1000));
    let currency = request_data
        .currency_override
        .as_deref()
        .unwrap_or(&config.currency.as_deref().unwrap_or("CHF"));
    let purpose = request_data
        .purpose_override
        .as_deref()
        .unwrap_or(&config.product_name.as_deref().unwrap_or("Payment"));
    let reference_id = format!(
        "connectify-{}-{}",
        SERVICE_NAME,
        Utc::now().timestamp_millis()
    );

    // --- Prepare data for form encoding using BTreeMap for ordered keys ---
    let mut form_params: BTreeMap<String, String> = BTreeMap::new();

    // Add required fields (convert numbers to strings)
    form_params.insert("amount".to_string(), amount.to_string());
    form_params.insert("currency".to_string(), currency.to_string());
    form_params.insert("purpose".to_string(), purpose.to_string());
    form_params.insert("referenceId".to_string(), reference_id.clone()); // Clone reference_id for use here
    form_params.insert("successRedirectUrl".to_string(), config.success_url.clone());
    form_params.insert("failedRedirectUrl".to_string(), config.failed_url.clone());
    form_params.insert("cancelRedirectUrl".to_string(), config.cancel_url.clone());

    // Add optional fields if present
    if let Some(email) = request_data.user_email.as_deref() {
        // Flatten 'fields' according to Payrexx convention (verify exact key format)
        form_params.insert("fields[email][value]".to_string(), email.to_string());
    }
    // TODO: Add other optional fields like basket, pm, psp, preAuthorization etc.
    //       to the form_params BTreeMap here if needed.

    // --- Generate Signature ---
    // URL-encode the parameters *before* signing
    let query_string_for_sig = serde_urlencoded::to_string(&form_params).map_err(|e| {
        PayrexxError::EncodingError(format!("Failed to urlencode params for signature: {}", e))
    })?;

    let api_secret = std::env::var("PAYREXX_API_SECRET").map_err(|_| PayrexxError::ConfigError)?;
    let api_signature = generate_payrexx_signature(&query_string_for_sig, &api_secret);

    // Add signature to parameters AFTER encoding the rest for the signature base string
    form_params.insert("ApiSignature".to_string(), api_signature);

    // --- Prepare Final Request ---
    // URL-encode the full set of parameters including the signature for the request body
    let final_request_body = serde_urlencoded::to_string(&form_params).map_err(|e| {
        PayrexxError::EncodingError(format!("Failed to urlencode final params: {}", e))
    })?;

    // Construct Payrexx API URL
    let api_url = format!(
        "https://api.payrexx.com/v1.0/Gateway/?instance={}",
        config.instance_name
    );

    println!("Sending POST request to Payrexx API: {}", api_url);

    // --- Make the API Call ---
    let response = HTTP_CLIENT
        .post(&api_url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        ) // Set correct content type
        .body(final_request_body) // Send urlencoded string as body
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    println!("Payrexx API response status: {}", status);
    if !status.is_success() {
        println!("Payrexx API response body (raw): {}", body_text);
    }

    // --- Parse Response (Still JSON) ---
    if status.is_success() {
        let payrexx_response: PayrexxApiResponse = serde_json::from_str(&body_text)?;

        if payrexx_response.status == "success" {
            if let Some(gateway_data) = payrexx_response.data.first() {
                println!(
                    "Payrexx gateway created successfully. Link: {}",
                    gateway_data.link
                );
                Ok(CreateGatewayResponse {
                    url: gateway_data.link.clone(),
                })
            } else {
                eprintln!("Payrexx API success status but missing data/link in response.");
                Err(PayrexxError::InternalError(
                    "Payrexx response missing gateway link".to_string(),
                ))
            }
        } else {
            let error_message = payrexx_response
                .message
                .unwrap_or_else(|| "Unknown Payrexx API error".to_string());
            eprintln!(
                "Payrexx API reported error. Status: {}, Message: {}",
                payrexx_response.status, error_message
            );
            Err(PayrexxError::ApiError {
                status: payrexx_response.status,
                message: error_message,
            })
        }
    } else {
        eprintln!(
            "Payrexx API request failed with HTTP status: {}. Body: {}",
            status, body_text
        );
        let message = match serde_json::from_str::<PayrexxApiResponse>(&body_text) {
            Ok(err_resp) => err_resp.message.unwrap_or(body_text),
            Err(_) => body_text,
        };
        Err(PayrexxError::ApiError {
            status: status.to_string(),
            message,
        })
    }
}

// --- Webhook Processing Logic ---

/// Verifies the signature of an incoming Payrexx webhook request.
/// Placeholder implementation - requires details from Payrexx documentation.
pub fn verify_payrexx_signature(
    _api_secret: &str,
    _request_body: &[u8],
    _signature_header: Option<&str>,
) -> Result<(), PayrexxError> {
    // TODO: Implement actual signature verification based on Payrexx docs.

    println!("⚠️ WARNING: Payrexx webhook signature verification is NOT implemented!");
    Ok(())
}

/// Processes a verified Payrexx webhook payload.
pub async fn process_webhook(
    payload: PayrexxWebhookPayload,
    // Add other dependencies if needed (e.g., database access, GCal client)
    // db_pool: &SqlitePool,
) -> Result<(), PayrexxError> {
    println!("Processing webhook event type: {:?}", payload.event_type);

    if let Some(transaction) = payload.transaction {
        println!(
            "Transaction ID: {:?}, Status: {:?}",
            transaction.id, transaction.status
        );

        // Handle based on transaction status
        match transaction.status.as_deref() {
            Some("confirmed") => {
                println!(
                    "✅ Payment confirmed for reference: {:?}",
                    transaction.reference_id
                );
                // TODO: Implement actions for successful payment
            }
            Some("waiting") => {
                println!("⏳ Payment waiting for confirmation.");
            }
            Some("cancelled") => {
                println!(
                    "❌ Payment cancelled for reference: {:?}",
                    transaction.reference_id
                );
                // TODO: Update order status if needed.
            }
            Some("failed") => {
                println!(
                    "❌ Payment failed for reference: {:?}",
                    transaction.reference_id
                );
                // TODO: Update order status if needed.
            }
            Some(other_status) => {
                println!("ℹ️ Received unhandled transaction status: {}", other_status);
            }
            None => {
                println!("⚠️ Webhook received without transaction status.");
            }
        }
    } else {
        println!("⚠️ Webhook received without transaction data.");
    }

    Ok(())
}

// Placeholder for service name or better reference ID generation
const SERVICE_NAME: &str = "connectify_payrexx";
