## CI Status

| Build & Test & Fmt & Clippy |
|:---------------------------:|
| [![Rust Tests](https://github.com/holg/connectify_rs/actions/workflows/rust-tests.yml/badge.svg?branch=main)](https://github.com/holg/connectify_rs/actions/workflows/rust-tests.yml) |
[Test, Clippy, Rustfmt, Code coverage, Benchmark, clippy]


# Connectify Google Calendar Integration

**connectify-gcal** provides Google Calendar booking and availability endpoints for Connectify services, built with Rust and Axum.

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

- Fetch available time slots for booking
- Create, delete, and cancel Google Calendar events
- Retrieve existing booked events within a date range
- Config-driven via `connectify-config`
- Asynchronous HTTP handlers with Axum
- Optional OpenAPI schemas and Swagger UI support (`openapi` feature)

## Prerequisites

- Rust (1.65+)
- Cargo
- A Google Service Account JSON key for Calendar API
- `connectify-config` for loading `GcalConfig`

## Installation

Add to your project’s `Cargo.toml`:
```toml
connectify-gcal = { path = "../connectify_gcal", features = ["openapi"] }
```

## Configuration

Configure your `config.yaml` (or environment variables) for the Google Calendar service:
```yaml
use_gcal: true
gcal:
  key_path: "/path/to/service_account.json"
  calendar_id: "your-calendar-id@group.calendar.google.com"
```
Ensure the JSON key file is accessible at `key_path`.

## Usage

In your application, merge the GCal routes under an API prefix:
```rust
use connectify_gcal::routes;

#[tokio::main]
async fn main() {
    let config = load_config().unwrap();
    let app = Router::new()
        .nest("/gcal", routes(Arc::new(config)).await);
    // serve `app` with Axum
}
```

## API Routes

| Method | Path                       | Description                                 |
| ------ | -------------------------- | ------------------------------------------- |
| GET    | `/availability`            | List available time slots                   |
| POST   | `/book`                    | Book an event (JSON body)                   |
| DELETE | `/delete/{event_id}`       | Delete an event by ID                       |
| PATCH  | `/mark_cancelled/{event_id}` | Mark an event as cancelled                  |
| GET    | `/bookings`                | Get booked events in a date range           |

## OpenAPI Documentation

Enable the `openapi` feature to derive `GcalApiDoc`, which can be merged into your service’s OpenAPI spec and served via Swagger UI.

## Examples

```bash
# Check availability from May 1 to May 7 for 30-minute slots
curl "http://localhost:8080/gcal/availability?start_date=2025-05-01&end_date=2025-05-07&duration_minutes=30"

# Book a slot
curl -X POST http://localhost:8080/gcal/book \
  -H 'Content-Type: application/json' \
  -d '{"start_time":"2025-05-02T10:00:00Z","end_time":"2025-05-02T10:30:00Z","summary":"Consultation"}'
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a branch for your feature or fix
3. Run `cargo fmt`, `cargo clippy`, and existing tests
4. Submit a pull request with documentation updates

## License

MIT OR Apache-2.0