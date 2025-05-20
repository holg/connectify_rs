// --- File: crates/connectify_stripe/src/error.rs ---
use connectify_common::{
    ConnectifyError, 
    external_service_error, 
    HttpStatusCode
};
use thiserror::Error;

/// Stripe-specific error types.
#[derive(Error, Debug)]
pub enum StripeError {
    /// Error occurred during a Stripe API request
    #[error("Stripe API request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Error returned by the Stripe API
    #[error("Stripe API returned an error: {message} (Status: {status_code})")]
    ApiError { status_code: u16, message: String },

    /// Error parsing Stripe API response
    #[error("Failed to parse Stripe API response: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Missing or incomplete Stripe configuration
    #[error("Stripe configuration missing or incomplete")]
    ConfigError,

    /// Webhook signature verification failed
    #[error("Stripe webhook signature verification failed: {0}")]
    WebhookSignatureError(String),

    /// Webhook event processing error
    #[error("Stripe webhook event processing error: {0}")]
    WebhookProcessingError(String),

    /// Fulfillment service call failed
    #[error("Fulfillment service call failed: {0}")]
    FulfillmentError(String),

    /// Missing fulfillment data in webhook metadata
    #[error("Missing fulfillment data in webhook metadata")]
    MissingFulfillmentData,

    /// Session not found or not paid
    #[error("Session not found or not paid")]
    SessionNotFoundOrNotPaid,

    /// Invalid fulfillment data for pricing
    #[error("Invalid fulfillment data for pricing: {0}")]
    InvalidFulfillmentDataForPricing(String),

    /// No matching price tier found
    #[error("No matching price tier found for duration: {0} minutes")]
    NoMatchingPriceTier(i64),

    /// Internal processing error
    #[error("Internal processing error: {0}")]
    InternalError(String),
}

/// Convert StripeError to ConnectifyError
impl From<StripeError> for ConnectifyError {
    fn from(err: StripeError) -> Self {
        match err {
            StripeError::RequestError(e) => ConnectifyError::HttpError(format!("Stripe request error: {}", e)),
            StripeError::ApiError { status_code, message } => external_service_error(
                "Stripe API", 
                format!("Status: {}, Message: {}", status_code, message)
            ),
            StripeError::ParseError(e) => ConnectifyError::ParseError(format!("Stripe response parse error: {}", e)),
            StripeError::ConfigError => ConnectifyError::ConfigError("Stripe configuration missing or incomplete".to_string()),
            StripeError::WebhookSignatureError(msg) => ConnectifyError::AuthError(format!("Stripe webhook signature error: {}", msg)),
            StripeError::WebhookProcessingError(msg) => external_service_error("Stripe webhook", msg),
            StripeError::FulfillmentError(msg) => external_service_error("Fulfillment service", msg),
            StripeError::MissingFulfillmentData => ConnectifyError::ValidationError("Missing fulfillment data in webhook metadata".to_string()),
            StripeError::SessionNotFoundOrNotPaid => ConnectifyError::NotFoundError("Stripe session not found or not paid".to_string()),
            StripeError::InvalidFulfillmentDataForPricing(msg) => ConnectifyError::ValidationError(format!("Invalid fulfillment data for pricing: {}", msg)),
            StripeError::NoMatchingPriceTier(duration) => ConnectifyError::ValidationError(format!("No matching price tier found for duration: {} minutes", duration)),
            StripeError::InternalError(msg) => ConnectifyError::InternalError(format!("Stripe internal error: {}", msg)),
        }
    }
}

/// Implement HttpStatusCode for StripeError to provide a consistent way to convert
/// StripeError to HTTP status codes.
impl HttpStatusCode for StripeError {
    fn status_code(&self) -> u16 {
        match self {
            StripeError::RequestError(_) => 500,
            StripeError::ApiError { status_code, .. } => *status_code,
            StripeError::ParseError(_) => 400,
            StripeError::ConfigError => 500,
            StripeError::WebhookSignatureError(_) => 401,
            StripeError::WebhookProcessingError(_) => 500,
            StripeError::FulfillmentError(_) => 502,
            StripeError::MissingFulfillmentData => 400,
            StripeError::SessionNotFoundOrNotPaid => 404,
            StripeError::InvalidFulfillmentDataForPricing(_) => 400,
            StripeError::NoMatchingPriceTier(_) => 400,
            StripeError::InternalError(_) => 500,
        }
    }
}
