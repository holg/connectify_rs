// --- File: crates/services/connectify_backend/src/service_factory.rs ---
//! Service factory implementation.
//!
//! This module provides an implementation of the ServiceFactory trait for the backend service.
use connectify_config::AppConfig;
use std::sync::Arc;
#[allow(unused_imports)] // even so it is used only by certain features, this shall change
use {
    chrono::{DateTime, Utc},
    chrono_tz::Tz,
    connectify_common::is_feature_enabled,
    connectify_common::services::{
        BookedEvent, BoxedError, CalendarEvent, CalendarEventResult, CalendarService,
        NotificationResult, NotificationService, PaymentIntentResult, PaymentService, RefundResult,
        ServiceFactory,
    },
    tracing::{error, info, warn},
};

#[cfg(feature = "gcal")]
use connectify_gcal::{
    auth::create_calendar_hub,
    service::{GcalServiceError, GoogleCalendarService},
};

#[cfg(feature = "stripe")]
use connectify_stripe::service::StripePaymentService;

#[cfg(feature = "twilio")]
use connectify_twilio::service::TwilioNotificationService;

#[cfg(feature = "firebase")]
use connectify_firebase::service::FirebaseServiceFactory;

/// Service factory implementation.
///
/// This struct implements the `ServiceFactory` trait, providing access to all external services
/// used by the application. It's a key component of the dependency injection pattern used in
/// this application.
///
/// The factory initializes services based on the application configuration and feature flags,
/// making them available through the trait methods. This centralized approach to service
/// management improves testability, modularity, and maintainability.
pub struct ConnectifyServiceFactory {
    /// Configuration for the service factory.
    ///
    /// This field stores the application configuration that was loaded at startup.
    /// It's used during the initialization of services in the `new` method, and is kept
    /// for several important reasons:
    ///
    /// 1. It may be needed for future service implementations that require configuration access
    /// 2. It allows for potential runtime reconfiguration of services
    /// 3. It maintains the complete context in which the factory was created
    /// 4. It supports the dependency injection pattern by keeping all dependencies explicit
    ///
    /// While it may not be directly accessed after initialization in the current implementation,
    /// keeping it ensures the factory has all the information it needs for future extensions.
    #[allow(dead_code)]
    config: Arc<AppConfig>,
    #[cfg(feature = "gcal")]
    calendar_service: Option<Arc<dyn CalendarService<Error = BoxedError>>>,
    #[cfg(feature = "stripe")]
    payment_service: Option<Arc<dyn PaymentService<Error = BoxedError>>>,
    #[cfg(feature = "twilio")]
    notification_service: Option<Arc<dyn NotificationService<Error = BoxedError>>>,
    #[cfg(feature = "firebase")]
    firebase_service_factory: Option<Arc<FirebaseServiceFactory>>,
}

impl ConnectifyServiceFactory {
    /// Create a new service factory.
    pub async fn new(config: Arc<AppConfig>) -> Self {
        #[allow(unused_mut)]
        let mut factory = Self {
            config: config.clone(),
            #[cfg(feature = "gcal")]
            calendar_service: None,
            #[cfg(feature = "stripe")]
            payment_service: None,
            #[cfg(feature = "twilio")]
            notification_service: None,
            #[cfg(feature = "firebase")]
            firebase_service_factory: None,
        };

        // Initialize services based on configuration
        #[cfg(feature = "gcal")]
        {
            if is_feature_enabled(&config, config.use_gcal, config.gcal.as_ref()) {
                info!("ℹ️ Initializing Google Calendar service...");
                match create_calendar_hub(config.gcal.as_ref().unwrap()).await {
                    Ok(hub) => {
                        let service = GoogleCalendarService::new(Arc::new(hub));

                        // Create a wrapper that converts GcalServiceError to a concrete error type
                        #[derive(Debug)]
                        struct BoxedCalendarError(Box<dyn std::error::Error + Send + Sync>);

                        impl std::fmt::Display for BoxedCalendarError {
                            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                write!(f, "{}", self.0)
                            }
                        }

                        impl std::error::Error for BoxedCalendarError {}

                        impl From<GcalServiceError> for BoxedCalendarError {
                            fn from(err: GcalServiceError) -> Self {
                                BoxedCalendarError(Box::new(err))
                            }
                        }

                        struct BoxedCalendarService {
                            inner: GoogleCalendarService,
                        }

                        impl CalendarService for BoxedCalendarService {
                            type Error = BoxedError;

                            fn get_busy_times(
                                &self,
                                calendar_id: &str,
                                start_time: DateTime<Tz>,
                                end_time: DateTime<Tz>,
                            ) -> std::pin::Pin<
                                Box<
                                    dyn std::future::Future<
                                            Output = Result<
                                                Vec<(DateTime<Tz>, DateTime<Tz>)>,
                                                Self::Error,
                                            >,
                                        > + Send
                                        + '_,
                                >,
                            > {
                                let calendar_id = calendar_id.to_string();
                                let inner = &self.inner;

                                Box::pin(async move {
                                    inner
                                        .get_busy_times(&calendar_id, start_time, end_time)
                                        .await
                                        .map_err(|e| BoxedError(Box::new(e)))
                                })
                            }

                            fn create_event(
                                &self,
                                calendar_id: &str,
                                event: CalendarEvent,
                            ) -> std::pin::Pin<
                                Box<
                                    dyn std::future::Future<
                                            Output = Result<CalendarEventResult, Self::Error>,
                                        > + Send
                                        + '_,
                                >,
                            > {
                                let calendar_id = calendar_id.to_string();
                                let event = event.clone();
                                let inner = &self.inner;

                                Box::pin(async move {
                                    inner
                                        .create_event(&calendar_id, event)
                                        .await
                                        .map_err(|e| BoxedError(Box::new(e)))
                                })
                            }

                            fn delete_event(
                                &self,
                                calendar_id: &str,
                                event_id: &str,
                                notify_attendees: bool,
                            ) -> std::pin::Pin<
                                Box<
                                    dyn std::future::Future<Output = Result<(), Self::Error>>
                                        + Send
                                        + '_,
                                >,
                            > {
                                let calendar_id = calendar_id.to_string();
                                let event_id = event_id.to_string();
                                let inner = &self.inner;

                                Box::pin(async move {
                                    inner
                                        .delete_event(&calendar_id, &event_id, notify_attendees)
                                        .await
                                        .map_err(|e| BoxedError(Box::new(e)))
                                })
                            }

                            fn mark_event_cancelled(
                                &self,
                                calendar_id: &str,
                                event_id: &str,
                                notify_attendees: bool,
                            ) -> std::pin::Pin<
                                Box<
                                    dyn std::future::Future<
                                            Output = Result<CalendarEventResult, Self::Error>,
                                        > + Send
                                        + '_,
                                >,
                            > {
                                let calendar_id = calendar_id.to_string();
                                let event_id = event_id.to_string();
                                let inner = &self.inner;

                                Box::pin(async move {
                                    inner
                                        .mark_event_cancelled(
                                            &calendar_id,
                                            &event_id,
                                            notify_attendees,
                                        )
                                        .await
                                        .map_err(|e| BoxedError(Box::new(e)))
                                })
                            }

                            fn get_booked_events(
                                &self,
                                calendar_id: &str,
                                start_time: DateTime<Tz>,
                                end_time: DateTime<Tz>,
                                include_cancelled: bool,
                            ) -> std::pin::Pin<
                                Box<
                                    dyn std::future::Future<
                                            Output = Result<Vec<BookedEvent>, Self::Error>,
                                        > + Send
                                        + '_,
                                >,
                            > {
                                let calendar_id = calendar_id.to_string();
                                let inner = &self.inner;

                                Box::pin(async move {
                                    inner
                                        .get_booked_events(
                                            &calendar_id,
                                            start_time,
                                            end_time,
                                            include_cancelled,
                                        )
                                        .await
                                        .map_err(|e| BoxedError(Box::new(e)))
                                })
                            }
                        }

                        let boxed_service = BoxedCalendarService { inner: service };
                        factory.calendar_service = Some(Arc::new(boxed_service));
                        info!("✅ Google Calendar service initialized.");
                    }
                    Err(e) => {
                        error!("🚨 Failed to initialize Google Calendar service: {}. GCal routes disabled.", e);
                    }
                }
            } else {
                info!("ℹ️ GCal feature compiled, but disabled via runtime config or missing gcal config section.");
            }
        }

        // Initialize Stripe service if enabled
        #[cfg(feature = "stripe")]
        {
            if is_feature_enabled(&config, config.use_stripe, config.stripe.as_ref()) {
                info!("ℹ️ Initializing Stripe payment service...");

                // Create a wrapper that converts StripeError to BoxedError
                struct BoxedPaymentService {
                    inner: StripePaymentService,
                }

                impl PaymentService for BoxedPaymentService {
                    type Error = BoxedError;

                    fn create_payment_intent(
                        &self,
                        amount: i64,
                        currency: &str,
                        description: Option<&str>,
                        metadata: Option<serde_json::Value>,
                    ) -> std::pin::Pin<
                        Box<
                            dyn std::future::Future<
                                    Output = Result<PaymentIntentResult, Self::Error>,
                                > + Send
                                + '_,
                        >,
                    > {
                        let currency = currency.to_string();
                        let description = description.map(|s| s.to_string());
                        let metadata = metadata.clone();
                        let inner = &self.inner;

                        Box::pin(async move {
                            inner
                                .create_payment_intent(
                                    amount,
                                    &currency,
                                    description.as_deref(),
                                    metadata.clone(),
                                )
                                .await
                                .map_err(|e| BoxedError(Box::new(e)))
                        })
                    }

                    fn confirm_payment_intent(
                        &self,
                        payment_intent_id: &str,
                    ) -> std::pin::Pin<
                        Box<
                            dyn std::future::Future<
                                    Output = Result<PaymentIntentResult, Self::Error>,
                                > + Send
                                + '_,
                        >,
                    > {
                        let payment_intent_id = payment_intent_id.to_string();
                        let inner = &self.inner;

                        Box::pin(async move {
                            inner
                                .confirm_payment_intent(&payment_intent_id)
                                .await
                                .map_err(|e| BoxedError(Box::new(e)))
                        })
                    }

                    fn cancel_payment_intent(
                        &self,
                        payment_intent_id: &str,
                    ) -> std::pin::Pin<
                        Box<
                            dyn std::future::Future<
                                    Output = Result<PaymentIntentResult, Self::Error>,
                                > + Send
                                + '_,
                        >,
                    > {
                        let payment_intent_id = payment_intent_id.to_string();
                        let inner = &self.inner;

                        Box::pin(async move {
                            inner
                                .cancel_payment_intent(&payment_intent_id)
                                .await
                                .map_err(|e| BoxedError(Box::new(e)))
                        })
                    }

                    fn create_refund(
                        &self,
                        payment_intent_id: &str,
                        amount: Option<i64>,
                        reason: Option<&str>,
                    ) -> std::pin::Pin<
                        Box<
                            dyn std::future::Future<Output = Result<RefundResult, Self::Error>>
                                + Send
                                + '_,
                        >,
                    > {
                        let payment_intent_id = payment_intent_id.to_string();
                        let reason = reason.map(|s| s.to_string());
                        let inner = &self.inner;

                        Box::pin(async move {
                            inner
                                .create_refund(&payment_intent_id, amount, reason.as_deref())
                                .await
                                .map_err(|e| BoxedError(Box::new(e)))
                        })
                    }
                }

                let service = StripePaymentService::new(config.clone());
                let boxed_service = BoxedPaymentService { inner: service };
                factory.payment_service = Some(Arc::new(boxed_service));
                info!("✅ Stripe payment service initialized.");
            }
        }

        // Initialize Twilio service if enabled
        #[cfg(feature = "twilio")]
        {
            if is_feature_enabled(&config, config.use_twilio, config.twilio.as_ref()) {
                info!("ℹ️ Initializing Twilio notification service...");

                // Create a wrapper that converts TwilioError to BoxedError
                struct BoxedNotificationService {
                    inner: TwilioNotificationService,
                }

                impl NotificationService for BoxedNotificationService {
                    type Error = BoxedError;

                    fn send_email(
                        &self,
                        to: &str,
                        subject: &str,
                        body: &str,
                        is_html: bool,
                    ) -> std::pin::Pin<
                        Box<
                            dyn std::future::Future<
                                    Output = Result<NotificationResult, Self::Error>,
                                > + Send
                                + '_,
                        >,
                    > {
                        let to = to.to_string();
                        let subject = subject.to_string();
                        let body = body.to_string();
                        let inner = &self.inner;

                        Box::pin(async move {
                            inner
                                .send_email(&to, &subject, &body, is_html)
                                .await
                                .map_err(|e| BoxedError(Box::new(e)))
                        })
                    }

                    fn send_sms(
                        &self,
                        to: &str,
                        body: &str,
                    ) -> std::pin::Pin<
                        Box<
                            dyn std::future::Future<
                                    Output = Result<NotificationResult, Self::Error>,
                                > + Send
                                + '_,
                        >,
                    > {
                        let to = to.to_string();
                        let body = body.to_string();
                        let inner = &self.inner;

                        Box::pin(async move {
                            inner
                                .send_sms(&to, &body)
                                .await
                                .map_err(|e| BoxedError(Box::new(e)))
                        })
                    }
                }

                let service = TwilioNotificationService::new(config.clone());
                let boxed_service = BoxedNotificationService { inner: service };
                factory.notification_service = Some(Arc::new(boxed_service));
                info!("✅ Twilio notification service initialized.");
            }
        }

        // Initialize Firebase service if enabled
        #[cfg(feature = "firebase")]
        {
            if is_feature_enabled(&config, config.use_firebase, config.firebase.as_ref()) {
                info!("ℹ️ Initializing Firebase service factory...");
                let firebase_factory = FirebaseServiceFactory::new(config.clone());
                factory.firebase_service_factory = Some(Arc::new(firebase_factory));
                info!("✅ Firebase service factory initialized.");
            }
        }

        factory
    }
}

impl ServiceFactory for ConnectifyServiceFactory {
    fn calendar_service(&self) -> Option<Arc<dyn CalendarService<Error = BoxedError>>> {
        #[cfg(feature = "gcal")]
        {
            if let Some(service) = self.calendar_service.clone() {
                return Some(service);
            }
        }

        #[cfg(feature = "firebase")]
        {
            if let Some(firebase_factory) = &self.firebase_service_factory {
                if let Some(service) = firebase_factory.calendar_service() {
                    return Some(service);
                }
            }
        }

        None
    }

    fn payment_service(&self) -> Option<Arc<dyn PaymentService<Error = BoxedError>>> {
        #[cfg(feature = "stripe")]
        {
            if let Some(service) = self.payment_service.clone() {
                return Some(service);
            }
        }

        #[cfg(feature = "firebase")]
        {
            if let Some(firebase_factory) = &self.firebase_service_factory {
                if let Some(service) = firebase_factory.payment_service() {
                    return Some(service);
                }
            }
        }

        None
    }

    fn notification_service(&self) -> Option<Arc<dyn NotificationService<Error = BoxedError>>> {
        #[cfg(feature = "twilio")]
        {
            if let Some(service) = self.notification_service.clone() {
                return Some(service);
            }
        }

        #[cfg(feature = "firebase")]
        {
            if let Some(firebase_factory) = &self.firebase_service_factory {
                if let Some(service) = firebase_factory.notification_service() {
                    return Some(service);
                }
            }
        }

        None
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::Arc;

    // Mock calendar service is not available outside of tests
    // #[cfg(all(feature = "gcal", test))]
    // use connectify_gcal::service::mock::MockCalendarService;

    /// Mock service factory for testing.
    pub struct MockServiceFactory {
        #[cfg(feature = "gcal")]
        calendar_service: Option<Arc<dyn CalendarService<Error = BoxedError>>>,
        #[cfg(feature = "stripe")]
        payment_service: Option<Arc<dyn PaymentService<Error = BoxedError>>>,
        #[cfg(feature = "twilio")]
        notification_service: Option<Arc<dyn NotificationService<Error = BoxedError>>>,
        #[cfg(feature = "firebase")]
        firebase_service_factory: Option<Arc<FirebaseServiceFactory>>,
    }

    impl Default for MockServiceFactory {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockServiceFactory {
        /// Create a new mock service factory.
        pub fn new() -> Self {
            Self {
                #[cfg(feature = "gcal")]
                calendar_service: None, // Mock implementation would be added in actual tests
                #[cfg(feature = "stripe")]
                payment_service: None,
                #[cfg(feature = "twilio")]
                notification_service: None,
                #[cfg(feature = "firebase")]
                firebase_service_factory: None,
            }
        }
    }

    impl ServiceFactory for MockServiceFactory {
        fn calendar_service(&self) -> Option<Arc<dyn CalendarService<Error = BoxedError>>> {
            #[cfg(feature = "gcal")]
            {
                if let Some(service) = self.calendar_service.clone() {
                    return Some(service);
                }
            }

            #[cfg(feature = "firebase")]
            {
                if let Some(firebase_factory) = &self.firebase_service_factory {
                    if let Some(service) = firebase_factory.calendar_service() {
                        return Some(service);
                    }
                }
            }

            None
        }

        fn payment_service(&self) -> Option<Arc<dyn PaymentService<Error = BoxedError>>> {
            #[cfg(feature = "stripe")]
            {
                if let Some(service) = self.payment_service.clone() {
                    return Some(service);
                }
            }

            #[cfg(feature = "firebase")]
            {
                if let Some(firebase_factory) = &self.firebase_service_factory {
                    if let Some(service) = firebase_factory.payment_service() {
                        return Some(service);
                    }
                }
            }

            None
        }

        fn notification_service(&self) -> Option<Arc<dyn NotificationService<Error = BoxedError>>> {
            #[cfg(feature = "twilio")]
            {
                if let Some(service) = self.notification_service.clone() {
                    return Some(service);
                }
            }

            #[cfg(feature = "firebase")]
            {
                if let Some(firebase_factory) = &self.firebase_service_factory {
                    if let Some(service) = firebase_factory.notification_service() {
                        return Some(service);
                    }
                }
            }

            None
        }
    }
}
