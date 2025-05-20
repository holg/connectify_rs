//! Logging utilities for the Connectify application.
//!
//! This module provides a standardized approach to logging across all crates
//! in the Connectify application. It includes functions for initializing the
//! tracing subscriber and for logging at different levels.

use tracing::{debug, error, info, trace, warn, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize the tracing subscriber.
///
/// This function should be called at the start of the application to set up
/// logging. It configures the tracing subscriber with the specified log level
/// and formats log messages with timestamps, log levels, targets, and file/line
/// information.
///
/// # Arguments
///
/// * `level` - The minimum log level to display. Defaults to INFO if not specified.
///
/// # Examples
///
/// ```
/// use connectify_common::logging;
///
/// // Initialize with default log level (INFO)
/// logging::init();
///
/// // Initialize with a specific log level
/// logging::init_with_level(tracing::Level::DEBUG);
/// ```
pub fn init() {
    init_with_level(Level::INFO);
}

/// Initialize the tracing subscriber with a specific log level.
///
/// This function allows specifying a custom log level when initializing the
/// tracing subscriber.
///
/// # Arguments
///
/// * `level` - The minimum log level to display.
pub fn init_with_level(level: Level) {
    // Create a filter based on the specified level
    let filter = EnvFilter::from_default_env()
        .add_directive(format!("connectify={}", level).parse().unwrap());

    // Initialize the subscriber with the filter
    // Use try_init to handle the case where a global default subscriber has already been set
    let result = tracing_subscriber::registry()
        .with(fmt::layer()
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_thread_names(true))
        .with(filter)
        .try_init();

    // Only log if initialization was successful or if it failed because a subscriber was already set
    if result.is_ok() {
        info!("Logging initialized at level: {}", level);
    }
}

/// Log a message at the TRACE level.
///
/// This is a convenience function that wraps the tracing::trace! macro.
///
/// # Arguments
///
/// * `message` - The message to log.
pub fn trace_log(message: &str) {
    trace!("{}", message);
}

/// Log a message at the DEBUG level.
///
/// This is a convenience function that wraps the tracing::debug! macro.
///
/// # Arguments
///
/// * `message` - The message to log.
pub fn debug_log(message: &str) {
    debug!("{}", message);
}

/// Log a message at the INFO level.
///
/// This is a convenience function that wraps the tracing::info! macro.
///
/// # Arguments
///
/// * `message` - The message to log.
pub fn info_log(message: &str) {
    info!("{}", message);
}

/// Log a message at the WARN level.
///
/// This is a convenience function that wraps the tracing::warn! macro.
///
/// # Arguments
///
/// * `message` - The message to log.
pub fn warn_log(message: &str) {
    warn!("{}", message);
}

/// Log a message at the ERROR level.
///
/// This is a convenience function that wraps the tracing::error! macro.
///
/// # Arguments
///
/// * `message` - The message to log.
pub fn error_log(message: &str) {
    error!("{}", message);
}

/// Log an error with context at the ERROR level.
///
/// This function logs an error along with additional context information.
///
/// # Arguments
///
/// * `error` - The error to log.
/// * `context` - Additional context information about the error.
pub fn log_error<E: std::fmt::Display>(error: E, context: &str) {
    error!("{}: {}", context, error);
}

/// Log a result, with different messages for success and error cases.
///
/// This function logs a success message at the INFO level if the result is Ok,
/// or an error message at the ERROR level if the result is Err.
///
/// # Arguments
///
/// * `result` - The result to log.
/// * `success_message` - The message to log if the result is Ok.
/// * `error_context` - Additional context information to include if the result is Err.
///
/// # Returns
///
/// The original result, allowing this function to be used in a chain.
pub fn log_result<T, E: std::fmt::Display>(
    result: Result<T, E>,
    success_message: &str,
    error_context: &str,
) -> Result<T, E> {
    match &result {
        Ok(_) => info!("{}", success_message),
        Err(e) => error!("{}: {}", error_context, e),
    }
    result
}
