# Connectify Architecture Documentation

This document provides an overview of the architecture of the Connectify application, including key design decisions, patterns, and the overall structure of the codebase.

## Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Design Patterns](#design-patterns)
- [Crate Structure](#crate-structure)
- [Data Flow](#data-flow)
- [Error Handling](#error-handling)
- [Configuration Management](#configuration-management)
- [Testing Strategy](#testing-strategy)
- [Performance Considerations](#performance-considerations)

## Overview

Connectify is a modular, service-oriented application built in Rust. It provides a set of services for calendar management, payment processing, notifications, and fulfillment. The application is designed to be extensible, maintainable, and testable, with a focus on clean separation of concerns and dependency injection.

## System Architecture

The Connectify application follows a modular architecture with the following key components:

1. **API Layer**: Handles HTTP requests and responses, routing, and serialization/deserialization of data.
2. **Service Layer**: Contains the business logic for each service (calendar, payment, notification, fulfillment).
3. **Integration Layer**: Integrates with external services like Google Calendar, Stripe, Payrexx, and Twilio.
4. **Configuration Layer**: Manages application configuration from files and environment variables.
5. **Common Utilities**: Provides shared functionality like error handling, logging, and HTTP client utilities.

The application is built as a set of Rust crates, each with a specific responsibility. The main crates are:

- `connectify_backend`: The main application that ties everything together.
- `connectify_common`: Common utilities and abstractions used across the application.
- `connectify_config`: Configuration management.
- `connectify_gcal`: Google Calendar integration.
- `connectify_stripe`: Stripe payment integration.
- `connectify_payrexx`: Payrexx payment integration.
- `connectify_twilio`: Twilio notification integration.
- `connectify_fulfillment`: Fulfillment service.

## Design Patterns

Connectify uses several design patterns to achieve its goals:

### Dependency Injection

Dependency injection is a core pattern used throughout the application. It allows for more modular, testable, and maintainable code by decoupling the implementation of a service from its usage. In Connectify, we use a trait-based approach to dependency injection, where services are defined as traits and implementations are provided through a factory pattern.

For more details, see the [Dependency Injection documentation](dependency_injection.md).

### Factory Pattern

The factory pattern is used to create instances of services. The `ServiceFactory` trait provides methods for getting instances of various services, and the `ConnectifyServiceFactory` struct implements this trait to provide concrete implementations of the services.

```
// ServiceFactory trait definition
trait ServiceFactory {
    // Get a calendar service instance
    fn calendar_service(&self) -> Option<Arc<dyn CalendarService>>;
    
    // Get a payment service instance
    fn payment_service(&self) -> Option<Arc<dyn PaymentService>>;
    
    // Get a notification service instance
    fn notification_service(&self) -> Option<Arc<dyn NotificationService>>;
}
```

### Builder Pattern

The builder pattern is used for constructing complex objects with many optional parameters. For example, the `AppStateBuilder` is used to construct `AppState` objects with different configurations and dependencies.

```
// Example of using the builder pattern
// app_state = AppStateBuilder::new(config)
//     .with_service_factory(service_factory)
//     .build();
```

### Strategy Pattern

The strategy pattern is used to define a family of algorithms, encapsulate each one, and make them interchangeable. For example, the `PaymentService` trait defines a strategy for processing payments, and different implementations (Stripe, Payrexx) provide different algorithms for doing so.

### Repository Pattern

The repository pattern is used to abstract the data access layer. For example, the `CalendarService` trait defines methods for accessing calendar data, and the `GoogleCalendarService` implementation provides concrete implementations of these methods for Google Calendar.

## Crate Structure

Each crate in the Connectify application follows a similar structure:

- `src/lib.rs`: The entry point for the crate, which exports the public API.
- `src/models.rs`: Data structures and models used by the crate.
- `src/logic.rs`: Core business logic.
- `src/handlers.rs`: HTTP request handlers.
- `src/routes.rs`: Route definitions.
- `src/service.rs`: Service implementations.
- `src/error.rs`: Error types and handling.

This consistent structure makes it easy to navigate the codebase and understand the responsibilities of each file.

## Data Flow

The typical data flow through the Connectify application is as follows:

1. An HTTP request is received by the API layer.
2. The request is routed to the appropriate handler based on the URL and HTTP method.
3. The handler extracts data from the request and calls the appropriate service method.
4. The service method performs the business logic, which may involve calling external services.
5. The service method returns a result, which is transformed into an HTTP response by the handler.
6. The HTTP response is returned to the client.

For example, when booking a calendar slot:

1. The client sends a POST request to `/api/gcal/book` with the slot details.
2. The request is routed to the `book_slot_handler` function.
3. The handler extracts the slot details from the request body and calls the `create_calendar_event` function.
4. The `create_calendar_event` function calls the Google Calendar API to create the event.
5. The function returns the created event, which is transformed into a JSON response by the handler.
6. The JSON response is returned to the client.

## Error Handling

Connectify uses a consistent approach to error handling across the codebase. Each crate defines its own error types that implement the `std::error::Error` trait. These error types are then converted to HTTP responses using the `IntoHttpResponse` trait from the `connectify_common` crate.

For example, the `GcalError` type in the `connectify_gcal` crate:

```
// GcalError enum definition
enum GcalError {
    ApiError(google_calendar3::Error),
    TimeParseError(String),
    CalculationError(String),
    Conflict,
    NoMatchingPriceTier(i64),
}
```

These error types are then mapped to HTTP status codes and error messages in the handlers:

```
// Example of error handling in a handler
// match create_calendar_event(...).await {
//     Ok(created_event) => {
//         Ok(Json(BookingResponse {
//             success: true,
//             event_id: created_event.id,
//             message: "Appointment booked successfully.",
//         }))
//     }
//     Err(GcalError::Conflict) => {
//         Err((
//             StatusCode::CONFLICT,
//             "Requested time slot is no longer available.",
//         ))
//     }
//     Err(e) => {
//         Err((
//             StatusCode::INTERNAL_SERVER_ERROR,
//             "Failed to book appointment.",
//         ))
//     }
// }
```

## Configuration Management

Connectify uses a layered approach to configuration management:

1. Default configuration values are defined in the code.
2. Configuration files (`config/default.yml`, `config/{RUN_ENV}.yml`) override the default values.
3. Environment variables override the values from configuration files.

The configuration is loaded at startup and made available to the application through the `AppConfig` struct. For more details, see the [Configuration documentation](../crates/connectify_config/README.md).

## Testing Strategy

Connectify uses a comprehensive testing strategy that includes:

1. **Unit Tests**: Test individual functions and methods in isolation.
2. **Integration Tests**: Test the interaction between different components.
3. **Mock Tests**: Test components with mock implementations of their dependencies.

For example, the `GoogleCalendarService` has a corresponding `MockCalendarService` that can be used in tests:

```
// Example of a mock service for testing
// struct MockCalendarService {
//     events: Mutex<HashMap<String, Vec<(String, CalendarEvent, String)>>>,
// }
//
// impl MockCalendarService {
//     fn new() -> Self {
//         Self {
//             events: Mutex::new(HashMap::new()),
//         }
//     }
// }
//
// impl CalendarService for MockCalendarService {
//     // Implementation of the CalendarService trait for testing
// }
```

## Performance Considerations

Connectify is designed with performance in mind:

1. **Asynchronous I/O**: The application uses asynchronous I/O with the Tokio runtime to handle many concurrent requests efficiently.
2. **Connection Pooling**: External service clients use connection pooling to reduce the overhead of establishing new connections.
3. **Caching**: Frequently accessed data is cached to reduce the number of external service calls.
4. **Efficient Error Handling**: Errors are handled efficiently without unnecessary allocations or deep copying of data.
5. **Minimal Dependencies**: The application uses a minimal set of dependencies to reduce compile times and binary size.