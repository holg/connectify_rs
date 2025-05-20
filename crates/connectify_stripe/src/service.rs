use std::sync::Arc;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use connectify_common::services::{PaymentService, PaymentIntentResult, RefundResult};
use connectify_config::AppConfig;
use crate::logic::{create_checkout_session, CreateCheckoutSessionRequest};
use crate::error::StripeError;

/// Stripe payment service implementation
pub struct StripePaymentService {
    config: Arc<AppConfig>,
}

impl StripePaymentService {
    /// Create a new Stripe payment service
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }
}

impl PaymentService for StripePaymentService {
    type Error = StripeError;

    fn create_payment_intent(
        &self,
        amount: i64,
        currency: &str,
        description: Option<&str>,
        metadata: Option<Value>,
    ) -> Pin<Box<dyn Future<Output = Result<PaymentIntentResult, Self::Error>> + Send + '_>> {
        // Clone the values to avoid lifetime issues
        let currency = currency.to_string();
        let description = description.map(|s| s.to_string());
        let metadata = metadata.clone();
        let _config = self.config.clone();

        Box::pin(async move {
            // Create a checkout session request
            let request = CreateCheckoutSessionRequest {
                product_name_override: description.map(|s| s.to_string()),
                amount_override: Some(amount),
                currency_override: Some(currency.to_string()),
                fulfillment_type: "payment".to_string(),
                fulfillment_data: metadata.unwrap_or(Value::Null),
                client_reference_id: None,
            };

            // Use the existing create_checkout_session function
            let stripe_config = self.config.stripe.as_ref()
                .ok_or_else(|| StripeError::ConfigError)?;

            let checkout_result = create_checkout_session(stripe_config, request).await?;

            // Convert to PaymentIntentResult
            Ok(PaymentIntentResult {
                id: checkout_result.session_id,
                status: "created".to_string(),
                amount,
                currency: currency.to_string(),
                client_secret: None,
            })
        })
    }

    fn confirm_payment_intent(
        &self,
        payment_intent_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<PaymentIntentResult, Self::Error>> + Send + '_>> {
        let payment_intent_id = payment_intent_id.to_string();
        Box::pin(async move {
            // This would need to be implemented with Stripe API
            // For now, return a placeholder
            Err(StripeError::ApiError { 
                status_code: 501, 
                message: format!("Not implemented: confirm_payment_intent for {}", payment_intent_id) 
            })
        })
    }

    fn cancel_payment_intent(
        &self,
        payment_intent_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<PaymentIntentResult, Self::Error>> + Send + '_>> {
        let payment_intent_id = payment_intent_id.to_string();
        Box::pin(async move {
            // This would need to be implemented with Stripe API
            // For now, return a placeholder
            Err(StripeError::ApiError { 
                status_code: 501, 
                message: format!("Not implemented: cancel_payment_intent for {}", payment_intent_id) 
            })
        })
    }

    fn create_refund(
        &self,
        payment_intent_id: &str,
        amount: Option<i64>,
        reason: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<RefundResult, Self::Error>> + Send + '_>> {
        let payment_intent_id = payment_intent_id.to_string();
        let reason = reason.map(|s| s.to_string());
        Box::pin(async move {
            // This would need to be implemented with Stripe API
            // For now, return a placeholder
            Err(StripeError::ApiError { 
                status_code: 501, 
                message: format!("Not implemented: create_refund for {} with amount {:?} and reason {:?}", 
                    payment_intent_id, amount, reason) 
            })
        })
    }
}
