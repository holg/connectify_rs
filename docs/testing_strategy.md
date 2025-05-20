# Connectify Testing Strategy

This document outlines the testing strategy for the Connectify project, including the types of tests, testing tools, and best practices for writing and running tests.

## Table of Contents

- [Overview](#overview)
- [Testing Pyramid](#testing-pyramid)
- [Types of Tests](#types-of-tests)
  - [Unit Tests](#unit-tests)
  - [Integration Tests](#integration-tests)
  - [End-to-End Tests](#end-to-end-tests)
  - [Property-Based Tests](#property-based-tests)
  - [Contract Tests](#contract-tests)
  - [Performance Tests](#performance-tests)
- [Testing Tools](#testing-tools)
- [Test Organization](#test-organization)
- [Writing Testable Code](#writing-testable-code)
- [Test Coverage](#test-coverage)
- [Continuous Integration](#continuous-integration)
- [Best Practices](#best-practices)

## Overview

Testing is a critical part of the Connectify development process. Our testing strategy aims to ensure that the application is reliable, maintainable, and meets the requirements. We use a combination of different types of tests to achieve comprehensive test coverage.

## Testing Pyramid

We follow the testing pyramid approach, which suggests having:

1. A large number of **unit tests** that test individual components in isolation.
2. A moderate number of **integration tests** that test the interaction between components.
3. A small number of **end-to-end tests** that test the entire application.

This approach provides a good balance between test coverage, execution speed, and maintenance cost.

## Types of Tests

### Unit Tests

Unit tests verify that individual components (functions, methods, or classes) work as expected in isolation. They are fast, focused, and help identify issues early in the development process.

In Connectify, we use Rust's built-in testing framework for unit tests. Unit tests are typically placed in the same file as the code they test, inside a `#[cfg(test)]` module.

Example of a unit test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_available_slots() {
        let start_time = Utc::now();
        let end_time = start_time + Duration::days(1);
        let busy_periods = vec![];
        let duration = Duration::minutes(60);
        let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        let working_days = vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri];
        let buffer = Duration::minutes(0);
        let step = Duration::minutes(15);

        let available_slots = calculate_available_slots(
            start_time,
            end_time,
            &busy_periods,
            duration,
            work_start,
            work_end,
            &working_days,
            buffer,
            step,
        );

        // Assert that the available slots are as expected
        assert!(!available_slots.is_empty());
        // Add more specific assertions based on the expected behavior
    }
}
```

### Integration Tests

Integration tests verify that different components work together correctly. They test the interaction between components and ensure that they integrate properly.

In Connectify, integration tests are placed in the `tests` directory of each crate. They use the public API of the crate to test the integration of different components.

Example of an integration test:

```rust
// tests/integration_test.rs
use connectify_gcal::service::GoogleCalendarService;
use connectify_common::services::CalendarService;

#[tokio::test]
async fn test_create_and_get_events() {
    // Create a test calendar service
    let service = GoogleCalendarService::new(/* ... */);
    
    // Create a test event
    let event = CalendarEvent {
        start_time: "2025-05-15T10:00:00Z".to_string(),
        end_time: "2025-05-15T11:00:00Z".to_string(),
        summary: "Test Event".to_string(),
        description: Some("This is a test event".to_string()),
    };
    
    // Create the event
    let result = service.create_event("test-calendar", event.clone()).await.unwrap();
    
    // Verify the event was created
    assert!(result.event_id.is_some());
    assert_eq!(result.status, "confirmed");
    
    // Get the booked events
    let events = service.get_booked_events(
        "test-calendar",
        Utc::now(),
        Utc::now() + Duration::days(7),
        false,
    ).await.unwrap();
    
    // Verify the event is in the list
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].summary, "Test Event");
}
```

### End-to-End Tests

End-to-end tests verify that the entire application works as expected from the user's perspective. They test the application as a whole, including all its components and external dependencies.

In Connectify, end-to-end tests are placed in the `tests/e2e` directory of the main application crate. They use the public API of the application to test the entire flow from the user's perspective.

Example of an end-to-end test:

```rust
// tests/e2e/booking_flow_test.rs
use axum::http::{Request, StatusCode};
use axum::body::Body;
use tower::ServiceExt;

#[tokio::test]
async fn test_booking_flow() {
    // Create a test application
    let app = create_test_app().await;
    
    // Get available slots
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/gcal/availability?start_date=2025-05-15&end_date=2025-05-15&duration_minutes=60")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Parse the response body to get the available slots
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let available_slots: AvailableSlotsResponse = serde_json::from_slice(&body).unwrap();
    
    // Book the first available slot
    let slot = &available_slots.slots[0];
    let request_body = serde_json::json!({
        "start_time": slot.start_time,
        "end_time": slot.end_time,
        "summary": "Test Booking",
        "description": "This is a test booking"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/gcal/book")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Parse the response body to get the booking result
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let booking_response: BookingResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(booking_response.success);
    assert!(booking_response.event_id.is_some());
}
```

### Property-Based Tests

Property-based tests verify that certain properties of the code hold for a wide range of inputs. They generate random inputs and check that the code behaves as expected for all of them.

In Connectify, we use the `proptest` crate for property-based testing. Property-based tests are typically placed in the same file as the code they test, inside a `#[cfg(test)]` module.

Example of a property-based test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_calculate_available_slots_properties(
            start_hour in 0..24u32,
            end_hour in 0..24u32,
            duration_minutes in 15..120i64,
        ) {
            // Ensure start_hour < end_hour
            let start_hour = start_hour % 24;
            let end_hour = (start_hour + 1 + (end_hour % 23)) % 24;
            
            let start_time = Utc::now().date_naive().and_hms_opt(start_hour, 0, 0).unwrap().and_local_timezone(Utc).unwrap();
            let end_time = Utc::now().date_naive().and_hms_opt(end_hour, 0, 0).unwrap().and_local_timezone(Utc).unwrap();
            let busy_periods = vec![];
            let duration = Duration::minutes(duration_minutes);
            let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
            let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
            let working_days = vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri];
            let buffer = Duration::minutes(0);
            let step = Duration::minutes(15);

            let available_slots = calculate_available_slots(
                start_time,
                end_time,
                &busy_periods,
                duration,
                work_start,
                work_end,
                &working_days,
                buffer,
                step,
            );

            // Property: All slots should be within the working hours
            for slot in &available_slots {
                let slot_time = slot.time();
                prop_assert!(slot_time >= work_start && slot_time <= work_end);
            }

            // Property: All slots should be on working days
            for slot in &available_slots {
                let slot_day = slot.weekday();
                prop_assert!(working_days.contains(&slot_day));
            }

            // Property: No slots should overlap with busy periods
            for slot in &available_slots {
                let slot_end = *slot + duration;
                for (busy_start, busy_end) in &busy_periods {
                    prop_assert!(slot_end <= *busy_start || *slot >= *busy_end);
                }
            }
        }
    }
}
```

### Contract Tests

Contract tests verify that the integration points between different services work as expected. They test the contracts between services and ensure that they adhere to the agreed-upon interfaces.

In Connectify, contract tests are placed in the `tests/contract` directory of each crate that integrates with external services. They use mock implementations of the external services to test the contracts.

Example of a contract test:

```rust
// tests/contract/stripe_contract_test.rs
use connectify_stripe::service::StripePaymentService;
use connectify_common::services::PaymentService;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_stripe_payment_service_contract() {
    // Start a mock server
    let mock_server = MockServer::start().await;
    
    // Mock the Stripe API
    Mock::given(method("POST"))
        .and(path("/v1/checkout/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "cs_test_abc123",
            "object": "checkout.session",
            "amount_total": 7500,
            "currency": "chf",
            "status": "open",
            "url": "https://checkout.stripe.com/pay/cs_test_abc123"
        })))
        .mount(&mock_server)
        .await;
    
    // Create a test payment service that uses the mock server
    let service = StripePaymentService::new_with_base_url(
        config.clone(),
        mock_server.uri(),
    );
    
    // Test the create_payment_intent method
    let result = service.create_payment_intent(
        7500,
        "chf",
        Some("Test Payment"),
        None,
    ).await.unwrap();
    
    // Verify the result matches the expected contract
    assert_eq!(result.id, "cs_test_abc123");
    assert_eq!(result.status, "open");
    assert_eq!(result.amount, 7500);
    assert_eq!(result.currency, "chf");
}
```

### Performance Tests

Performance tests verify that the application meets the performance requirements. They measure the response time, throughput, and resource usage of the application under different load conditions.

In Connectify, performance tests are placed in the `benches` directory of each crate. They use the `criterion` crate for benchmarking.

Example of a performance test:

```rust
// benches/calendar_service_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use connectify_gcal::service::GoogleCalendarService;
use connectify_common::services::CalendarService;

fn bench_get_busy_times(c: &mut Criterion) {
    let service = GoogleCalendarService::new(/* ... */);
    let calendar_id = "test-calendar";
    let start_time = Utc::now();
    let end_time = start_time + Duration::days(7);
    
    c.bench_function("get_busy_times", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                black_box(
                    service.get_busy_times(
                        black_box(calendar_id),
                        black_box(start_time),
                        black_box(end_time),
                    ).await
                )
            });
    });
}

criterion_group!(benches, bench_get_busy_times);
criterion_main!(benches);
```

## Testing Tools

Connectify uses the following testing tools:

- **Rust's built-in testing framework**: For unit and integration tests.
- **tokio**: For asynchronous testing.
- **mockall**: For creating mock implementations of traits for testing.
- **wiremock**: For mocking HTTP services.
- **proptest**: For property-based testing.
- **criterion**: For benchmarking and performance testing.

## Test Organization

Tests in Connectify are organized as follows:

- **Unit tests**: Placed in the same file as the code they test, inside a `#[cfg(test)]` module.
- **Integration tests**: Placed in the `tests` directory of each crate.
- **End-to-end tests**: Placed in the `tests/e2e` directory of the main application crate.
- **Contract tests**: Placed in the `tests/contract` directory of each crate that integrates with external services.
- **Performance tests**: Placed in the `benches` directory of each crate.

## Writing Testable Code

To make code more testable, follow these guidelines:

1. **Use dependency injection**: Pass dependencies as parameters or use a dependency injection framework.
2. **Use interfaces (traits)**: Define interfaces for components and use them instead of concrete implementations.
3. **Keep functions small and focused**: Small functions are easier to test and understand.
4. **Avoid global state**: Global state makes testing difficult because it's hard to isolate.
5. **Use mock implementations**: Create mock implementations of interfaces for testing.

## Test Coverage

We aim for high test coverage, but we focus on testing the most critical parts of the application first. We use the following guidelines for test coverage:

- **Core business logic**: 90-100% coverage
- **API endpoints**: 80-90% coverage
- **Utility functions**: 70-80% coverage
- **Configuration and setup code**: 50-70% coverage

To check test coverage, we use the `grcov` tool:

```bash
cargo install grcov
RUSTFLAGS="-Zinstrument-coverage" LLVM_PROFILE_FILE="your_name-%p-%m.profraw" cargo test
grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing -o ./target/debug/coverage/
```

## Continuous Integration

We use continuous integration (CI) to run tests automatically on every push and pull request. Our CI pipeline includes:

1. **Building the project**: Ensures that the code compiles.
2. **Running unit tests**: Ensures that individual components work as expected.
3. **Running integration tests**: Ensures that components work together correctly.
4. **Running end-to-end tests**: Ensures that the entire application works as expected.
5. **Checking test coverage**: Ensures that the code is adequately tested.
6. **Running linters**: Ensures that the code follows the project's style guidelines.

## Best Practices

Here are some best practices for testing in Connectify:

1. **Write tests first**: Follow test-driven development (TDD) when possible.
2. **Keep tests simple**: Tests should be easy to understand and maintain.
3. **Test one thing at a time**: Each test should focus on a single aspect of the code.
4. **Use descriptive test names**: Test names should describe what the test is checking.
5. **Use setup and teardown**: Use setup and teardown functions to avoid duplication.
6. **Avoid test interdependence**: Tests should not depend on each other.
7. **Test edge cases**: Test boundary conditions and error cases.
8. **Keep tests fast**: Tests should run quickly to provide fast feedback.
9. **Use test doubles**: Use mocks, stubs, and fakes to isolate the code being tested.
10. **Refactor tests**: Refactor tests to keep them clean and maintainable.