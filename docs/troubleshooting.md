# Connectify Troubleshooting Guide

This guide provides solutions for common issues that may arise when developing, deploying, or using the Connectify application.

## Table of Contents

- [API Issues](#api-issues)
- [Calendar Integration Issues](#calendar-integration-issues)
- [Payment Integration Issues](#payment-integration-issues)
- [Notification Integration Issues](#notification-integration-issues)
- [Configuration Issues](#configuration-issues)
- [Deployment Issues](#deployment-issues)
- [Performance Issues](#performance-issues)
- [Debugging Techniques](#debugging-techniques)

## API Issues

### API Endpoint Returns 404

**Symptoms:**
- API endpoint returns a 404 Not Found error
- Client receives "Resource not found" message

**Possible Causes:**
1. Incorrect URL path
2. Route not registered in the router
3. Feature flag for the route is disabled

**Solutions:**
1. Verify the URL path is correct
2. Check if the route is registered in the router:
   ```
   // In routes.rs
   pub fn routes() -> Router {
       Router::new()
           .route("/api/endpoint", get(handler))
           // ...
   }
   ```
3. Check if the feature flag for the route is enabled in the configuration:
   ```
   // In config/default.yml
   use_feature: true
   ```

### API Endpoint Returns 401 or 403

**Symptoms:**
- API endpoint returns a 401 Unauthorized or 403 Forbidden error
- Client receives "Unauthorized" or "Forbidden" message

**Possible Causes:**
1. Missing or invalid authentication token
2. Token has expired
3. User does not have permission to access the resource

**Solutions:**
1. Check if the authentication token is included in the request header:
   ```
   Authorization: Bearer your_token_here
   ```
2. Verify the token is valid and has not expired
3. Check if the user has the required permissions

### API Endpoint Returns 500

**Symptoms:**
- API endpoint returns a 500 Internal Server Error
- Server logs show an error message

**Possible Causes:**
1. Unhandled exception in the handler
2. Database connection issue
3. External service integration issue

**Solutions:**
1. Check the server logs for error messages:
   ```
   RUST_LOG=debug cargo run
   ```
2. Add error handling to the handler:
   ```
   match result {
       Ok(value) => Ok(Json(value)),
       Err(e) => {
           error!("Error: {}", e);
           Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()))
       }
   }
   ```
3. Check if the database and external services are available

## Calendar Integration Issues

### Cannot Create Calendar Events

**Symptoms:**
- Creating a calendar event fails
- Server logs show an error message related to the Google Calendar API

**Possible Causes:**
1. Invalid or expired service account credentials
2. Missing or incorrect calendar ID
3. Insufficient permissions for the service account

**Solutions:**
1. Check if the service account credentials are valid:
   ```
   // In config/default.yml
   gcal:
     key_path: "./service_account_key.json"
   ```
2. Verify the calendar ID is correct:
   ```
   // In config/default.yml
   gcal:
     calendar_id: "primary"
   ```
3. Ensure the service account has the required permissions:
   - Go to the Google Cloud Console
   - Navigate to IAM & Admin > Service Accounts
   - Select the service account
   - Add the required roles (e.g., Calendar API > Calendar Editor)

### Calendar Events Not Showing Up

**Symptoms:**
- Calendar events are created successfully but do not appear in the calendar
- Server logs show no error messages

**Possible Causes:**
1. Events are created in a different calendar
2. Events are created with incorrect time zone
3. Events are created with visibility set to private

**Solutions:**
1. Check if the events are created in the correct calendar:
   ```
   let result = calendar_service.create_event("primary", event).await?;
   ```
2. Ensure the time zone is set correctly:
   ```
   let event = CalendarEvent {
       start_time: "2025-05-15T10:00:00Z".to_string(), // Note the "Z" for UTC
       end_time: "2025-05-15T11:00:00Z".to_string(),
       // ...
   };
   ```
3. Check if the events are created with the correct visibility:
   ```
   // In the GoogleCalendarService implementation
   let new_event = Event {
       // ...
       visibility: Some("public".to_string()),
       // ...
   };
   ```

### Calendar API Rate Limiting

**Symptoms:**
- Calendar API requests fail with a 429 Too Many Requests error
- Server logs show rate limiting error messages

**Possible Causes:**
1. Too many requests to the Google Calendar API
2. Insufficient quota for the Google Calendar API

**Solutions:**
1. Implement rate limiting for the Google Calendar API:
   ```
   // In the GoogleCalendarService implementation
   let rate_limiter = RateLimiter::new(
       // 100 requests per minute
       100,
       Duration::from_secs(60),
   );
   ```
2. Increase the quota for the Google Calendar API:
   - Go to the Google Cloud Console
   - Navigate to APIs & Services > Dashboard
   - Select the Calendar API
   - Click on Quotas
   - Request a quota increase

## Payment Integration Issues

### Payment Processing Fails

**Symptoms:**
- Payment processing fails
- Server logs show an error message related to the payment gateway

**Possible Causes:**
1. Invalid or expired API keys
2. Incorrect payment amount or currency
3. Test mode vs. live mode mismatch

**Solutions:**
1. Check if the API keys are valid:
   ```
   // In config/default.yml
   stripe:
     secret_key: "secret_from_env"
   ```
2. Verify the payment amount and currency:
   ```
   let result = payment_service.create_payment_intent(
       7500, // Amount in cents
       "chf", // Currency code
       // ...
   ).await?;
   ```
3. Ensure the API keys match the mode (test or live):
   - Test mode keys start with `sk_test_`
   - Live mode keys start with `sk_live_`

### Webhook Events Not Received

**Symptoms:**
- Webhook events are not received
- Payment status is not updated after payment completion

**Possible Causes:**
1. Webhook URL is not accessible from the internet
2. Webhook signature verification fails
3. Webhook events are not configured in the payment gateway

**Solutions:**
1. Ensure the webhook URL is accessible from the internet:
   - Use a service like ngrok for local development
   - Configure proper DNS and firewall rules for production
2. Verify the webhook signature:
   ```
   // In the webhook handler
   let signature = req.headers().get("Stripe-Signature").unwrap();
   let event = stripe::Webhook::construct_event(
       &body,
       signature,
       &webhook_secret,
   )?;
   ```
3. Configure the webhook events in the payment gateway:
   - Go to the Stripe Dashboard
   - Navigate to Developers > Webhooks
   - Add a new webhook endpoint
   - Select the events to send (e.g., `payment_intent.succeeded`)

### Refund Processing Fails

**Symptoms:**
- Refund processing fails
- Server logs show an error message related to the payment gateway

**Possible Causes:**
1. Payment has already been refunded
2. Payment is not in a refundable state
3. Refund amount exceeds the payment amount

**Solutions:**
1. Check if the payment has already been refunded:
   ```
   // In the payment service
   let payment = stripe.payment_intents.retrieve(&payment_intent_id).await?;
   if payment.status == "refunded" {
       return Err(PaymentError::AlreadyRefunded);
   }
   ```
2. Ensure the payment is in a refundable state:
   ```
   // In the payment service
   if payment.status != "succeeded" {
       return Err(PaymentError::NotRefundable);
   }
   ```
3. Verify the refund amount does not exceed the payment amount:
   ```
   // In the payment service
   if amount > payment.amount {
       return Err(PaymentError::RefundAmountTooLarge);
   }
   ```

## Notification Integration Issues

### Email Notifications Not Sent

**Symptoms:**
- Email notifications are not sent
- Server logs show an error message related to the email service

**Possible Causes:**
1. Invalid or expired API keys
2. Incorrect email address format
3. Email service is not configured correctly

**Solutions:**
1. Check if the API keys are valid:
   ```
   // In config/default.yml
   twilio:
     api_key_sid: "secret_from_env"
     api_key_secret: "secret_from_env"
   ```
2. Verify the email address format:
   ```
   let result = notification_service.send_email(
       "john@example.com", // Valid email address
       "Subject",
       "Body",
       false,
   ).await?;
   ```
3. Ensure the email service is configured correctly:
   - Check the Twilio SendGrid dashboard
   - Verify the sender email is verified
   - Check if there are any sending restrictions

### SMS Notifications Not Sent

**Symptoms:**
- SMS notifications are not sent
- Server logs show an error message related to the SMS service

**Possible Causes:**
1. Invalid or expired API keys
2. Incorrect phone number format
3. SMS service is not configured correctly

**Solutions:**
1. Check if the API keys are valid:
   ```
   // In config/default.yml
   twilio:
     account_sid: "secret_from_env"
     api_key_sid: "secret_from_env"
     api_key_secret: "secret_from_env"
   ```
2. Verify the phone number format:
   ```
   let result = notification_service.send_sms(
       "+41791234567", // E.164 format
       "Your appointment has been confirmed.",
   ).await?;
   ```
3. Ensure the SMS service is configured correctly:
   - Check the Twilio dashboard
   - Verify the sender phone number is verified
   - Check if there are any sending restrictions

## Configuration Issues

### Configuration Not Loaded

**Symptoms:**
- Application fails to start
- Server logs show an error message related to configuration loading

**Possible Causes:**
1. Configuration file not found
2. Configuration file has invalid format
3. Required environment variables not set

**Solutions:**
1. Check if the configuration file exists:
   ```
   ls -la config/default.yml
   ```
2. Verify the configuration file has valid format:
   ```
   cat config/default.yml
   ```
3. Ensure the required environment variables are set:
   ```
   export CONNECTIFY__SERVER__HOST=127.0.0.1
   export CONNECTIFY__SERVER__PORT=8086
   ```

### Feature Flags Not Working

**Symptoms:**
- Features that should be enabled are disabled
- Features that should be disabled are enabled

**Possible Causes:**
1. Feature flags not set correctly in the configuration
2. Environment variables overriding the configuration
3. Feature flags not checked correctly in the code

**Solutions:**
1. Check if the feature flags are set correctly in the configuration:
   ```
   // In config/default.yml
   use_gcal: true
   use_stripe: true
   use_twilio: true
   ```
2. Verify if environment variables are overriding the configuration:
   ```
   env | grep CONNECTIFY
   ```
3. Ensure the feature flags are checked correctly in the code:
   ```
   if is_feature_enabled(&config, config.use_gcal, config.gcal.as_ref()) {
       // Feature is enabled
   }
   ```

## Deployment Issues

### Docker Container Fails to Start

**Symptoms:**
- Docker container fails to start
- Docker logs show an error message

**Possible Causes:**
1. Missing or incorrect environment variables
2. Missing or incorrect volume mounts
3. Port conflicts

**Solutions:**
1. Check if the required environment variables are set:
   ```
   docker run -e CONNECTIFY__SERVER__HOST=0.0.0.0 -e CONNECTIFY__SERVER__PORT=8086 ...
   ```
2. Verify the volume mounts:
   ```
   docker run -v /path/to/config:/app/config ...
   ```
3. Ensure there are no port conflicts:
   ```
   docker run -p 8086:8086 ...
   ```

### Kubernetes Pod Fails to Start

**Symptoms:**
- Kubernetes pod fails to start
- Kubernetes logs show an error message

**Possible Causes:**
1. Missing or incorrect environment variables
2. Missing or incorrect volume mounts
3. Resource limits too low

**Solutions:**
1. Check if the required environment variables are set:
   ```
   kubectl describe pod connectify-backend
   ```
2. Verify the volume mounts:
   ```
   kubectl describe pod connectify-backend
   ```
3. Ensure the resource limits are sufficient:
   ```
   kubectl describe pod connectify-backend
   ```

## Performance Issues

### High Latency

**Symptoms:**
- API requests take a long time to complete
- Server logs show slow response times

**Possible Causes:**
1. Inefficient database queries
2. Slow external service calls
3. Insufficient resources

**Solutions:**
1. Optimize database queries:
   - Add indexes
   - Use query caching
   - Optimize query structure
2. Implement caching for external service calls:
   ```
   // In the service implementation
   let cache_key = format!("calendar_events:{}:{}", calendar_id, date);
   if let Some(cached_events) = cache.get(&cache_key) {
       return Ok(cached_events);
   }
   
   let events = fetch_events_from_api().await?;
   cache.set(&cache_key, events.clone(), Duration::from_secs(300));
   Ok(events)
   ```
3. Increase resources:
   - Scale horizontally (add more instances)
   - Scale vertically (add more CPU/memory)

### High Memory Usage

**Symptoms:**
- Application uses a lot of memory
- Application crashes with out-of-memory errors

**Possible Causes:**
1. Memory leaks
2. Large data structures in memory
3. Inefficient memory usage

**Solutions:**
1. Use memory profiling tools to identify memory leaks:
   ```
   RUSTFLAGS="-Z instrument-memory-profile" cargo run
   ```
2. Optimize large data structures:
   - Use more efficient data structures
   - Implement pagination for large data sets
   - Use streaming for large responses
3. Implement more efficient memory usage:
   - Use references instead of cloning data
   - Use `Arc` for shared ownership
   - Use `Box` for large data structures

## Debugging Techniques

### Enabling Debug Logging

To enable debug logging, set the `RUST_LOG` environment variable:

```
RUST_LOG=debug cargo run
```

For more granular control, you can specify the log level for specific modules:

```
RUST_LOG=connectify_gcal=debug,connectify_stripe=info cargo run
```

### Using the Debugger

To debug the application using VS Code:

1. Install the [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension.
2. Install the [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) extension.
3. Create a `.vscode/launch.json` file with the following content:
   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug executable",
         "cargo": {
           "args": ["build", "--bin=connectify-backend"],
           "filter": {
             "name": "connectify-backend",
             "kind": "bin"
           }
         },
         "args": [],
         "cwd": "${workspaceFolder}",
         "env": {
           "RUST_LOG": "debug"
         }
       }
     ]
   }
   ```
4. Press F5 to start debugging.

### Inspecting HTTP Requests

To inspect HTTP requests, you can use the `curl` command with the `-v` flag:

```
curl -v -X GET http://localhost:8086/api/gcal/availability?start_date=2025-05-15&end_date=2025-05-15&duration_minutes=60
```

For more complex requests, you can use a tool like [Postman](https://www.postman.com/) or [Insomnia](https://insomnia.rest/).

### Monitoring External Service Calls

To monitor external service calls, you can use the `tracing` crate to add spans and events:

```
use tracing::{info, instrument};

#[instrument(skip(client))]
async fn fetch_events(client: &Client, calendar_id: &str) -> Result<Vec<Event>, Error> {
    info!("Fetching events for calendar {}", calendar_id);
    let response = client.get(&format!("/calendars/{}/events", calendar_id))
        .send()
        .await?;
    info!("Received response with status {}", response.status());
    let events = response.json().await?;
    info!("Parsed {} events", events.len());
    Ok(events)
}
```

Then, you can use a tool like [Jaeger](https://www.jaegertracing.io/) to visualize the traces.