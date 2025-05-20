// --- File: crates/connectify_common/src/error.rs ---
use std::fmt;
use thiserror::Error;

/// The base error type for all Connectify errors.
///
/// This enum provides a common set of error variants that can be used across all crates.
/// Each crate can extend this by implementing From<SpecificError> for ConnectifyError.
#[derive(Error, Debug)]
pub enum ConnectifyError {
    /// Error occurred during an HTTP request
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    /// Error occurred while parsing data
    #[error("Failed to parse data: {0}")]
    ParseError(String),

    /// Error occurred due to missing or invalid configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Error occurred during authentication or authorization
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Error occurred during validation
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Error occurred during database operation
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Error occurred during external service call
    #[error("External service error: {service_name} - {message}")]
    ExternalServiceError {
        service_name: String,
        message: String,
    },

    /// Error occurred due to a conflict (e.g., resource already exists)
    #[error("Conflict: {0}")]
    ConflictError(String),

    /// Error occurred due to a resource not being found
    #[error("Not found: {0}")]
    NotFoundError(String),

    /// Error occurred due to a timeout
    #[error("Timeout: {0}")]
    TimeoutError(String),

    /// Error occurred due to rate limiting
    #[error("Rate limited: {0}")]
    RateLimitError(String),

    /// Error occurred due to an internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Error that doesn't fit into any other category
    #[error("Other error: {0}")]
    OtherError(String),
}

/// A trait for converting errors to HTTP status codes.
///
/// This trait can be implemented by error types to provide a consistent way
/// to convert errors to HTTP status codes.
pub trait HttpStatusCode {
    /// Returns the HTTP status code for this error.
    fn status_code(&self) -> u16;
}

impl HttpStatusCode for ConnectifyError {
    fn status_code(&self) -> u16 {
        match self {
            ConnectifyError::HttpError(_) => 500,
            ConnectifyError::ParseError(_) => 400,
            ConnectifyError::ConfigError(_) => 500,
            ConnectifyError::AuthError(_) => 401,
            ConnectifyError::ValidationError(_) => 400,
            ConnectifyError::DatabaseError(_) => 500,
            ConnectifyError::ExternalServiceError { .. } => 502,
            ConnectifyError::ConflictError(_) => 409,
            ConnectifyError::NotFoundError(_) => 404,
            ConnectifyError::TimeoutError(_) => 504,
            ConnectifyError::RateLimitError(_) => 429,
            ConnectifyError::InternalError(_) => 500,
            ConnectifyError::OtherError(_) => 500,
        }
    }
}

/// A trait for adding context to errors.
///
/// This trait can be implemented by error types to provide a consistent way
/// to add context to errors.
pub trait Context<T, E> {
    /// Adds context to an error.
    fn context<C>(self, context: C) -> Result<T, ConnectifyError>
    where
        C: fmt::Display + Send + Sync + 'static;

    /// Adds context to an error with a lazy context provider.
    fn with_context<C, F>(self, f: F) -> Result<T, ConnectifyError>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E: std::error::Error + Send + Sync + 'static> Context<T, E> for Result<T, E> {
    fn context<C>(self, context: C) -> Result<T, ConnectifyError>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|error| ConnectifyError::InternalError(format!("{}: {}", context, error)))
    }

    fn with_context<C, F>(self, f: F) -> Result<T, ConnectifyError>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|error| ConnectifyError::InternalError(format!("{}: {}", f(), error)))
    }
}

// Common error conversions
impl From<reqwest::Error> for ConnectifyError {
    fn from(err: reqwest::Error) -> Self {
        ConnectifyError::HttpError(err.to_string())
    }
}

impl From<serde_json::Error> for ConnectifyError {
    fn from(err: serde_json::Error) -> Self {
        ConnectifyError::ParseError(err.to_string())
    }
}

impl From<std::io::Error> for ConnectifyError {
    fn from(err: std::io::Error) -> Self {
        ConnectifyError::InternalError(err.to_string())
    }
}

// Utility functions for error handling
pub fn config_error<T: fmt::Display>(message: T) -> ConnectifyError {
    ConnectifyError::ConfigError(message.to_string())
}

pub fn validation_error<T: fmt::Display>(message: T) -> ConnectifyError {
    ConnectifyError::ValidationError(message.to_string())
}

pub fn not_found<T: fmt::Display>(message: T) -> ConnectifyError {
    ConnectifyError::NotFoundError(message.to_string())
}

pub fn conflict<T: fmt::Display>(message: T) -> ConnectifyError {
    ConnectifyError::ConflictError(message.to_string())
}

pub fn external_service_error<T: fmt::Display>(service_name: &str, message: T) -> ConnectifyError {
    ConnectifyError::ExternalServiceError {
        service_name: service_name.to_string(),
        message: message.to_string(),
    }
}

pub fn internal_error<T: fmt::Display>(message: T) -> ConnectifyError {
    ConnectifyError::InternalError(message.to_string())
}
