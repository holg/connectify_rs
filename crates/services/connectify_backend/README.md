

# Connectify Backend

**Connectify Backend** is the core HTTP API service powering Connectify’s integrations and payment workflows. It’s built in Rust with Axum and Tokio for high performance, and is fully modular via Cargo feature flags.

## Table of Contents

- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Feature Flags](#feature-flags)
- [API Documentation (OpenAPI)](#api-documentation-openapi)
- [Development Mode](#development-mode)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Modular Integrations**
  - Twilio SMS/webhooks
  - Google Calendar (GCal)
  - Stripe payments
  - Payrexx payments
  - Custom Fulfillment logic
- **OpenAPI / Swagger UI**
- **Asynchronous runtime** with Tokio
- **Configuration-driven** (via `connectify-config`)
- **Static file serving** in development mode

## Prerequisites

- **Rust** (latest stable toolchain)
- **Cargo** (comes with Rust)
- **Git** (to clone the repo)

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/connectify_rs.git
   cd connectify_rs/crates/services/connectify_backend
   ```
2. Build with default features:
   ```bash
   cargo build --release
   ```
3. To enable specific integrations, pass the corresponding Cargo feature(s):
   ```bash
   cargo build --features "twilio gcal stripe openapi"
   ```

## Configuration

This service uses a configuration file compatible with the `connectify-config` crate. By default, it looks for `config.yaml` in the working directory.

Example `config.yaml`:
```yaml
server:
  host: "0.0.0.0"
  port: 8080
use_twilio: true
twilio:
  account_sid: "AC..."
  auth_token: "xyz"
use_gcal: true
gcal:
  client_id: "..."
  client_secret: "..."
  token_path: "/path/to/token.json"
use_stripe: true
stripe:
  api_key: "sk_..."
use_payrexx: false
payrexx:
  api_key: "..."
use_fulfillment: false
```

## Usage

Run the service:
```bash
cargo run --features "twilio gcal stripe openapi"
```

By default, the API is available at `http://<host>:<port>/api`.

## Feature Flags

| Feature         | Description                           |
| --------------- | ------------------------------------- |
| `twilio`        | Enable Twilio SMS/webhook endpoints  |
| `gcal`          | Enable Google Calendar integration   |
| `stripe`        | Enable Stripe payment endpoints      |
| `payrexx`       | Enable Payrexx payment endpoints     |
| `fulfillment`   | Enable custom fulfillment endpoints  |
| `openapi`       | Generate OpenAPI spec + Swagger UI   |

## API Documentation (OpenAPI)

When built with the `openapi` feature, Swagger UI is served at:

```
http://<host>:<port>/api/docs
```

And the raw OpenAPI JSON is available at:

```
http://<host>:<port>/api/docs/openapi.json
```

## Development Mode

In debug builds (`cargo run` without `--release`), the service also serves static files from `../../dist` under `/static`, useful for local front-end development.

## Contributing

Contributions are welcome! Please open issues or pull requests in the main repository. Ensure all new code is covered by tests and adheres to the project’s coding guidelines.

## License

This project is licensed under the MIT License. See [LICENSE](../../LICENSE) for details.