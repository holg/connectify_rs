# Dependency Injection in Connectify

This document describes the dependency injection pattern used in the Connectify application.

## Overview

Dependency injection is a design pattern that allows for more modular, testable, and maintainable code by decoupling the implementation of a service from its usage. In Connectify, we use a trait-based approach to dependency injection, where services are defined as traits and implementations are provided through a factory pattern.

## Service Abstractions

The core of our dependency injection approach is the service abstraction layer, which provides interfaces for external services used by the application. These abstractions are defined in the `connectify_common` crate in the `services` module.

### Available Service Abstractions

- `CalendarService`: For calendar operations like checking availability, booking slots, and managing events.
- `PaymentService`: For payment operations like creating charges and handling refunds.
- `NotificationService`: For sending notifications via email or SMS.

### Service Factory

Services are accessed through the `ServiceFactory` trait, which provides methods for getting instances of the various services:

```rust
pub trait ServiceFactory: Send + Sync {
    /// Get a calendar service instance.
    fn calendar_service(&self) -> Option<Arc<dyn CalendarService<Error = Box<dyn std::error::Error + Send + Sync>>>>;

    /// Get a payment service instance.
    fn payment_service(&self) -> Option<Arc<dyn PaymentService<Error = Box<dyn std::error::Error + Send + Sync>>>>;

    /// Get a notification service instance.
    fn notification_service(&self) -> Option<Arc<dyn NotificationService<Error = Box<dyn std::error::Error + Send + Sync>>>>;
}
```

The application state holds a reference to a service factory, which is used to get the services needed by the application:

```rust
pub struct AppState {
    /// The application configuration.
    pub config: Arc<AppConfig>,
    
    /// Service factory for accessing external services.
    pub service_factory: Arc<dyn ServiceFactory>,
    
    // ...
}
```

## Implementing Service Abstractions

Each service abstraction is defined as a trait with associated methods. For example, the `CalendarService` trait:

```rust
#[async_trait]
pub trait CalendarService: Send + Sync {
    /// Error type returned by calendar service operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get busy time intervals within a specified time range.
    async fn get_busy_times(
        &self,
        calendar_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, Self::Error>;

    // ... other methods ...
}
```

To implement a service abstraction, create a struct that implements the trait:

```rust
pub struct GoogleCalendarService {
    calendar_hub: Arc<HubType>,
}

#[async_trait]
impl CalendarService for GoogleCalendarService {
    type Error = GcalServiceError;

    async fn get_busy_times(
        &self,
        calendar_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, Self::Error> {
        // Implementation...
    }

    // ... other methods ...
}
```

## Service Factory Implementation

The service factory is implemented in the `connectify_backend` crate in the `service_factory.rs` file:

```rust
pub struct ConnectifyServiceFactory {
    config: Arc<AppConfig>,
    #[cfg(feature = "gcal")]
    calendar_service: Option<Arc<dyn CalendarService<Error = Box<dyn std::error::Error + Send + Sync>>>>,
    #[cfg(feature = "stripe")]
    payment_service: Option<Arc<dyn PaymentService<Error = Box<dyn std::error::Error + Send + Sync>>>>,
    #[cfg(feature = "twilio")]
    notification_service: Option<Arc<dyn NotificationService<Error = Box<dyn std::error::Error + Send + Sync>>>>,
}

impl ConnectifyServiceFactory {
    /// Create a new service factory.
    pub async fn new(config: Arc<AppConfig>) -> Self {
        let mut factory = Self {
            config: config.clone(),
            #[cfg(feature = "gcal")]
            calendar_service: None,
            #[cfg(feature = "stripe")]
            payment_service: None,
            #[cfg(feature = "twilio")]
            notification_service: None,
        };

        // Initialize services based on configuration
        #[cfg(feature = "gcal")]
        {
            if config.use_gcal && config.gcal.is_some() {
                // Initialize Google Calendar service
                // ...
            }
        }

        // Initialize other services
        // ...

        factory
    }
}

impl ServiceFactory for ConnectifyServiceFactory {
    fn calendar_service(&self) -> Option<Arc<dyn CalendarService<Error = Box<dyn std::error::Error + Send + Sync>>>> {
        #[cfg(feature = "gcal")]
        {
            self.calendar_service.clone()
        }
        #[cfg(not(feature = "gcal"))]
        {
            None
        }
    }

    // ... other methods ...
}
```

## Using Services

To use a service, get it from the service factory:

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

## Testing with Mock Services

For testing, you can use the mock implementations provided in the `mock` modules:

```rust
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Mock calendar service for testing.
    pub struct MockCalendarService {
        events: Mutex<HashMap<String, Vec<(String, CalendarEvent, String)>>>,
    }

    impl MockCalendarService {
        /// Create a new mock calendar service.
        pub fn new() -> Self {
            Self {
                events: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CalendarService for MockCalendarService {
        type Error = GcalServiceError;

        async fn get_busy_times(
            &self,
            calendar_id: &str,
            start_time: DateTime<Utc>,
            end_time: DateTime<Utc>,
        ) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, Self::Error> {
            // Mock implementation...
        }

        // ... other methods ...
    }
}
```

There's also a mock service factory for testing:

```rust
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::Arc;
    
    #[cfg(feature = "gcal")]
    use connectify_gcal::service::mock::MockCalendarService;

    /// Mock service factory for testing.
    pub struct MockServiceFactory {
        #[cfg(feature = "gcal")]
        calendar_service: Option<Arc<dyn CalendarService<Error = Box<dyn std::error::Error + Send + Sync>>>>,
        #[cfg(feature = "stripe")]
        payment_service: Option<Arc<dyn PaymentService<Error = Box<dyn std::error::Error + Send + Sync>>>>,
        #[cfg(feature = "twilio")]
        notification_service: Option<Arc<dyn NotificationService<Error = Box<dyn std::error::Error + Send + Sync>>>>,
    }

    impl MockServiceFactory {
        /// Create a new mock service factory.
        pub fn new() -> Self {
            Self {
                #[cfg(feature = "gcal")]
                calendar_service: Some(Arc::new(MockCalendarService::new())),
                #[cfg(feature = "stripe")]
                payment_service: None,
                #[cfg(feature = "twilio")]
                notification_service: None,
            }
        }
    }

    impl ServiceFactory for MockServiceFactory {
        // ... implementation ...
    }
}
```

## Benefits

This dependency injection approach provides several benefits:

1. **Testability**: Services can be easily mocked for testing.
2. **Modularity**: Services are defined by their interfaces, not their implementations.
3. **Flexibility**: Implementations can be swapped out without changing the code that uses them.
4. **Configurability**: Services can be conditionally initialized based on configuration.
5. **Separation of concerns**: The code that uses a service doesn't need to know how it's implemented.

## Conclusion

The dependency injection pattern used in Connectify provides a flexible, testable, and maintainable way to work with external services. By defining services as traits and providing implementations through a factory pattern, we can easily swap out implementations for testing or to change the behavior of the application without modifying the core logic.