//! Database client for Connectify
//!
//! This module provides a database client that is designed to be database agnostic,
//! using SQLx as the underlying database library.

use crate::error::DbError;
use connectify_config::{AppConfig, DatabaseConfig};
use sqlx::pool::PoolOptions;
use sqlx::{Pool, Transaction};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

/// Type alias for a database transaction
pub type DbTransaction<'a> = Transaction<'a, sqlx::Any>;

/// Database client for Connectify
///
/// This client provides a database-agnostic interface to the database,
/// using SQLx as the underlying database library.
#[derive(Debug, Clone)]
pub struct DbClient {
    /// The database connection pool
    pool: Pool<sqlx::Any>,
}

impl DbClient {
    /// Create a new database client
    ///
    /// This function creates a new database client using the provided configuration.
    /// It will attempt to connect to the database using the URL from the configuration.
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
    /// This function will return an error if:
    ///
    /// * The database configuration is missing
    /// * The database URL is missing
    /// * The database connection fails
    pub async fn new(config: &Arc<AppConfig>) -> Result<Self, DbError> {
        // Get the database configuration
        let db_config = config
            .database
            .as_ref()
            .ok_or_else(|| DbError::ConfigError("Database configuration is missing".to_string()))?;

        // Create the database client
        Self::from_config(db_config).await
    }

    /// Create a new database client from a database configuration
    ///
    /// This function creates a new database client using the provided database configuration.
    /// It will attempt to connect to the database using the URL from the configuration.
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
    /// This function will return an error if:
    ///
    /// * The database URL is missing
    /// * The database connection fails
    pub async fn from_config(db_config: &DatabaseConfig) -> Result<Self, DbError> {
        // Get the database URL
        let db_url = &db_config.url;
        if db_url.is_empty() {
            return Err(DbError::ConfigError("Database URL is empty".to_string()));
        }

        // Create the connection pool
        let pool = Self::create_pool(db_url).await?;

        // Create the client
        Ok(Self { pool })
    }

    /// Create a new database client from a database URL
    ///
    /// This function creates a new database client using the provided database URL.
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
    /// This function will return an error if:
    ///
    /// * The database URL is invalid
    /// * The database connection fails
    pub async fn from_url(db_url: &str) -> Result<Self, DbError> {
        if db_url.is_empty() {
            return Err(DbError::UrlError("Database URL is empty".to_string()));
        }

        // Create the connection pool
        let pool = Self::create_pool(db_url).await?;

        // Create the client
        Ok(Self { pool })
    }

    /// Create a connection pool
    ///
    /// This function creates a connection pool for the database.
    ///
    /// # Arguments
    ///
    /// * `db_url` - The database URL
    ///
    /// # Returns
    ///
    /// A connection pool
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The database URL is invalid
    /// * The database connection fails
    async fn create_pool(db_url: &str) -> Result<Pool<sqlx::Any>, DbError> {
        debug!("Creating database pool with URL: {}", db_url);

        // Explicitly register the database drivers
        #[cfg(feature = "sqlite")]
        {
            // This import ensures the SQLite driver is linked and registered
            // use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
            // Register the SQLite driver with the "any" driver
            sqlx::any::install_default_drivers();

            // We don't need to handle SQLite specially here, as we'll set create_if_missing
            // in the AnyConnectOptions below
        }

        #[cfg(feature = "postgres")]
        {
            // This import ensures the PostgreSQL driver is linked and registered
            #[allow(unused_imports)]
            use sqlx::postgres::PgPoolOptions as _;
        }

        #[cfg(feature = "mysql")]
        {
            // This import ensures the MySQL driver is linked and registered
            #[allow(unused_imports)]
            use sqlx::mysql::MySqlPoolOptions as _;
        }

        // Configure the connection pool
        let pool_options = PoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(600));

        // For SQLite, we need to create the database file if it doesn't exist
        // Unfortunately, we can't directly set create_if_missing on AnyConnectOptions
        // So we'll create a directory for the database file if it doesn't exist
        if db_url.starts_with("sqlite:") {
            debug!("Handling SQLite URL: {}", db_url);
            // Extract the database file path from the URL
            // Handle both "sqlite:example.db" and "sqlite://example.db" formats
            let db_path = if db_url.starts_with("sqlite://") {
                db_url.strip_prefix("sqlite://").unwrap_or(db_url)
            } else {
                db_url.strip_prefix("sqlite:").unwrap_or(db_url)
            };

            // If it's not an in-memory database, ensure the directory exists
            debug!("Extracted database path: {}", db_path);
            if !db_path.contains(":memory:") && !db_path.is_empty() {
                debug!(
                    "Checking if database file exists: {}",
                    std::path::Path::new(db_path).exists()
                );
                // Get the directory part of the path
                if let Some(dir) = std::path::Path::new(db_path).parent() {
                    // Create the directory if it doesn't exist
                    if !dir.exists() {
                        debug!("Creating directory for SQLite database: {:?}", dir);
                        std::fs::create_dir_all(dir).map_err(|e| {
                            error!("Failed to create directory for SQLite database: {}", e);
                            DbError::PoolError(format!("Failed to create directory: {}", e))
                        })?;
                    }
                }

                // Create an empty file if it doesn't exist
                if !std::path::Path::new(db_path).exists() {
                    debug!("Creating empty SQLite database file: {}", db_path);
                    std::fs::File::create(db_path).map_err(|e| {
                        error!("Failed to create SQLite database file: {}", e);
                        DbError::PoolError(format!("Failed to create database file: {}", e))
                    })?;
                    debug!(
                        "Created SQLite database file successfully. File exists: {}",
                        std::path::Path::new(db_path).exists()
                    );
                }
            }
        }

        // Create the connection pool
        let pool = pool_options
            .connect_with(sqlx::any::AnyConnectOptions::from_str(db_url)?)
            .await
            .map_err(|e| {
                error!("Failed to create database pool: {}", e);
                DbError::PoolError(e.to_string())
            })?;

        info!("Database pool created successfully");
        debug!(
            "Database connection test: {}",
            sqlx::query("SELECT 1").execute(&pool).await.is_ok()
        );
        Ok(pool)
    }

    /// Get the database connection pool
    ///
    /// This function returns a reference to the database connection pool.
    ///
    /// # Returns
    ///
    /// A reference to the database connection pool
    pub fn pool(&self) -> &Pool<sqlx::Any> {
        &self.pool
    }

    /// Begin a transaction
    ///
    /// This function begins a new transaction on the database.
    ///
    /// # Returns
    ///
    /// A new transaction
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The transaction cannot be started
    pub async fn begin(&self) -> Result<DbTransaction<'_>, DbError> {
        self.pool
            .begin()
            .await
            .map_err(|e| DbError::TransactionError(e.to_string()))
    }

    /// Execute a query that returns no rows
    ///
    /// This function executes a query that does not return any rows.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to execute
    ///
    /// # Returns
    ///
    /// The number of rows affected
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The query fails to execute
    pub async fn execute(&self, query: &str) -> Result<u64, DbError> {
        sqlx::query(query)
            .execute(&self.pool)
            .await
            .map(|r| r.rows_affected())
            .map_err(|e| DbError::QueryError(e.to_string()))
    }

    /// Check if the database is healthy
    ///
    /// This function checks if the database is healthy by executing a simple query.
    ///
    /// # Returns
    ///
    /// `true` if the database is healthy, `false` otherwise
    pub async fn is_healthy(&self) -> bool {
        sqlx::query("SELECT 1").execute(&self.pool).await.is_ok()
    }
}

impl std::fmt::Display for DbClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DbClient")
    }
}
