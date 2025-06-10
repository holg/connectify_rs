//! Error types for the database client

use thiserror::Error;

/// Errors that can occur when working with the database client
#[derive(Debug, Error)]
pub enum DbError {
    /// Error from SQLx
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    /// Error with the database configuration
    #[error("Database configuration error: {0}")]
    ConfigError(String),

    /// Error with database URL parsing
    #[error("Database URL error: {0}")]
    UrlError(String),

    /// Error with database connection
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    /// Error with database pool creation
    #[error("Database pool error: {0}")]
    PoolError(String),

    /// Error with database query
    #[error("Database query error: {0}")]
    QueryError(String),

    /// Error with database transaction
    #[error("Database transaction error: {0}")]
    TransactionError(String),

    /// Other errors
    #[error("Other database error: {0}")]
    Other(String),
}
