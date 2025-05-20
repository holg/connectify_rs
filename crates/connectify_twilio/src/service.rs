use connectify_common::services::{NotificationResult, NotificationService};
use connectify_config::AppConfig;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;

/// Twilio-specific error types.
#[derive(Error, Debug)]
pub enum TwilioError {
    /// Error occurred during a Twilio API request
    #[error("Twilio API request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Error returned by the Twilio API
    #[error("Twilio API returned an error: {message} (Status: {status_code})")]
    ApiError { status_code: u16, message: String },

    /// Missing or incomplete Twilio configuration
    #[error("Twilio configuration missing or incomplete")]
    ConfigError,

    /// Internal processing error
    #[error("Internal processing error: {0}")]
    InternalError(String),
}

/// Twilio notification service implementation
pub struct TwilioNotificationService {
    /// Configuration for the Twilio service.
    ///
    /// This field stores the application configuration that contains Twilio API credentials
    /// and settings. While it's not used in the current placeholder implementation (which
    /// returns "Not implemented" errors), it's kept for several reasons:
    ///
    /// 1. It will be needed when implementing the actual Twilio API integration
    /// 2. It follows the pattern used by other service implementations
    /// 3. It maintains consistency with the dependency injection architecture
    /// 4. It makes the service's dependencies explicit
    ///
    /// When the actual implementation is completed, this configuration will be used to
    /// authenticate with the Twilio API and configure the service behavior.
    #[allow(dead_code)]
    config: Arc<AppConfig>,
}

impl TwilioNotificationService {
    /// Create a new Twilio notification service
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }
}

impl NotificationService for TwilioNotificationService {
    type Error = TwilioError;

    fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        _is_html: bool,
    ) -> Pin<Box<dyn Future<Output = Result<NotificationResult, Self::Error>> + Send + '_>> {
        // Clone the values to avoid lifetime issues
        let to = to.to_string();
        let subject = subject.to_string();
        let _body = body.to_string();

        Box::pin(async move {
            // This would need to be implemented with Twilio API
            // For now, return a placeholder
            Err(TwilioError::ApiError {
                status_code: 501,
                message: format!(
                    "Not implemented: send_email to {} with subject {}",
                    to, subject
                ),
            })
        })
    }

    fn send_sms(
        &self,
        to: &str,
        body: &str,
    ) -> Pin<Box<dyn Future<Output = Result<NotificationResult, Self::Error>> + Send + '_>> {
        // Clone the values to avoid lifetime issues
        let to = to.to_string();
        let body = body.to_string();

        Box::pin(async move {
            // This would need to be implemented with Twilio API
            // For now, return a placeholder
            Err(TwilioError::ApiError {
                status_code: 501,
                message: format!("Not implemented: send_sms to {} with body {}", to, body),
            })
        })
    }
}
