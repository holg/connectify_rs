//! Firebase service factory implementation.
//!
//! This module provides an implementation of the ServiceFactory trait for Firebase services.

use crate::client::FirebaseClient;
use connectify_common::services::{
    BoxedError, CalendarService, NotificationService, PaymentService, ServiceFactory,
};
use connectify_config::AppConfig;
use std::sync::Arc;
use tracing::{debug, error, info};

#[cfg(feature = "database")]
use crate::repository::DeviceRegistrationRepository;
#[cfg(feature = "database")]
use crate::repository_factory::DeviceRegistrationRepositoryFactory;
#[cfg(feature = "database")]
use connectify_db::{DbClient, DbClientFactory, RepositoryFactory};

/// Firebase service factory.
///
/// This struct implements the `ServiceFactory` trait for Firebase services.
/// It provides a Firebase client with database integration for storing and
/// retrieving device registration tokens.
pub struct FirebaseServiceFactory {
    /// Configuration for the service factory.
    config: Arc<AppConfig>,

    /// Database client for storing device registration tokens.
    /// This is None if database integration is not enabled.
    #[cfg(feature = "database")]
    db_client: Option<DbClient>,

    /// Repository for device registrations.
    /// This is None if database integration is not enabled.
    #[cfg(feature = "database")]
    repository: Option<DeviceRegistrationRepository>,
}

impl FirebaseServiceFactory {
    /// Create a new Firebase service factory.
    #[cfg(feature = "database")]
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            db_client: None,
            repository: None,
        }
    }

    /// Create a new Firebase service factory.
    #[cfg(not(feature = "database"))]
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Initialize the database client and repository.
    ///
    /// This method initializes the database client and repository if database
    /// integration is enabled in the configuration.
    ///
    /// # Returns
    ///
    /// `Ok(())` if initialization was successful, or an error if it failed.
    #[cfg(feature = "database")]
    pub async fn init_db(&mut self) -> Result<(), BoxedError> {
        // Check if database integration is enabled
        if !self.config.use_firebase || self.config.firebase.is_none() {
            debug!("Firebase is not enabled in configuration");
            return Ok(());
        }

        // Check if database is configured
        if !self.config.use_firebase || self.config.database.is_none() {
            debug!("Database is not configured, skipping Firebase database integration");
            return Ok(());
        }

        // Create the database client factory
        let db_client_factory = DbClientFactory::new();

        // Create the database client
        let db_client = db_client_factory
            .from_app_config(&self.config)
            .await
            .map_err(|e| {
                error!("Failed to create database client: {}", e);
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;

        // Create the repository factory
        let repository_factory = DeviceRegistrationRepositoryFactory::new();

        // Create the repository
        let repository = repository_factory.create_repository(db_client.clone());

        // Initialize the schema
        repository.init_schema().await.map_err(|e| {
            error!("Failed to initialize device registration schema: {}", e);
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;

        // Store the client and repository
        self.db_client = Some(db_client);
        self.repository = Some(repository);

        info!("Firebase database integration initialized successfully");
        Ok(())
    }

    /// Initialize the database client and repository.
    ///
    /// This is a stub implementation when the database feature is not enabled.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    #[cfg(not(feature = "database"))]
    pub async fn init_db(&mut self) -> Result<(), BoxedError> {
        debug!("Database feature is not enabled, skipping initialization");
        Ok(())
    }

    /// Get a Firebase client with database integration.
    ///
    /// This method returns a Firebase client with the repository set if database
    /// integration is enabled.
    ///
    /// # Returns
    ///
    /// A Firebase client with database integration if enabled, or without database
    /// integration if not enabled.
    #[cfg(feature = "database")]
    pub fn client(&self) -> FirebaseClient {
        let firebase_config = self.config.firebase.clone().unwrap_or_default();
        let mut client = FirebaseClient::new(firebase_config);

        // Set the repository if available
        if let Some(repository) = self.repository.clone() {
            client = client.with_repository(repository);
        }

        client
    }

    /// Get a Firebase client without database integration.
    ///
    /// This method returns a Firebase client without database integration
    /// when the database feature is not enabled.
    ///
    /// # Returns
    ///
    /// A Firebase client without database integration.
    #[cfg(not(feature = "database"))]
    pub fn client(&self) -> FirebaseClient {
        let firebase_config = self.config.firebase.clone().unwrap_or_default();
        FirebaseClient::new(firebase_config)
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
