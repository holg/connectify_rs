# Connectify Integration Points

This document describes how Connectify integrates with external services, including authentication, API usage, data mapping, and error handling.

## Table of Contents

- [Google Calendar Integration](#google-calendar-integration)
- [Stripe Integration](#stripe-integration)
- [Payrexx Integration](#payrexx-integration)
- [Twilio Integration](#twilio-integration)
- [Integration Best Practices](#integration-best-practices)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)
- [Monitoring](#monitoring)

## Google Calendar Integration

Connectify integrates with Google Calendar to manage calendar events, check availability, and book appointments.

### Authentication

Connectify uses OAuth 2.0 with a service account to authenticate with the Google Calendar API. The service account credentials are stored in a JSON key file, which is referenced in the configuration.

```
gcal:
  key_path: "./service_account_key.json"
  calendar_id: "primary"
```

### API Usage

The Google Calendar integration uses the following API endpoints:

1. **FreeBusy API**: To check availability for a given time range.
2. **Events API**: To create, update, delete, and query calendar events.

### Data Mapping

| Connectify Model | Google Calendar API |
|------------------|---------------------|
| `CalendarEvent.start_time` | `Event.start.dateTime` |
| `CalendarEvent.end_time` | `Event.end.dateTime` |
| `CalendarEvent.summary` | `Event.summary` |
| `CalendarEvent.description` | `Event.description` |

### Error Handling

The Google Calendar integration handles the following error scenarios:

1. **Authentication Errors**: If the service account credentials are invalid or expired, the integration will log an error and return an appropriate error response.
2. **API Errors**: If the Google Calendar API returns an error, the integration will log the error and return an appropriate error response.
3. **Rate Limiting**: If the Google Calendar API rate limits the requests, the integration will implement exponential backoff and retry the request.

### Example Usage

```
// Create a calendar event
let event = CalendarEvent {
    start_time: "2025-05-15T10:00:00Z".to_string(),
    end_time: "2025-05-15T11:00:00Z".to_string(),
    summary: "Consultation with John Doe".to_string(),
    description: Some("Initial consultation to discuss project requirements".to_string()),
};

let result = calendar_service.create_event("primary", event).await?;
```

## Stripe Integration

Connectify integrates with Stripe to process payments for bookings.

### Authentication

Connectify uses API keys to authenticate with the Stripe API. The API keys are stored in environment variables or a secure secrets management system.

```
stripe:
  secret_key: "secret_from_env"
```

### API Usage

The Stripe integration uses the following API endpoints:

1. **Checkout Sessions API**: To create checkout sessions for payment processing.
2. **Payment Intents API**: To create and manage payment intents.
3. **Refunds API**: To process refunds.

### Data Mapping

| Connectify Model | Stripe API |
|------------------|------------|
| `PaymentIntent.amount` | `PaymentIntent.amount` |
| `PaymentIntent.currency` | `PaymentIntent.currency` |
| `PaymentIntent.description` | `PaymentIntent.description` |
| `PaymentIntent.metadata` | `PaymentIntent.metadata` |

### Webhooks

Connectify uses Stripe webhooks to receive notifications about payment events. The following webhook events are handled:

1. `payment_intent.succeeded`: When a payment is successfully processed.
2. `payment_intent.payment_failed`: When a payment fails.
3. `checkout.session.completed`: When a checkout session is completed.

### Error Handling

The Stripe integration handles the following error scenarios:

1. **Authentication Errors**: If the API keys are invalid, the integration will log an error and return an appropriate error response.
2. **API Errors**: If the Stripe API returns an error, the integration will log the error and return an appropriate error response.
3. **Webhook Verification**: If a webhook signature is invalid, the integration will reject the webhook and log an error.

### Example Usage

```
// Create a payment intent
let result = payment_service.create_payment_intent(
    7500,
    "chf",
    Some("Consultation with John Doe"),
    Some(serde_json::json!({
        "booking_id": "abc123",
        "customer_email": "john@example.com"
    })),
).await?;
```

## Payrexx Integration

Connectify integrates with Payrexx as an alternative payment gateway.

### Authentication

Connectify uses API keys to authenticate with the Payrexx API. The API keys are stored in environment variables or a secure secrets management system.

```
payrexx:
  api_key: "secret_from_env"
  instance_name: "my-instance"
```

### API Usage

The Payrexx integration uses the following API endpoints:

1. **Gateway API**: To create payment pages.
2. **Transaction API**: To query transaction status.

### Data Mapping

| Connectify Model | Payrexx API |
|------------------|-------------|
| `PaymentIntent.amount` | `Gateway.amount` |
| `PaymentIntent.currency` | `Gateway.currency` |
| `PaymentIntent.description` | `Gateway.purpose` |
| `PaymentIntent.metadata` | `Gateway.fields` |

### Webhooks

Connectify uses Payrexx webhooks to receive notifications about payment events. The following webhook events are handled:

1. `transaction.success`: When a payment is successfully processed.
2. `transaction.declined`: When a payment is declined.

### Error Handling

The Payrexx integration handles the following error scenarios:

1. **Authentication Errors**: If the API keys are invalid, the integration will log an error and return an appropriate error response.
2. **API Errors**: If the Payrexx API returns an error, the integration will log the error and return an appropriate error response.
3. **Webhook Verification**: If a webhook signature is invalid, the integration will reject the webhook and log an error.

### Example Usage

```
// Create a payment page
let result = payment_service.create_payment_page(
    7500,
    "chf",
    Some("Consultation with John Doe"),
    Some(serde_json::json!({
        "booking_id": "abc123",
        "customer_email": "john@example.com"
    })),
).await?;
```

## Twilio Integration

Connectify integrates with Twilio to send notifications via email and SMS.

### Authentication

Connectify uses API keys to authenticate with the Twilio API. The API keys are stored in environment variables or a secure secrets management system.

```
twilio:
  account_sid: "secret_from_env"
  api_key_sid: "secret_from_env"
  api_key_secret: "secret_from_env"
```

### API Usage

The Twilio integration uses the following API endpoints:

1. **SMS API**: To send SMS notifications.
2. **Email API**: To send email notifications.

### Data Mapping

| Connectify Model | Twilio API |
|------------------|------------|
| `Notification.to` | `Message.to` |
| `Notification.body` | `Message.body` |
| `Notification.subject` | `Email.subject` |
| `Notification.is_html` | `Email.contentType` |

### Error Handling

The Twilio integration handles the following error scenarios:

1. **Authentication Errors**: If the API keys are invalid, the integration will log an error and return an appropriate error response.
2. **API Errors**: If the Twilio API returns an error, the integration will log the error and return an appropriate error response.
3. **Rate Limiting**: If the Twilio API rate limits the requests, the integration will implement exponential backoff and retry the request.

### Example Usage

```
// Send an SMS notification
let result = notification_service.send_sms(
    "+41791234567",
    "Your appointment has been confirmed for May 15, 2025 at 10:00 AM.",
).await?;

// Send an email notification
let result = notification_service.send_email(
    "john@example.com",
    "Appointment Confirmation",
    "Your appointment has been confirmed for May 15, 2025 at 10:00 AM.",
    false,
).await?;
```

## Integration Best Practices

When integrating with external services, follow these best practices:

1. **Use Dependency Injection**: Use the dependency injection pattern to make it easy to swap out implementations for testing.
2. **Handle Errors Gracefully**: Implement proper error handling for all external service calls.
3. **Implement Retries**: Use exponential backoff and retries for transient errors.
4. **Use Circuit Breakers**: Implement circuit breakers to prevent cascading failures.
5. **Monitor Integration Health**: Set up monitoring and alerting for integration health.
6. **Keep Credentials Secure**: Store API keys and credentials in a secure secrets management system.
7. **Validate Webhooks**: Verify webhook signatures to prevent security vulnerabilities.
8. **Log Integration Events**: Log all integration events for debugging and auditing.

## Error Handling

Connectify uses a consistent approach to error handling for external service integrations:

1. **Define Error Types**: Each integration defines its own error types that implement the `std::error::Error` trait.
2. **Map External Errors**: External service errors are mapped to internal error types.
3. **Provide Context**: Error messages include context about the operation that failed.
4. **Log Errors**: All errors are logged with appropriate context.
5. **Return Appropriate Status Codes**: Errors are mapped to appropriate HTTP status codes.

Example error type for the Google Calendar integration:

```
#[derive(Error, Debug)]
pub enum GcalError {
    #[error("Google API Error: {0}")]
    ApiError(#[from] google_calendar3::Error),
    #[error("Failed to parse time: {0}")]
    TimeParseError(String),
    #[error("Calculation error: {0}")]
    CalculationError(String),
    #[error("Booking conflict")]
    Conflict,
    #[error("No matching price tier found for duration: {0} minutes")]
    NoMatchingPriceTier(i64),
}
```

## Rate Limiting

Connectify implements rate limiting for external service integrations to prevent exceeding API rate limits:

1. **Token Bucket Algorithm**: Uses a token bucket algorithm to limit the rate of requests.
2. **Exponential Backoff**: Implements exponential backoff for retries.
3. **Jitter**: Adds jitter to retry intervals to prevent thundering herd problems.
4. **Circuit Breaker**: Implements a circuit breaker to prevent cascading failures.

Example rate limiting configuration:

```
// Configure rate limiter
let rate_limiter = RateLimiter::new(
    // 100 requests per minute
    100,
    Duration::from_secs(60),
    // Exponential backoff starting at 100ms, doubling each retry, with max 10 retries
    ExponentialBackoff::new(Duration::from_millis(100), 2.0, 10),
);
```

## Monitoring

Connectify monitors the health and performance of external service integrations:

1. **Health Checks**: Periodically checks the health of external services.
2. **Metrics**: Collects metrics on request rates, latency, and error rates.
3. **Logging**: Logs all integration events for debugging and auditing.
4. **Alerting**: Sets up alerts for integration health issues.

Example metrics collected:

- **Request Rate**: Number of requests per second to each external service.
- **Latency**: Time taken for each request to complete.
- **Error Rate**: Percentage of requests that result in errors.
- **Circuit Breaker Status**: Whether the circuit breaker is open, closed, or half-open.
- **Rate Limiter Status**: Number of requests allowed and rejected by the rate limiter.