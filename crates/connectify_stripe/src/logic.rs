// --- File: crates/connectify_stripe/src/logic.rs ---
use reqwest::Client;
use serde::{Deserialize, Serialize};
use connectify_config::{StripeConfig};
use thiserror::Error;
use once_cell::sync::Lazy;
use std::{env, collections::{HashMap}, time::{SystemTime, UNIX_EPOCH}};
use hmac::{Hmac, Mac};
use sha2::Sha256;
#[allow(unused_imports)]
#[cfg(feature = "openapi")] use serde_json::json;

// Conditionally import ToSchema if openapi feature is enabled
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// --- Error Handling ---
#[derive(Error, Debug)]
pub enum StripeError {
    #[error("Stripe API request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Stripe API returned an error: {message} (Status: {status_code})")]
    ApiError { status_code: u16, message: String },
    #[error("Failed to parse Stripe API response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Stripe configuration missing or incomplete")]
    ConfigError,
    #[error("Stripe webhook signature verification failed: {0}")] // For webhook errors
    WebhookSignatureError(String),
    #[error("Stripe webhook event processing error: {0}")] // For webhook errors
    WebhookProcessingError(String),
    #[error("Internal processing error: {0}")]
    InternalError(String),
}

// --- Static HTTP Client ---
static HTTP_CLIENT: Lazy<Client> = Lazy::new(Client::new);

// --- Data Structures ---

/// Request from our frontend to create a Stripe Checkout Session.
// ** Added openapi derive **
#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateCheckoutSessionRequest {
    #[cfg_attr(feature = "openapi", schema(example = "Room Booking"))]
    pub product_name_override: Option<String>,
    #[cfg_attr(feature = "openapi", schema(example = 5000))]
    pub amount_override: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(example = "CHF"))]
    pub currency_override: Option<String>,
    #[cfg_attr(feature = "openapi", schema(example = json!({"user_id": "user123", "booking_ref": "ref789"})))]
    pub metadata: Option<serde_json::Value>,
}

/// Represents the relevant part of the response FROM the Stripe API.
// This is an internal struct, OpenAPI derive not strictly needed unless exposed
#[derive(Deserialize, Debug)]
struct StripeCheckoutSessionResponse {
    pub id: String,
    pub url: Option<String>,
}

/// Represents the outer Stripe Event object.
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateCheckoutSessionResponse {
    #[cfg_attr(feature = "openapi", schema(example = "https://checkout.stripe.com/pay/cs_test_a1..."))]
    pub url: String,
    #[cfg_attr(feature = "openapi", schema(example = "cs_test_a1..."))]
    pub session_id: String,
}

// --- Core Logic Function ---

/// Represents the `data` field within a Stripe Event.
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StripeEventData {
    /// The actual object related to the event, e.g., a Checkout Session, Charge, etc.
    /// Using serde_json::Value because the structure of 'object' varies by event type.
    pub object: serde_json::Value,
    // previous_attributes: Option<serde_json::Value>, // If needed for some event types
}

/// Represents the `request` field within a Stripe Event (useful for idempotency).
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StripeEventRequest {
    pub id: Option<String>, // Request ID
    pub idempotency_key: Option<String>, // Idempotency key used for the request
}
/// Represents the outer Stripe Event object.
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StripeEvent {
    pub id: String,
    pub object: String, // "event"
    pub api_version: Option<String>,
    pub created: i64, // Unix timestamp
    pub livemode: bool,
    #[serde(rename = "type")]
    pub event_type: String, // e.g., "checkout.session.completed"
    pub data: StripeEventData,
    pub request: Option<StripeEventRequest>
}
/// Specific structure for the `data.object` when event_type is "checkout.session.completed".
/// Define fields you care about.
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StripeCheckoutSessionObject {
    pub id: String, // Checkout Session ID (cs_...)
    pub object: String, // "checkout.session"
    pub amount_total: Option<i64>, // Total amount in cents
    pub currency: Option<String>,
    pub customer: Option<String>, // Customer ID (cus_...) if created
    pub customer_details: Option<StripeCustomerDetails>,
    pub metadata: Option<HashMap<String, String>>, // Metadata you passed
    pub payment_intent: Option<String>, // Payment Intent ID (pi_...)
    pub payment_status: Option<String>, // e.g., "paid", "unpaid", "no_payment_required"
    pub status: Option<String>, // e.g., "open", "complete", "expired"
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
    pub client_reference_id: Option<String>
}
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StripeCustomerDetails {
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    // address: Option<StripeAddress>, // Define StripeAddress if needed
}

// --- Webhook Processing Logic ---

/// Verifies the signature of an incoming Stripe webhook request.
///
/// # Arguments
/// * `payload_bytes` - The raw request body bytes.
/// * `sig_header` - The value of the 'Stripe-Signature' header.
/// * `secret` - Your Stripe webhook signing secret (whsec_...).
///
/// Returns Ok(()) if the signature is valid, otherwise StripeError::WebhookSignatureError.
pub fn verify_stripe_signature(
    payload_bytes: &[u8],
    sig_header: Option<&str>,
    secret: &str,
) -> Result<(), StripeError> {
    let sig_header_value = sig_header
        .ok_or_else(|| StripeError::WebhookSignatureError("Missing Stripe-Signature header".to_string()))?;
    // Debug: Log the received signature header
    println!("[DEBUG] Stripe-Signature Header: {}", sig_header_value);
    let mut timestamp_str: Option<&str> = None;
    let mut v1_signatures_hex: Vec<&str> = Vec::new();

    for item in sig_header_value.split(',') {
        let parts: Vec<&str> = item.trim().splitn(2, '=').collect();
        if parts.len() == 2 {
            match parts[0] {
                "t" => timestamp_str = Some(parts[1]),
                "v1" => v1_signatures_hex.push(parts[1]), // Add to list
                _ => {} // Ignore other parts like v0
            }
        }
    }

    let parsed_timestamp = timestamp_str
        .ok_or_else(|| StripeError::WebhookSignatureError("Missing timestamp 't' in Stripe-Signature".to_string()))?
        .parse::<i64>()
        .map_err(|_| StripeError::WebhookSignatureError("Invalid timestamp format in Stripe-Signature".to_string()))?;

    if v1_signatures_hex.is_empty() {
        return Err(StripeError::WebhookSignatureError("Missing v1 signature in Stripe-Signature".to_string()));
    }
    // Debug: Log parsed components
    println!("[DEBUG] Parsed Timestamp (t): {}", parsed_timestamp);
    println!("[DEBUG] Provided Signatures (v1 list): {:?}", v1_signatures_hex);

    // Check timestamp tolerance (e.g., 10 minutes)
    let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64;
    const TOLERANCE_SECONDS: i64 = 600; // 10 minutes
    if (current_timestamp - parsed_timestamp).abs() > TOLERANCE_SECONDS {
        eprintln!("[WARN] Timestamp outside tolerance. Current: {}, Event: {}, Diff: {}",
                  current_timestamp, parsed_timestamp, (current_timestamp - parsed_timestamp).abs());
        // return Err(StripeError::WebhookSignatureError("Timestamp outside tolerance".to_string())); // Consider re-enabling for production
    }

    // Construct the signed payload string
    let signed_payload_string = format!("{}.{}",
                                        timestamp_str.unwrap(), // Use the original string timestamp from header
                                        String::from_utf8_lossy(payload_bytes)
    );
    println!("[DEBUG] String to Sign: '{}'", signed_payload_string);

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| StripeError::WebhookSignatureError("Invalid webhook secret format for HMAC".to_string()))?;
    mac.update(signed_payload_string.as_bytes());
    let expected_signature_bytes = mac.finalize().into_bytes();
    let calculated_signature_hex = hex::encode(expected_signature_bytes);

    // Debug: Log calculated signature
    println!("[DEBUG] Calculated Signature: {}", calculated_signature_hex);

    // ** Iterate through all provided v1 signatures and check for a match **
    for provided_sig_hex in v1_signatures_hex {
        if constant_time_eq(calculated_signature_hex.as_bytes(), provided_sig_hex.as_bytes()) {
            return Ok(()); // Signature matches one of the v1 signatures
        }
    }
    // If no match was found after checking all v1 signatures
    eprintln!("[ERROR] Stripe signature mismatch. Calculated: {}, Provided in header did not match.", calculated_signature_hex);
    Err(StripeError::WebhookSignatureError("Signature mismatch".to_string()))
}

/// Helper for constant-time string comparison.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}
/// Processes a verified Stripe webhook event.
pub async fn process_stripe_webhook(
    event: StripeEvent,
    // Add other dependencies if needed (e.g., database access)
    // db_pool: &SqlitePool,
) -> Result<(), StripeError> {
    println!("Processing Stripe event type: {}", event.event_type);

    match event.event_type.as_str() {
        "checkout.session.completed" => {
            let session: StripeCheckoutSessionObject = serde_json::from_value(event.data.object)
                .map_err(|e| StripeError::WebhookProcessingError(format!("Failed to parse checkout session object: {}", e)))?;

            println!("Checkout Session ID: {}", session.id);
            println!("Payment Status: {:?}", session.payment_status);
            println!("Metadata: {:?}", session.metadata);
            println!("Client Reference ID: {:?}", session.client_reference_id);

            if session.payment_status.as_deref() == Some("paid") {
                println!("✅ Payment for Checkout Session {} was successful.", session.id);
                // TODO: FULFILLMENT LOGIC:
                // 1. Use session.id or session.client_reference_id or session.metadata
                //    to identify the order/booking in your system.
                // 2. Mark the order as paid in your database.
                // 3. Trigger any necessary fulfillment actions (e.g., grant access, send email, book GCal event).
                // 4. Ensure this processing is idempotent (Stripe might retry webhooks).
                //    Check if you've already processed this event.id before.
            } else {
                println!("ℹ️ Checkout session {} completed, but payment status is: {:?}. No fulfillment action taken.", session.id, session.payment_status);
            }
        }
        "payment_intent.succeeded" => {
            let payment_intent_id: Option<&str> = event.data.object.get("id").and_then(|v| v.as_str());
            println!("PaymentIntent succeeded: {:?}", payment_intent_id);
            // You might handle this if you're working directly with PaymentIntents
        }
        "payment_intent.payment_failed" => {
            let payment_intent_id: Option<&str> = event.data.object.get("id").and_then(|v| v.as_str());
            println!("PaymentIntent failed: {:?}", payment_intent_id);
            // Handle failed payment attempts if necessary
        }
        // Add more event types as needed
        _ => {
            println!("Received unhandled Stripe event type: {}", event.event_type);
        }
    }
    Ok(())
}

/// Creates a Stripe Checkout Session.
pub async fn create_checkout_session(
    stripe_config: &StripeConfig,
    request_data: CreateCheckoutSessionRequest,
) -> Result<CreateCheckoutSessionResponse, StripeError> {
    println!("Initiating Stripe Checkout Session creation...");

    // Load Stripe Secret Key directly from environment for security
    let stripe_secret_key = env::var("STRIPE_SECRET_KEY")
        .map_err(|_| StripeError::ConfigError)?;

    // Determine final values for the session
    let product_name = request_data.product_name_override
        .as_deref()
        .or(stripe_config.product_name.as_deref())
        .unwrap_or("Service/Product");

    let unit_amount = request_data.amount_override
        .or(stripe_config.unit_amount)
        .unwrap_or(150); // TODO use configured value or default

    let currency = request_data.currency_override
        .as_deref()
        .or(stripe_config.currency.as_deref())
        .unwrap_or("chf")// TODO use configured value or default
        .to_lowercase();

    // Prepare parameters for Stripe API call (form-urlencoded)
    let mut form_body: Vec<(String, String)> = Vec::new();
    form_body.push(("payment_method_types[]".to_string(), "card".to_string()));
    form_body.push(("mode".to_string(), "payment".to_string()));
    form_body.push(("success_url".to_string(), stripe_config.success_url.clone()));
    form_body.push(("cancel_url".to_string(), stripe_config.cancel_url.clone()));

    form_body.push(("line_items[0][price_data][currency]".to_string(), currency));
    form_body.push(("line_items[0][price_data][product_data][name]".to_string(), product_name.to_string()));
    form_body.push(("line_items[0][price_data][unit_amount]".to_string(), unit_amount.to_string()));
    form_body.push(("line_items[0][quantity]".to_string(), "1".to_string()));

    if let Some(metadata_val) = request_data.metadata {
        if let Some(obj) = metadata_val.as_object() {
            for (key, value) in obj {
                if let Some(str_val) = value.as_str() {
                    form_body.push((format!("metadata[{}]", key), str_val.to_string()));
                }
                // Add handling for other metadata value types if needed (e.g., numbers)
            }
        }
    }

    let api_url = "https://api.stripe.com/v1/checkout/sessions";

    println!("Sending request to Stripe API: {}", api_url);
    // For debugging form body:
    println!("Form Body: {:?}", form_body);

    let response = HTTP_CLIENT
        .post(api_url)
        .basic_auth(stripe_secret_key, None::<&str>) // Stripe uses API key as username, empty password
        .form(&form_body) // Send as form-urlencoded
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    println!("Stripe API response status: {}", status);
    if !status.is_success() {
        println!("Stripe API response body (raw): {}", body_text);
    }

    if status.is_success() {
        let stripe_response: StripeCheckoutSessionResponse = serde_json::from_str(&body_text)?;
        if let Some(url) = stripe_response.url {
            println!("Stripe Checkout Session created successfully. URL: {}", url);
            Ok(CreateCheckoutSessionResponse { url, session_id: stripe_response.id })
        } else {
            eprintln!("Stripe response missing checkout session URL: {}", body_text);
            Err(StripeError::InternalError("Stripe response missing checkout URL".to_string()))
        }
    } else {
        let error_message = match serde_json::from_str::<serde_json::Value>(&body_text) {
            Ok(json_body) => {
                json_body.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str())
                    .unwrap_or(&body_text).to_string()
            }
            Err(_) => body_text,
        };
        eprintln!("Stripe API request failed with HTTP status: {}. Message: {}", status, error_message);
        Err(StripeError::ApiError { status_code: status.as_u16(), message: error_message })
    }
}