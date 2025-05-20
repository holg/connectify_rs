// --- File: crates/connectify_stripe/src/logic.rs ---
use chrono::{DateTime, Utc};
use connectify_config::{AppConfig, StripeConfig}; //, PriceTier};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
#[cfg(feature = "openapi")]
use serde_json::json;
use sha2::Sha256;
use std::{
    collections::HashMap,
    env,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::{error, info};

// Import the StripeError from the error module
use crate::error::StripeError;

// Import the HTTP client from connectify_common
use connectify_common::HTTP_CLIENT;

// Conditionally import ToSchema if openapi feature is enabled
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

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
    // --- Fulfillment Information ---
    /// Type of fulfillment to trigger (e.g., "gcal_booking", "twilio_session_setup")
    #[cfg_attr(feature = "openapi", schema(example = "gcal_booking"))]
    pub fulfillment_type: String,
    /// JSON data specific to the fulfillment_type
    #[cfg_attr(feature = "openapi", schema(example = json!({
        "start_time": "2025-07-15T10:00:00Z",
        "end_time": "2025-07-15T11:00:00Z",
        "summary": "Consultation via Stripe",
        "description": "Details discussed during checkout."
    })))]
    pub fulfillment_data: serde_json::Value,

    // Stripe's client_reference_id can also be used to link to your internal order
    #[cfg_attr(feature = "openapi", schema(example = "my_internal_order_123"))]
    pub client_reference_id: Option<String>,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct StripeCheckoutSessionResponse {
    pub id: String,
    pub url: Option<String>,
}

/// Represents the outer Stripe Event object.
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateCheckoutSessionResponse {
    #[cfg_attr(
        feature = "openapi",
        schema(example = "https://checkout.stripe.com/pay/cs_test_a1...")
    )]
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
    pub id: Option<String>,              // Request ID
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
    pub request: Option<StripeEventRequest>,
}
/// Specific structure for the `data.object` when event_type is "checkout.session.completed".
/// Define fields you care about.
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StripeCheckoutSessionObject {
    pub id: String,                // Checkout Session ID (cs_...)
    pub object: String,            // "checkout.session"
    pub amount_total: Option<i64>, // Total amount in cents
    pub currency: Option<String>,
    pub customer: Option<String>, // Customer ID (cus_...) if created
    pub customer_details: Option<StripeCustomerDetails>,
    pub metadata: Option<HashMap<String, String>>, // Metadata you passed
    pub payment_intent: Option<String>,            // Payment Intent ID (pi_...)
    pub payment_status: Option<String>,            // e.g., "paid", "unpaid", "no_payment_required"
    pub status: Option<String>,                    // e.g., "open", "complete", "expired"
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
    pub client_reference_id: Option<String>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
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
    let sig_header_value = sig_header.ok_or_else(|| {
        StripeError::WebhookSignatureError("Missing Stripe-Signature header".to_string())
    })?;
    // Debug: Log the received signature header
    info!("[DEBUG] Stripe-Signature Header: {}", sig_header_value);
    let mut timestamp_str: Option<&str> = None;
    let mut v1_signatures_hex: Vec<&str> = Vec::new();

    for item in sig_header_value.split(',') {
        let parts: Vec<&str> = item.trim().splitn(2, '=').collect();
        if parts.len() == 2 {
            match parts[0] {
                "t" => timestamp_str = Some(parts[1]),
                "v1" => v1_signatures_hex.push(parts[1]), // Add to list
                _ => {}                                   // Ignore other parts like v0
            }
        }
    }

    let parsed_timestamp = timestamp_str
        .ok_or_else(|| {
            StripeError::WebhookSignatureError(
                "Missing timestamp 't' in Stripe-Signature".to_string(),
            )
        })?
        .parse::<i64>()
        .map_err(|_| {
            StripeError::WebhookSignatureError(
                "Invalid timestamp format in Stripe-Signature".to_string(),
            )
        })?;

    if v1_signatures_hex.is_empty() {
        return Err(StripeError::WebhookSignatureError(
            "Missing v1 signature in Stripe-Signature".to_string(),
        ));
    }
    // Debug: Log parsed components
    info!("[DEBUG] Parsed Timestamp (t): {}", parsed_timestamp);
    info!(
        "[DEBUG] Provided Signatures (v1 list): {:?}",
        v1_signatures_hex
    );

    // Check timestamp tolerance (e.g., 10 minutes)
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    const TOLERANCE_SECONDS: i64 = 600; // 10 minutes
    if (current_timestamp - parsed_timestamp).abs() > TOLERANCE_SECONDS {
        info!(
            "[WARN] Timestamp outside tolerance. Current: {}, Event: {}, Diff: {}",
            current_timestamp,
            parsed_timestamp,
            (current_timestamp - parsed_timestamp).abs()
        );
        // return Err(StripeError::WebhookSignatureError("Timestamp outside tolerance".to_string())); // Consider re-enabling for production
    }

    // Construct the signed payload string
    let signed_payload_string = format!(
        "{}.{}",
        timestamp_str.unwrap(), // Use the original string timestamp from header
        String::from_utf8_lossy(payload_bytes)
    );
    info!("[DEBUG] String to Sign: '{}'", signed_payload_string);

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| {
        StripeError::WebhookSignatureError("Invalid webhook secret format for HMAC".to_string())
    })?;
    mac.update(signed_payload_string.as_bytes());
    let expected_signature_bytes = mac.finalize().into_bytes();
    let calculated_signature_hex = hex::encode(expected_signature_bytes);

    // Debug: Log calculated signature
    info!("[DEBUG] Calculated Signature: {}", calculated_signature_hex);

    // ** Iterate through all provided v1 signatures and check for a match **
    for provided_sig_hex in v1_signatures_hex {
        if constant_time_eq(
            calculated_signature_hex.as_bytes(),
            provided_sig_hex.as_bytes(),
        ) {
            return Ok(()); // Signature matches one of the v1 signatures
        }
    }
    // If no match was found after checking all v1 signatures
    info!(
        "[ERROR] Stripe signature mismatch. Calculated: {}, Provided in header did not match.",
        calculated_signature_hex
    );
    Err(StripeError::WebhookSignatureError(
        "Signature mismatch".to_string(),
    ))
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
    app_config: Arc<AppConfig>,
) -> Result<(), StripeError> {
    info!("Processing Stripe event type: {}", event.event_type);

    match event.event_type.as_str() {
        "checkout.session.completed" => {
            let session: StripeCheckoutSessionObject = serde_json::from_value(event.data.object)
                .map_err(|e| {
                    StripeError::WebhookProcessingError(format!(
                        "Failed to parse checkout session object: {}",
                        e
                    ))
                })?;

            info!("Checkout Session ID: {}", session.id);
            info!("Payment Status: {:?}", session.payment_status);
            info!("Metadata: {:?}", session.metadata);
            info!("Client Reference ID: {:?}", session.client_reference_id);

            if session.payment_status.as_deref() == Some("paid") {
                info!(
                    "✅ Payment for Checkout Session {} was successful.",
                    session.id
                );
                // --- Trigger Fulfillment ---
                let metadata = session.metadata.as_ref();
                let fulfillment_type = metadata.and_then(|m| m.get("ff_type").cloned());
                let fulfillment_data_json_str =
                    metadata.and_then(|m| m.get("ff_data_json").cloned());

                if let (Some(ff_type), Some(ff_data_str)) =
                    (fulfillment_type, fulfillment_data_json_str)
                {
                    // Deserialize the ff_data_json string back into a serde_json::Value
                    let fulfillment_payload_value: serde_json::Value =
                        serde_json::from_str(&ff_data_str).map_err(|e| {
                            StripeError::WebhookProcessingError(format!(
                                "Failed to parse ff_data_json: {}",
                                e
                            ))
                        })?;

                    if let Some(fulfillment_cfg) = app_config
                        .fulfillment
                        .as_ref()
                        .and_then(|f| f.shared_secret.as_ref())
                    {
                        let fulfillment_base_url = format!(
                            // Construct base URL
                            "http://{}:{}",
                            app_config.server.host, app_config.server.port
                        );

                        // Construct specific fulfillment endpoint URL based on ff_type
                        let fulfillment_endpoint_path = match ff_type.as_str() {
                            "gcal_booking" => "/api/fulfill/gcal-booking",
                            // "twilio_session" => "/api/fulfill/twilio-session", // Example
                            "adhoc_gcal_twilio" => "/api/fulfill/adhoc-gcal-twilio",
                            // "twilio_session" => "/api/fulfill/twilio-session", // Example
                            _ => {
                                info!(
                                    "[Stripe Webhook] Unknown fulfillment_type in metadata: {}",
                                    ff_type
                                );
                                return Err(StripeError::WebhookProcessingError(format!(
                                    "Unknown fulfillment type: {}",
                                    ff_type
                                )));
                            }
                        };
                        let fulfillment_url =
                            format!("{}{}", fulfillment_base_url, fulfillment_endpoint_path);

                        info!("[Stripe Webhook] Calling fulfillment service at {} for type '{}', session {}", fulfillment_url, ff_type, session.id);

                        let client = HTTP_CLIENT.clone(); // Use the static client
                        match client
                            .post(&fulfillment_url)
                            .header("X-Internal-Auth-Secret", fulfillment_cfg) // Use the shared secret
                            .json(&fulfillment_payload_value) // Send the original JSON Value
                            .send()
                            .await
                        {
                            Ok(resp) if resp.status().is_success() => {
                                info!("[Stripe Webhook] Fulfillment for session {} (type: {}) triggered successfully.", session.id, ff_type);
                            }
                            Ok(resp) => {
                                let status = resp.status(); // Store the status before consuming the response
                                let err_text = resp.text().await.unwrap_or_else(|_| {
                                    "Unknown error from fulfillment service".to_string()
                                });
                                info!("[Stripe Webhook] Fulfillment call for session {} (type: {}) failed: {} - {}", session.id, ff_type, status, err_text);
                                return Err(StripeError::FulfillmentError(format!(
                                    "Fulfillment service call failed: {} - {}",
                                    status, err_text
                                )));
                            }
                            Err(e) => {
                                info!("[Stripe Webhook] Error calling fulfillment service for session {}: {}", session.id, e);
                                return Err(StripeError::FulfillmentError(format!(
                                    "Error calling fulfillment service: {}",
                                    e
                                )));
                            }
                        }
                    } else {
                        info!("[Stripe Webhook] Fulfillment shared secret not configured. Cannot call fulfillment service for session {}.", session.id);
                        return Err(StripeError::ConfigError);
                    }
                } else {
                    info!("[Stripe Webhook] Missing 'ff_type' or 'ff_data_json' in metadata for session {}. Cannot trigger fulfillment.", session.id);
                    // Decide if this is an error or just no fulfillment needed
                    return Err(StripeError::MissingFulfillmentData);
                }
            } else {
                info!("ℹ️ Checkout session {} completed, but payment status is: {:?}. No fulfillment action taken.", session.id, session.payment_status);
            }
        }
        "payment_intent.succeeded" => {
            let payment_intent_id: Option<&str> =
                event.data.object.get("id").and_then(|v| v.as_str());
            info!("PaymentIntent succeeded: {:?}", payment_intent_id);
            // You might handle this if you're working directly with PaymentIntents
        }
        "payment_intent.payment_failed" => {
            let payment_intent_id: Option<&str> =
                event.data.object.get("id").and_then(|v| v.as_str());
            info!("PaymentIntent failed: {:?}", payment_intent_id);
            // Handle failed payment attempts if necessary
        }
        // Add more event types as needed
        _ => {
            info!("Received unhandled Stripe event type: {}", event.event_type);
        }
    }
    Ok(())
}

/// Creates a Stripe Checkout Session.
/// Creates a Stripe Checkout Session.
pub async fn create_checkout_session(
    stripe_config: &StripeConfig,
    request_data: CreateCheckoutSessionRequest,
) -> Result<CreateCheckoutSessionResponse, StripeError> {
    info!(
        "[Stripe Logic] Creating Checkout Session for fulfillment type: {}",
        request_data.fulfillment_type
    );

    let stripe_secret_key = env::var("STRIPE_SECRET_KEY").map_err(|_| StripeError::ConfigError)?;

    // --- Determine Price and Product Name based on fulfillment_data and price_tiers ---
    let unit_amount: i64;
    let product_name: String;
    let currency: String;

    if request_data.fulfillment_type == "gcal_booking"
        || request_data.fulfillment_type == "adhoc_gcal_twilio"
    {
        let start_time_str = request_data
            .fulfillment_data
            .get("start_time")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                StripeError::InvalidFulfillmentDataForPricing(
                    "Missing start_time in fulfillment_data for GCal-based booking".to_string(),
                )
            })?;
        let end_time_str = request_data
            .fulfillment_data
            .get("end_time")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                StripeError::InvalidFulfillmentDataForPricing(
                    "Missing end_time in fulfillment_data for GCal-based booking".to_string(),
                )
            })?;

        let start_dt = DateTime::parse_from_rfc3339(start_time_str)
            .map_err(|e| {
                StripeError::InvalidFulfillmentDataForPricing(format!(
                    "Invalid start_time format: {}",
                    e
                ))
            })?
            .with_timezone(&Utc);
        let end_dt = DateTime::parse_from_rfc3339(end_time_str)
            .map_err(|e| {
                StripeError::InvalidFulfillmentDataForPricing(format!(
                    "Invalid end_time format: {}",
                    e
                ))
            })?
            .with_timezone(&Utc);

        if end_dt <= start_dt {
            return Err(StripeError::InvalidFulfillmentDataForPricing(
                "End time must be after start time.".to_string(),
            ));
        }
        let duration_minutes = (end_dt - start_dt).num_minutes();

        let tier = stripe_config
            .price_tiers
            .iter()
            .find(|t| t.duration_minutes == duration_minutes)
            .ok_or_else(|| StripeError::NoMatchingPriceTier(duration_minutes))?;

        unit_amount = tier.unit_amount;
        // Use product_name from tier, fallback to summary from fulfillment_data, then a generic default
        product_name = tier.product_name.clone().unwrap_or_else(|| {
            request_data
                .fulfillment_data
                .get("summary")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| format!("Service - {} min", duration_minutes))
        });
        currency = tier
            .currency
            .clone()
            .unwrap_or_else(|| {
                stripe_config
                    .default_currency
                    .clone()
                    .unwrap_or_else(|| "chf".to_string())
            })
            .to_lowercase();

        info!("[Stripe Logic] Type: {}. Duration: {} mins. Tier: amount={}, product='{}', currency='{}'",
                 request_data.fulfillment_type, duration_minutes, unit_amount, product_name, currency);
    } else {
        // Handle other fulfillment types or default pricing if necessary in the future
        return Err(StripeError::InvalidFulfillmentDataForPricing(format!(
            "Unsupported fulfillment_type for dynamic pricing: {}",
            request_data.fulfillment_type
        )));
    }
    // --- End Price and Product Name Determination ---
    #[allow(clippy::vec_init_then_push)]
    let mut form_body: Vec<(String, String)> = vec![
        ("payment_method_types[]".to_string(), "card".to_string()),
        ("mode".to_string(), "payment".to_string()),
        ("success_url".to_string(), stripe_config.success_url.clone()),
        ("cancel_url".to_string(), stripe_config.cancel_url.clone()),
        ("line_items[0][price_data][currency]".to_string(), currency),
        (
            "line_items[0][price_data][product_data][name]".to_string(),
            product_name,
        ),
        (
            "line_items[0][price_data][unit_amount]".to_string(),
            unit_amount.to_string(),
        ),
        ("line_items[0][quantity]".to_string(), "1".to_string()),
    ];
    if let Some(client_ref_id) = &request_data.client_reference_id {
        form_body.push(("client_reference_id".to_string(), client_ref_id.clone()));
    }

    // Store fulfillment information in Stripe metadata
    form_body.push((
        "metadata[ff_type]".to_string(),
        request_data.fulfillment_type.clone(),
    ));
    let fulfillment_data_str =
        serde_json::to_string(&request_data.fulfillment_data).map_err(|e| {
            StripeError::InternalError(format!("Failed to serialize fulfillment_data: {}", e))
        })?;
    form_body.push(("metadata[ff_data_json]".to_string(), fulfillment_data_str));

    let api_url = "https://api.stripe.com/v1/checkout/sessions";

    info!("[Stripe Logic] Sending request to Stripe API: {}", api_url);
    // For debugging form body:
    // info!("[Stripe Logic] Form Body: {:?}", form_body);

    let response = HTTP_CLIENT
        .post(api_url)
        .basic_auth(stripe_secret_key, None::<&str>)
        .form(&form_body)
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    info!("[Stripe Logic] Stripe API response status: {}", status);
    if !status.is_success() {
        info!(
            "[Stripe Logic] Stripe API response body (raw): {}",
            body_text
        );
    }

    if status.is_success() {
        let stripe_response: StripeCheckoutSessionApiResponse = serde_json::from_str(&body_text)?;
        if let Some(url) = stripe_response.url {
            info!(
                "[Stripe Logic] Stripe Checkout Session created successfully. URL: {}",
                url
            );
            Ok(CreateCheckoutSessionResponse {
                url,
                session_id: stripe_response.id,
            })
        } else {
            info!(
                "[Stripe Logic] Stripe response missing checkout session URL: {}",
                body_text
            );
            Err(StripeError::InternalError(
                "Stripe response missing checkout URL".to_string(),
            ))
        }
    } else {
        let error_message = match serde_json::from_str::<serde_json::Value>(&body_text) {
            Ok(json_body) => json_body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or(&body_text)
                .to_string(),
            Err(_) => body_text,
        };
        info!(
            "[Stripe Logic] Stripe API request failed with HTTP status: {}. Message: {}",
            status, error_message
        );
        Err(StripeError::ApiError {
            status_code: status.as_u16(),
            message: error_message,
        })
    }
}

// Response FROM Stripe API when retrieving a session
// This is a more complete version of StripeCheckoutSessionObject
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StripeCheckoutSessionData {
    // Renamed for clarity
    pub id: String,
    pub object: String, // "checkout.session"
    pub amount_total: Option<i64>,
    pub currency: Option<String>,
    pub customer: Option<String>,
    pub customer_details: Option<StripeCustomerDetails>,
    pub metadata: Option<HashMap<String, String>>,
    pub payment_intent: Option<String>,
    pub payment_status: Option<String>, // e.g., "paid", "unpaid", "no_payment_required"
    pub status: Option<String>,         // e.g., "open", "complete", "expired"
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
    pub client_reference_id: Option<String>,
    pub created: Option<i64>,
    pub expires_at: Option<i64>,
    // Add other fields you might want to display on the confirmation page
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct StripeCheckoutSessionApiResponse {
    // Used for session creation response
    pub id: String,
    pub url: Option<String>,
}

// --- NEW: Function to get Checkout Session Details ---
/// Retrieves details of a Stripe Checkout Session.
pub async fn get_checkout_session_details(
    session_id: &str,
    // stripe_config: &StripeConfig, // Not strictly needed if secret key is from env
) -> Result<StripeCheckoutSessionData, StripeError> {
    info!(
        "[Stripe Logic] Retrieving Checkout Session details for ID: {}",
        session_id
    );

    let stripe_secret_key = env::var("STRIPE_SECRET_KEY").map_err(|_| StripeError::ConfigError)?;

    let api_url = format!("https://api.stripe.com/v1/checkout/sessions/{}", session_id);

    let response = HTTP_CLIENT
        .get(&api_url)
        .basic_auth(stripe_secret_key, None::<&str>)
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    if status.is_success() {
        let session_data: StripeCheckoutSessionData = serde_json::from_str(&body_text)?;
        // Optionally, verify payment_status here if needed for the confirmation page
        if session_data.payment_status.as_deref() != Some("paid")
            && session_data.status.as_deref() != Some("complete")
        {
            // This might happen if user hits success URL but payment is still processing or failed later
            info!("[Stripe Logic] Warning: Checkout session {} status is {:?}, payment_status is {:?}.",
                     session_id, session_data.status, session_data.payment_status);
            // Depending on requirements, you might return an error or different data
        }
        Ok(session_data)
    } else {
        let error_message = match serde_json::from_str::<serde_json::Value>(&body_text) {
            Ok(json_body) => json_body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or(&body_text)
                .to_string(),
            Err(_) => body_text,
        };
        error!(
            "[Stripe Logic] Failed to retrieve session {}: {} - {}",
            session_id, status, error_message
        );
        Err(StripeError::ApiError {
            status_code: status.as_u16(),
            message: error_message,
        })
    }
}

// --- NEW: Structures for Listing Checkout Sessions (Admin) ---
#[derive(Deserialize, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams, ToSchema))] // For query parameters
pub struct ListSessionsAdminQuery {
    #[cfg_attr(feature = "openapi", param(example = 10, required = false))]
    pub limit: Option<u8>, // Stripe limit is 1-100
    #[cfg_attr(
        feature = "openapi",
        param(example = "cs_test_a1...", required = false)
    )]
    pub starting_after: Option<String>,
    #[cfg_attr(
        feature = "openapi",
        param(example = "cs_test_z0...", required = false)
    )]
    pub ending_before: Option<String>,
    // Add other Stripe list parameters as needed (e.g., created, customer, payment_intent)
    // pub customer: Option<String>,
    // pub payment_intent: Option<String>,
    // pub created: Option<i64>, // Unix timestamp for filtering
}

/// Represents the list object returned by Stripe API.
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StripeListObject<T> {
    pub object: String, // "list"
    pub data: Vec<T>,
    pub has_more: bool,
    pub url: String, // URL for this list, usually /v1/checkout/sessions
                     // next_page: Option<String>, // Stripe uses starting_after/ending_before for pagination
}

// Type alias for the specific list response
pub type ListSessionsAdminResponse = StripeListObject<StripeCheckoutSessionData>;

// --- NEW: Function to list Checkout Sessions (Admin) ---
pub async fn list_checkout_sessions_admin(
    query_params: ListSessionsAdminQuery,
) -> Result<ListSessionsAdminResponse, StripeError> {
    info!(
        "[Stripe Logic] Listing Checkout Sessions for admin. Params: {:?}",
        query_params
    );

    let stripe_secret_key = env::var("STRIPE_SECRET_KEY").map_err(|_| StripeError::ConfigError)?;

    let base_url = "https://api.stripe.com/v1/checkout/sessions";

    // Build query parameters for the reqwest client
    let mut request_query_params = Vec::new();
    if let Some(limit) = query_params.limit {
        request_query_params.push(("limit", limit.to_string()));
    }
    if let Some(starting_after) = query_params.starting_after {
        request_query_params.push(("starting_after", starting_after));
    }
    if let Some(ending_before) = query_params.ending_before {
        request_query_params.push(("ending_before", ending_before));
    }
    // Add other filters here, e.g.:
    // request_query_params.push(("expand[]", "data.line_items".to_string())); // To expand line items

    let response = HTTP_CLIENT
        .get(base_url)
        .basic_auth(stripe_secret_key, None::<&str>)
        .query(&request_query_params) // reqwest can take Vec of tuples for query params
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    if status.is_success() {
        let list_response: ListSessionsAdminResponse = serde_json::from_str(&body_text)?;
        Ok(list_response)
    } else {
        let error_message = match serde_json::from_str::<serde_json::Value>(&body_text) {
            Ok(json_body) => json_body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or(&body_text)
                .to_string(),
            Err(_) => body_text,
        };
        error!(
            "[Stripe Logic] Failed to list sessions: {} - {}",
            status, error_message
        );
        Err(StripeError::ApiError {
            status_code: status.as_u16(),
            message: error_message,
        })
    }
}
