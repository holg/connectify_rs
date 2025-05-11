# Connectify Fulfillment

**connectify-fulfillment** orchestrates post-payment fulfillment tasks for Connectify services, such as booking Google Calendar events or sending Twilio notifications after successful payments.

## Table of Contents

- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [API Routes](#api-routes)
- [OpenAPI Documentation](#openapi-documentation)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Internal fulfillment endpoints** protected by a shared-secret header
- **Google Calendar bookings** after payment (requires `gcal` feature)
- **Twilio actions** or other service integrations (requires `twilio` feature)
- **Config-driven** via `connectify-config`
- **Optional OpenAPI schemas** for documenting fulfillment APIs (`openapi` feature)

## Prerequisites

- Rust (1.65+)
- Cargo
- An existing Connectify service that triggers fulfillment (e.g., Stripe webhook)
- A shared secret configured for internal API authentication

## Installation

In your service’s `Cargo.toml`, add:
```toml
connectify-fulfillment = { path = "../connectify_fulfillment", features = ["gcal", "twilio", "openapi"] }
```
Enable only the features you need (`gcal`, `twilio`, `openapi`).

## Configuration

Load the shared-secret in your `config.yaml` under the `fulfillment` section:
```yaml
fulfillment:
  shared_secret: "your_internal_shared_secret"
```
Ensure any feature-specific configs are also present:
```yaml
use_gcal: true
gcal:
  key_path: "/path/to/service_account.json"
  calendar_id: "your-cal-id@group.calendar.google.com"
use_twilio: true
twilio:
  account_sid: "AC..."
  api_key_sid: "SK..."
  api_key_secret: "..."
```

## Usage

Merge the fulfillment router into your Axum application:
```rust
use std::sync::Arc;
use axum::Router;
use connectify_config::load_config;
use connectify_fulfillment::routes;

#[tokio::main]
async fn main() {
    let config = Arc::new(load_config().unwrap());

    // If using GCal/Twilio features, prepare their states as in your main service
    let gcal_state = /* Option<Arc<GcalState>> */;
    let twilio_state = /* Option<Arc<TwilioState>> */;

    let app = Router::new()
        .nest(
            "/fulfillment",
            routes(config.clone(), gcal_state, twilio_state)
        );

    // Serve `app`...
}
```

## API Routes

| Method | Path                         | Description                                   |
| ------ | ---------------------------- | --------------------------------------------- |
| POST   | `/fulfillment/gcal-booking`  | Trigger a Google Calendar booking fulfillment |
| POST   | `/fulfillment/twilio-...`    | Trigger a Twilio action (future)              |

All requests must include the header:
```http
X-Internal-Auth-Secret: your_internal_shared_secret
```

## OpenAPI Documentation

When compiled with `--features openapi`, `FulfillmentApiDoc` provides schemas and paths that can be merged into your main OpenAPI spec. No standalone Swagger UI is served by this crate.

## Examples

```bash
curl -X POST http://localhost:8080/fulfillment/gcal-booking \
  -H "Content-Type: application/json" \
  -H "X-Internal-Auth-Secret: your_internal_shared_secret" \
  -d '{
    "start_time": "2025-06-10T10:00:00Z",
    "end_time": "2025-06-10T11:00:00Z",
    "summary": "Post-payment booking",
    "description": "Booked after Stripe payment",
    "original_reference_id": "stripe_tx_123abc"
}'
```

## Contributing

1. Fork the repo
2. Create a feature branch
3. Run `cargo fmt`, `cargo clippy`, and tests
4. Submit a pull request with documentation updates

## License

MIT OR Apache-2.0
