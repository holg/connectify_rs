# Code Documentation Guidelines

This document provides guidelines for adding inline documentation to functions, methods, and types in the Connectify codebase. It includes examples of well-documented code and best practices for writing clear and helpful documentation.

## Table of Contents

- [Introduction](#introduction)
- [Documentation Style](#documentation-style)
- [Function Documentation](#function-documentation)
- [Type Documentation](#type-documentation)
- [Module Documentation](#module-documentation)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Introduction

Good code documentation is essential for maintainability, onboarding new developers, and ensuring that the codebase remains understandable as it grows. In Rust, documentation is written using doc comments, which start with `///` for line comments or `/** */` for block comments.

Documentation comments are written in Markdown and can include code examples, links, and formatting. They are processed by the Rust documentation tool `rustdoc` to generate HTML documentation.

## Documentation Style

In the Connectify codebase, we follow these documentation style guidelines:

1. Use `///` for line comments rather than `/** */` for block comments.
2. Write documentation in complete sentences with proper punctuation.
3. Start function documentation with a verb in the present tense (e.g., "Returns", "Creates", "Validates").
4. Include examples for complex functions or types.
5. Document all parameters and return values.
6. Document error conditions and panics.
7. Use Markdown formatting for clarity.

## Function Documentation

Function documentation should include:

1. A brief description of what the function does.
2. A more detailed explanation if necessary.
3. Documentation for each parameter.
4. Documentation for the return value.
5. Documentation for any errors that might be returned.
6. Examples of how to use the function.

Example:

```rust
/// Calculates available time slots based on busy periods and constraints.
///
/// This function takes a range of dates, a list of busy periods, and various constraints
/// (such as working hours, appointment duration, etc.) and returns a list of available
/// time slots for booking.
///
/// # Arguments
///
/// * `start_time` - The start of the date range to check for availability.
/// * `end_time` - The end of the date range to check for availability.
/// * `busy_periods` - A list of busy periods (start and end times) that are already booked.
/// * `duration` - The duration of the appointment to book.
/// * `work_start_time` - The start of the working day.
/// * `work_end_time` - The end of the working day.
/// * `working_days` - A list of days of the week that are working days.
/// * `buffer_time` - The buffer time between appointments.
/// * `step` - The time step to use when checking for available slots.
///
/// # Returns
///
/// A vector of `DateTime<Utc>` representing the start times of available slots.
///
/// # Examples
///
/// ```
/// use chrono::{DateTime, Duration, NaiveTime, Utc, Weekday};
///
/// let start_time = Utc::now();
/// let end_time = start_time + Duration::days(7);
/// let busy_periods = vec![];
/// let duration = Duration::minutes(60);
/// let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
/// let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
/// let working_days = vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri];
/// let buffer = Duration::minutes(0);
/// let step = Duration::minutes(15);
///
/// let available_slots = calculate_available_slots(
///     start_time,
///     end_time,
///     &busy_periods,
///     duration,
///     work_start,
///     work_end,
///     &working_days,
///     buffer,
///     step,
/// );
/// ```
pub fn calculate_available_slots(
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    busy_periods: &[(DateTime<Utc>, DateTime<Utc>)],
    duration: Duration,
    work_start_time: NaiveTime,
    work_end_time: NaiveTime,
    working_days: &[Weekday],
    buffer_time: Duration,
    step: Duration,
) -> Vec<DateTime<Utc>> {
    // Implementation...
}
```

## Type Documentation

Type documentation should include:

1. A brief description of what the type represents.
2. A more detailed explanation if necessary.
3. Documentation for each field or variant.
4. Examples of how to use the type.

Example:

```rust
/// Represents a request to book a calendar slot.
///
/// This struct contains all the information needed to book a calendar slot,
/// including the start and end times, summary, and description.
///
/// # Fields
///
/// * `start_time` - The start time of the slot in ISO 8601 format.
/// * `end_time` - The end time of the slot in ISO 8601 format.
/// * `summary` - The summary or title of the event.
/// * `description` - An optional description of the event.
///
/// # Examples
///
/// ```
/// use connectify_gcal::models::BookSlotRequest;
///
/// let request = BookSlotRequest {
///     start_time: "2025-05-15T10:00:00Z".to_string(),
///     end_time: "2025-05-15T11:00:00Z".to_string(),
///     summary: "Consultation with John Doe".to_string(),
///     description: Some("Initial consultation to discuss project requirements".to_string()),
/// };
/// ```
#[derive(Debug, Deserialize)]
pub struct BookSlotRequest {
    /// The start time of the slot in ISO 8601 format.
    pub start_time: String,
    /// The end time of the slot in ISO 8601 format.
    pub end_time: String,
    /// The summary or title of the event.
    pub summary: String,
    /// An optional description of the event.
    pub description: Option<String>,
}
```

## Module Documentation

Module documentation should include:

1. A brief description of what the module does.
2. A more detailed explanation if necessary.
3. An overview of the key types and functions in the module.
4. Examples of how to use the module.

Example:

```rust
//! Calendar service implementation.
//!
//! This module provides an implementation of the `CalendarService` trait for Google Calendar.
//! It includes functions for checking availability, booking slots, and managing calendar events.
//!
//! # Examples
//!
//! ```
//! use connectify_gcal::service::GoogleCalendarService;
//! use connectify_common::services::CalendarService;
//!
//! async fn example(calendar_service: &GoogleCalendarService) {
//!     let calendar_id = "primary";
//!     let start_time = chrono::Utc::now();
//!     let end_time = start_time + chrono::Duration::days(7);
//!
//!     let busy_times = calendar_service.get_busy_times(calendar_id, start_time, end_time).await.unwrap();
//!     info!("Busy times: {:?}", busy_times);
//! }
//! ```
```

## Examples

Here are some examples of well-documented functions from the Connectify codebase:

### Example 1: Error Handling Function

```rust
/// Maps a result to an HTTP response.
///
/// This function takes a result and maps it to an HTTP response. If the result is `Ok`,
/// the value is serialized to JSON and returned with a 200 OK status code. If the result
/// is `Err`, the error is mapped to an appropriate HTTP status code and error message.
///
/// # Arguments
///
/// * `result` - The result to map.
///
/// # Returns
///
/// An HTTP response with the appropriate status code and body.
///
/// # Examples
///
/// ```
/// use connectify_common::http::handle_json_result;
/// use axum::http::StatusCode;
///
/// async fn handler() -> impl axum::response::IntoResponse {
///     let result: Result<String, MyError> = Ok("Hello, world!".to_string());
///     handle_json_result(result)
/// }
/// ```
pub fn handle_json_result<T, E>(result: Result<T, E>) -> impl axum::response::IntoResponse
where
    T: Serialize,
    E: IntoHttpResponse,
{
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(err) => err.into_http_response(),
    }
}
```

### Example 2: Configuration Loading Function

```rust
/// Loads the application configuration.
///
/// This function loads the application configuration from files and environment variables.
/// It first loads the default configuration from `config/default.yml`, then overrides it
/// with environment-specific configuration from `config/{RUN_ENV}.yml`, and finally
/// overrides it with environment variables.
///
/// # Returns
///
/// The loaded configuration, or an error if the configuration could not be loaded.
///
/// # Errors
///
/// This function returns an error if:
///
/// * The configuration files could not be read.
/// * The configuration files contain invalid YAML.
/// * The configuration values could not be deserialized into the `AppConfig` struct.
/// * The configuration values fail validation.
///
/// # Examples
///
/// ```
/// use connectify_config::load_config;
///
/// let config = load_config().expect("Failed to load configuration");
/// info!("Server host: {}", config.server.host);
/// info!("Server port: {}", config.server.port);
/// ```
pub fn load_config() -> Result<AppConfig, ConfigurationError> {
    // Implementation...
}
```

## Best Practices

Here are some best practices for writing good documentation:

1. **Be concise**: Write clear, concise documentation that gets to the point quickly.
2. **Be complete**: Document all parameters, return values, and error conditions.
3. **Use examples**: Provide examples for complex functions or types.
4. **Keep it up to date**: Update documentation when you change code.
5. **Use Markdown**: Use Markdown formatting to make documentation more readable.
6. **Link to related items**: Use links to refer to related functions, types, or modules.
7. **Document edge cases**: Document edge cases and how they are handled.
8. **Document performance considerations**: Document any performance considerations or optimizations.
9. **Document thread safety**: Document thread safety considerations for concurrent code.
10. **Document API stability**: Document whether an API is stable, experimental, or deprecated.

Remember that good documentation is not just about describing what the code does, but also why it does it that way. Include context and rationale where appropriate to help future developers understand the design decisions behind the code.