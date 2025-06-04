#[cfg(test)]
mod tests {
    use crate::twilio_sms::{send_sms, SmsRequest};
    use axum::extract::State;
    use axum::Json;
    use connectify_config::load_config;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_send_sms() {
        // Skip this test if not in production environment
        if std::env::var("RUN_ENV").unwrap_or_default() != "production" {
            println!("Skipping test_send_sms in non-production environment");
            return;
        }
        // Create a mock config with Twilio settings
        let config = Arc::new(load_config().unwrap());

        // Create a test SMS request
        let request = Json(SmsRequest {
            to: "whatsapp:+1 123123123123".to_string(),
            message: "Test message von OSS Web Connectify RS".to_string(),
        });

        // Call the send_sms function with both required arguments
        let result = send_sms(State(config), request).await;

        // Assert on the result - this will depend on your test environment// For a unit test without actual API calls, you might expect an error
        assert!(
            result.is_ok(),
            "Expected error when Twilio is not configured properly"
        );
    }
}
