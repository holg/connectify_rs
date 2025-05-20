## CI Status

| Build & Test & Fmt & Clippy |
|:---------------------------:|
| [![Rust Tests](https://github.com/holg/connectify_rs/actions/workflows/rust-tests.yml/badge.svg?branch=main)](https://github.com/holg/connectify_rs/actions/workflows/rust-tests.yml) |
[Test, Clippy, Rustfmt, Code coverage, Benchmark, clippy]


# Connectify Twilio

**connectify-twilio** provides a simple HTTP API for generating Twilio Video access tokens, built with Rust and Axum. It integrates seamlessly with the `connectify-config` crate for configuration and supports optional OpenAPI documentation.

## Table of Contents

- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [API Routes](#api-routes)
- [OpenAPI Documentation](#openapi-documentation)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Features

- Generate Twilio Video access tokens via `/generate-token`
- Query parameters for user identity and room name
- Built on Axum for lightweight, async HTTP handling
- Config-driven via `connectify-config`
- Optional OpenAPI schemas and Swagger UI support (with `openapi` feature)

## Prerequisites

- Rust (1.65+)
- Cargo
- A Twilio account with API Key SID and Secret

## Installation

Add to your project’s `Cargo.toml`:
```toml
connectify-twilio = { path = "../connectify_twilio", features = ["openapi"] }
```

Then in your code, merge the routes:
```rust
use connectify_twilio::routes;

let app = Router::new()
    .nest("/twilio", routes(config.clone()));
```

## Usage

Run your service, ensuring the `twilio` feature is enabled:
```bash
cargo run --features "openapi"
```

Then request a token:
```bash
curl "http://localhost:8080/twilio/generate-token?identity=User_123&roomName=MyRoom"
```

## Configuration

This crate relies on `connectify-config` for loading `TwilioConfig`:

```yaml
use_twilio: true
twilio:
  account_sid: "ACxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
  api_key_sid: "SKxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
  api_key_secret: "your-twilio-api-key-secret"
```

Ensure `api_key_secret` is set via environment variable or config file.

## API Routes

| Method | Path               | Query Parameters                | Description                    |
| ------ | ------------------ | ------------------------------- | ------------------------------ |
| GET    | `/generate-token`  | `identity` (String), `roomName` (String) | Returns a JSON-formatted Twilio access token |

## OpenAPI Documentation

When compiled with `--features openapi`, the `TwilioApiDoc` definitions can be merged into your main OpenAPI spec. Swagger UI support is provided via `utoipa-swagger-ui` on the Axum app.

## Examples

```bash
# Generate a token for user "Alice" in room "ChatRoom"
curl "http://localhost:8080/twilio/generate-token?identity=Alice&roomName=ChatRoom"
```

Response:
```json
{ "token": "<JWT_ACCESS_TOKEN>" }
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Run `cargo fmt` and `cargo clippy`
4. Submit a pull request with tests and documentation updates

## License

MIT © Holger Trahe