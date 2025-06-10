//! Factory for creating database clients
//!
//! This module provides a factory for creating database clients that are
//! designed to be database agnostic.

use crate::client::DbClient;
use crate::error::DbError;
use connectify_config::{AppConfig, DatabaseConfig};
use std::sync::Arc;
use tracing::debug;

/// Factory for creating database clients
///
/// This factory provides methods for creating database clients using
/// different configuration sources.
#[derive(Debug, Clone)]
pub struct DbClientFactory;

impl DbClientFactory {
    /// Create a new database client factory
    ///
    /// # Returns
    ///
    /// A new database client factory
    pub fn new() -> Self {
        Self
    }

    /// Create a new database client from an application configuration
    ///
    /// This method creates a new database client using the provided application
    /// configuration. It will attempt to connect to the database using the URL
    /// from the configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The application configuration
    ///
    /// # Returns
    ///
    /// A new database client
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    ///
    /// * The database configuration is missing
    /// * The database URL is missing
    /// * The database connection fails
    pub async fn from_app_config(&self, config: &Arc<AppConfig>) -> Result<DbClient, DbError> {
        debug!("Creating database client from application configuration");

        // Get the database configuration
        let db_config = config
            .database
            .as_ref()
            .ok_or_else(|| DbError::ConfigError("Database configuration is missing".to_string()))?;

        // Create the database client
        self.from_db_config(db_config).await
    }

    /// Create a new database client from a database configuration
    ///
    /// This method creates a new database client using the provided database
    /// configuration. It will attempt to connect to the database using the URL
    /// from the configuration.
    ///
    /// # Arguments
    ///
    /// * `db_config` - The database configuration
    ///
    /// # Returns
    ///
    /// A new database client
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    ///
    /// * The database URL is missing
    /// * The database connection fails
    pub async fn from_db_config(&self, db_config: &DatabaseConfig) -> Result<DbClient, DbError> {
        debug!("Creating database client from database configuration");

        // Create the database client
        DbClient::from_config(db_config).await
    }

    /// Create a new database client from a database URL
    ///
    /// This method creates a new database client using the provided database URL.
    /// It will attempt to connect to the database using the URL.
    ///
    /// # Arguments
    ///
    /// * `db_url` - The database URL
    ///
    /// # Returns
    ///
    /// A new database client
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    ///
    /// * The database URL is invalid
    /// * The database connection fails
    pub async fn from_url(&self, db_url: &str) -> Result<DbClient, DbError> {
        debug!("Creating database client from URL");

        // Create the database client
        DbClient::from_url(db_url).await
    }
}

impl Default for DbClientFactory {
    fn default() -> Self {
        Self::new()
    }
}
