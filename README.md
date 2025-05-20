## CI Status

| Build & Test & Fmt & Clippy |
|:---------------------------:|
| [![Rust Tests](https://github.com/holg/connectify_rs/actions/workflows/rust-tests.yml/badge.svg?branch=main)](https://github.com/holg/connectify_rs/actions/workflows/rust-tests.yml) |
[Test, Clippy, Rustfmt, Code coverage, Benchmark, clippy]

# Connectify-RS: Modular API Service

Connectify-RS is a modular backend service built in Rust using the Axum web framework. Designed as a Cargo workspace, it integrates various external services—Twilio for communication, Google Calendar for scheduling, Stripe and Payrexx for payments—and includes an internal fulfillment service to coordinate post-payment actions.

## Table of Contents
1. [Project Overview](#project-overview)
2. [Features](#features)
3. [Workspace Structure](#workspace-structure)
4. [Prerequisites](#prerequisites)
5. [Configuration](#configuration)
   - [Environment Variables (`.env`)](#environment-variables-env)
   - [Configuration Files (`config/`)](#configuration-files-config)
6. [Building and Running](#building-and-running)
   - [Building](#building)
   - [Running the Backend Service](#running-the-backend-service)
   - [Enabling Features](#enabling-features)
7. [API Documentation (Swagger UI)](#api-documentation-swagger-ui)
8. [Crate Details](#crate-details)
9. [Testing](#testing)
10. [Security Considerations](#security-considerations)
11. [Contributing](#contributing)
12. [License](#license)

## Project Overview

Connectify-RS serves as a backend hub, enabling seamless integration with various third-party services. Its modular architecture, built using a Cargo workspace, allows features to be selectively enabled or disabled, making it ideal for applications requiring communication, scheduling, and payment processing. The core is an Axum-based web server exposing API endpoints for these integrations.

## Features

- **Modular Design:** Enable or disable integrations via Cargo features and runtime flags.
- **Centralized Configuration:** Unified management with `config-rs` and `.env` support.
- **Twilio Integration:** Generate Twilio Video access tokens.
- **Google Calendar Integration:** Check availability and book events with a Service Account.
- **Payment Processing:**
  - **Stripe:** Stripe Checkout Sessions & webhooks.
  - **Payrexx:** Payment links & webhooks.
- **Fulfillment Service:** Internal API for post-payment actions (e.g., calendar booking).
- **API Documentation:** Auto-generated OpenAPI/Swagger UI via `utoipa`.

## Workspace Structure

```text
connectify_rs/
├── Cargo.toml                # Root workspace manifest
├── config/
│   └── default.yml           # Default configuration values
├── crates/                   # Modular libraries and executables
│   ├── connectify_config     # Config loader and models (core)
│   ├── connectify_common     # Shared utilities (placeholder)
│   ├── connectify_payrexx    # Payrexx integration
│   ├── connectify_stripe     # Stripe integration
│   ├── connectify_twilio     # Twilio integration
│   ├── connectify_gcal       # Google Calendar integration
│   ├── connectify_calendly   # Calendly integration (WIP)
│   ├── connectify_fulfillment# Fulfillment workflows
│   └── services/
│       ├── connectify_backend# Main Axum API service
│       └── rustdis/          # Experimental placeholder service
└── cross_build_on_mac.sh     # Cross-compilation helper script
```

## Prerequisites

- **Rust & Cargo:** Latest stable toolchain.
- **Credentials & Accounts:**
  - Twilio: Account SID, API Key SID & Secret.
  - Google Calendar: Service Account JSON key with Calendar API.
  - Stripe: API Secret Key & Webhook Signing Secret.
  - Payrexx: Instance Name & API Secret.
- **(Optional) ngrok:** For webhook testing.

## Configuration

Configuration is layered (file → environment):

### Environment Variables (`.env`)
Create a `.env` in the workspace root (add to `.gitignore`):

- `RUN_ENV` (e.g., `development`, defaults to `debug`).
- `PREFIX` (env var prefix for config, default `HTR`).
- **Secrets (must not be committed):**
  - `TWILIO_API_KEY_SECRET`
  - `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`
  - `PAYREXX_API_SECRET`
  - `GOOGLE_SERVICE_ACCOUNT_KEY_PATH`
  - `FULFILLMENT_SHARED_SECRET`
  - Other prefixed vars: `HTR_GCAL__CALENDAR_ID`, etc.

### Configuration Files (`config/`)
- `config/default.yml`: Base settings.
- `config/{RUN_ENV}.yml`: Environment-specific overrides.

These map to the `AppConfig` in `connectify_config` (fields like `use_twilio`, `twilio`, `gcal`, etc.).

## Building and Running
All commands run from the workspace root.

### Building
- Build all crates with all features:
  ```bash
  cargo build --all-features --release
  ```
- Build only the backend with specific features:
  ```bash
  cargo build -p connectify-backend --features twilio,gcal,stripe,payrexx,fulfillment,openapi --release
  ```

### Running the Backend Service
The `connectify-backend` executable:
```bash
cargo run -p connectify-backend --features twilio,gcal,stripe,payrexx,fulfillment,openapi -- .env
```
- The `-- .env` argument specifies a dotenv path for `ensure_dotenv_loaded` (defaults to `.env`).
- The console will show the bind address (e.g., `http://127.0.0.1:8080`).

### Enabling Features
- **Compile-time:** `--features` flags for integrations and `openapi`.
- **Runtime:** `use_XXX: bool` in config (e.g., `use_twilio: true`).
Routes are active only if both compile- and run-time flags are enabled.

## API Documentation (Swagger UI)
With `openapi` enabled, access Swagger UI at:
```text
http://<host>:<port>/api/docs
```
The raw OpenAPI JSON is at `/api/docs/openapi.json`.
Documentation includes all compiled feature endpoints.

## Crate Details
See individual crate READMEs under `crates/` for full API and configuration specifics.

## Testing
- Run tests per crate: `cargo test -p <crate> --features <features>`.
- Run all tests: `cargo test --all-features`.
- Use `ngrok` or Stripe CLI for webhook testing.

## Security Considerations
- **Secrets:** Never commit secrets; use env vars.
- **Webhook Verification:** Validate signatures for Stripe/Payrexx.
- **Internal API Security:** Protect fulfillment endpoints with `X-Internal-Auth-Secret`.
- **HTTPS:** Use TLS in production.
- **Dependencies:** Keep up-to-date; run `cargo audit`.

## Contributing
1. Fork the repo and create a feature branch.
2. Run `cargo fmt`, `cargo clippy`, and tests.
3. Update relevant README(s) and this overview.
4. Submit a pull request.

## License
This project is licensed under MIT OR Apache-2.0 (see crates’ `Cargo.toml`).