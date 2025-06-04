//! Firebase service factory implementation.
//!
//! This module provides an implementation of the ServiceFactory trait for Firebase services.

use connectify_common::services::{
    BoxedError, CalendarService, NotificationService, PaymentService, ServiceFactory,
};
use connectify_config::AppConfig;
use std::sync::Arc;

/// Firebase service factory.
///
/// This struct implements the `ServiceFactory` trait for Firebase services.
/// Currently, it doesn't provide any specific service implementations as Firebase
/// is primarily used for push notifications which don't directly map to the existing
/// service traits.
pub struct FirebaseServiceFactory {
    /// Configuration for the service factory.
    #[allow(dead_code)]
    config: Arc<AppConfig>,
}

impl FirebaseServiceFactory {
    /// Create a new Firebase service factory.
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }
}

impl ServiceFactory for FirebaseServiceFactory {
    fn calendar_service(&self) -> Option<Arc<dyn CalendarService<Error = BoxedError>>> {
        // Firebase doesn't provide calendar services
        None
    }

    fn payment_service(&self) -> Option<Arc<dyn PaymentService<Error = BoxedError>>> {
        // Firebase doesn't provide payment services
        None
    }

    fn notification_service(&self) -> Option<Arc<dyn NotificationService<Error = BoxedError>>> {
        // Firebase could potentially be used for notifications, but it would require
        // implementing the NotificationService trait for FirebaseClient, which is
        // beyond the scope of this change.
        None
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;

    /// Mock Firebase service factory for testing.
    pub struct MockFirebaseServiceFactory;

    impl Default for MockFirebaseServiceFactory {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockFirebaseServiceFactory {
        /// Create a new mock Firebase service factory.
        pub fn new() -> Self {
            Self
        }
    }

    impl ServiceFactory for MockFirebaseServiceFactory {
        fn calendar_service(&self) -> Option<Arc<dyn CalendarService<Error = BoxedError>>> {
            None
        }

        fn payment_service(&self) -> Option<Arc<dyn PaymentService<Error = BoxedError>>> {
            None
        }

        fn notification_service(&self) -> Option<Arc<dyn NotificationService<Error = BoxedError>>> {
            None
        }
    }
}
