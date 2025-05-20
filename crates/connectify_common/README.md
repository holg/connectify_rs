## CI Status

| Build & Test & Fmt & Clippy |
|:---------------------------:|
| [![Rust Tests](https://github.com/holg/connectify_rs/actions/workflows/rust-tests.yml/badge.svg?branch=main)](https://github.com/holg/connectify_rs/actions/workflows/rust-tests.yml) |
[Test, Clippy, Rustfmt, Code coverage, Benchmark, clippy]

# Connectify Common

This crate provides common functionality and abstractions for the Connectify application, including:

- Service abstractions for external services (Calendar, Payment, Notification)
- Dependency injection pattern using traits and factories
- A base `ConnectifyError` enum with various error variants
- Utilities for converting between error types
- HTTP response utilities for Axum

## Service Abstractions

One of the key features of this crate is the service abstraction layer, which provides interfaces for external services used by the application. These abstractions allow for dependency injection and easier testing by decoupling the application logic from specific implementations of external services.

### Available Service Abstractions

- `CalendarService`: For calendar operations like checking availability, booking slots, and managing events.
- `PaymentService`: For payment operations like creating charges and handling refunds.
- `NotificationService`: For sending notifications via email or SMS.

### Using Service Abstractions

Services are accessed through the `ServiceFactory` trait, which provides methods for getting instances of the various services. The application state holds a reference to a service factory, which is used to get the services needed by the application.

```rust
// Get a calendar service from the service factory
if let Some(calendar_service) = app_state.service_factory.calendar_service() {
    // Use the calendar service
    let busy_times = calendar_service.get_busy_times(
        calendar_id,
        start_time,
        end_time,
    ).await?;

    // ...
}
```

### Implementing Service Abstractions

Each service abstraction is defined as a trait with associated methods. To implement a service abstraction, create a struct that implements the trait:

```rust
use connectify_common::services::{CalendarService, CalendarEvent, CalendarEventResult};

pub struct MyCalendarService {
    // ...
}

#[async_trait]
impl CalendarService for MyCalendarService {
    type Error = MyError;

    async fn get_busy_times(
        &self,
        calendar_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, Self::Error> {
        // Implementation...
    }

    // Implement other methods...
}
```

### Testing with Mock Services

For testing, you can use the mock implementations provided in the `mock` modules:

```rust
use connectify_common::services::CalendarService;
use connectify_gcal::service::mock::MockCalendarService;

#[tokio::test]
async fn test_my_function() {
    // Create a mock calendar service
    let calendar_service = MockCalendarService::new();

    // Use the mock service in your test
    // ...
}
```

## Dependency Injection

The service abstractions and factory pattern provide a form of dependency injection for the application. This allows for easier testing and more flexible configuration.

To use dependency injection:

1. Define your service interfaces as traits.
2. Implement these traits for your concrete service implementations.
3. Create a service factory that provides instances of these services.
4. Use the service factory to get the services you need.

This approach makes it easy to swap out implementations for testing or to change the behavior of the application without modifying the core logic.

## Error Handling

## Error Types

### ConnectifyError

The base error type for all Connectify errors. It provides a common set of error variants that can be used across all crates:

```rust
pub enum ConnectifyError {
    HttpError(String),
    ParseError(String),
    ConfigError(String),
    AuthError(String),
    ValidationError(String),
    DatabaseError(String),
    ExternalServiceError { service_name: String, message: String },
    ConflictError(String),
    NotFoundError(String),
    TimeoutError(String),
    RateLimitError(String),
    InternalError(String),
    OtherError(String),
}
```

### Crate-Specific Error Types

Each crate should define its own error type that extends `ConnectifyError`. For example:

```rust
#[derive(Error, Debug)]
pub enum StripeError {
    #[error("Stripe API request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Stripe API returned an error: {message} (Status: {status_code})")]
    ApiError { status_code: u16, message: String },

    // ... other error variants specific to Stripe
}

// Convert StripeError to ConnectifyError
impl From<StripeError> for ConnectifyError {
    fn from(err: StripeError) -> Self {
        match err {
            StripeError::RequestError(e) => ConnectifyError::HttpError(format!("Stripe request error: {}", e)),
            // ... other conversions
        }
    }
}
```

## HTTP Status Codes

The `HttpStatusCode` trait provides a consistent way to convert errors to HTTP status codes:

```rust
pub trait HttpStatusCode {
    fn status_code(&self) -> u16;
}
```

Implement this trait for your crate-specific error type:

```rust
impl HttpStatusCode for StripeError {
    fn status_code(&self) -> u16 {
        match self {
            StripeError::RequestError(_) => 500,
            StripeError::ApiError { status_code, .. } => *status_code,
            // ... other status codes
        }
    }
}
```

## HTTP Response Utilities

The `IntoHttpResponse` trait and related utilities make it easy to convert errors to Axum HTTP responses:

```rust
// Convert an error to an Axum HTTP response
pub trait IntoHttpResponse {
    fn into_http_response(self) -> Response;
}

// Handle a Result<T, ConnectifyError> in an Axum handler
pub fn handle_result<T>(result: Result<T, ConnectifyError>) -> Result<T, Response>
where
    T: IntoResponse;

// Handle a Result<T, ConnectifyError> that returns JSON in an Axum handler
pub fn handle_json_result<T>(result: Result<T, ConnectifyError>) -> Result<Json<T>, Response>
where
    T: serde::Serialize;

// Map a Result<T, E> to a Result<T, Response> using a custom error mapper
pub fn map_error<T, E, F>(result: Result<T, E>, f: F) -> Result<T, Response>
where
    T: IntoResponse,
    F: FnOnce(E) -> ConnectifyError;

// Map a Result<T, E> to a Result<Json<T>, Response> using a custom error mapper
pub fn map_json_error<T, E, F>(result: Result<T, E>, f: F) -> Result<Json<T>, Response>
where
    T: serde::Serialize,
    F: FnOnce(E) -> ConnectifyError;
```

## Usage in Axum Handlers

Here's an example of how to use these utilities in an Axum handler:

```rust
pub async fn create_checkout_session_handler(
    State(state): State<Arc<StripeState>>,
    Json(payload): Json<CreateCheckoutSessionRequest>,
) -> Result<Json<CreateCheckoutSessionResponse>, Response> {
    if !state.config.use_stripe {
        return Err(ConnectifyError::ConfigError("Stripe service is disabled".to_string()).into_response());
    }

    if let Some(stripe_config) = state.config.stripe.as_ref() {
        // Use map_json_error to convert StripeError to ConnectifyError and then to a Response
        map_json_error(
            create_checkout_session(stripe_config, payload).await,
            |err| err.into() // Convert StripeError to ConnectifyError using the From implementation
        )
    } else {
        Err(config_error("Stripe configuration not loaded").into_response())
    }
}
```

## Adding Context to Errors

The `Context` trait provides a way to add context to errors:

```rust
pub trait Context<T, E> {
    fn context<C>(self, context: C) -> Result<T, ConnectifyError>
    where
        C: fmt::Display + Send + Sync + 'static;

    fn with_context<C, F>(self, f: F) -> Result<T, ConnectifyError>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}
```

Example usage:

```rust
let result = some_function()
    .context("Failed to do something")
    .map_err(|e| e.into_response())?;
```
