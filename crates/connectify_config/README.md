

# Connectify Config

**connectify-config** is a centralized configuration loader for Connectify services, providing:

- Hierarchical config loading from files (`config/default`, `config/{RUN_ENV}`)
- Environment variable overrides with a customizable prefix
- Optional `.env` file support for local development
- Strongly-typed config models for server, database, and third-party integrations

## Table of Contents

- [Features](#features)
- [Getting Started](#getting-started)
- [Usage](#usage)
- [Configuration Files](#configuration-files)
- [Environment Variables](#environment-variables)
- [Config Models](#config-models)
- [Feature Flags](#feature-flags)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Layered Config**: Loads `config/default` then `config/{RUN_ENV}` if present
- **Env Prefix**: Supports environment variables prefixed with `PREFIX` or default `HTR`
- **Dotenv Support**: Auto-loads a `.env` file once per process
- **Serde-powered**: Deserialize into Rust structs via `serde`
- **Optional OpenAPI Schemas**: Derive `utoipa::ToSchema` when `openapi` feature enabled

## Getting Started

Add `connectify-config` to your `Cargo.toml`:
```toml
connectify-config = { path = "../connectify_config" }
```

Then enable the `openapi` feature if you need schemas:
```toml
[dependencies]
connectify-config = { path = "../connectify_config", features = ["openapi"] }
```

## Usage

In your `main.rs` or library:
```rust
use connectify_config::load_config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    println!("Loaded config: {:#?}", config);
    // Use config.server.host, config.server.port, etc.
    Ok(())
}
```

## Configuration Files

Place your YAML or other supported format in:

- `config/default.(yaml|toml|json)` (optional)
- `config/{RUN_ENV}.(yaml|toml|json)` (optional)

The default `RUN_ENV` is `debug` unless overridden via `RUN_ENV` env var.

## Environment Variables

Overrides can be provided via environment variables, using the `PREFIX` (default `HTR`).

- `HTR__SERVER__HOST` → `server.host`
- `HTR__SERVER__PORT` → `server.port`
- `HTR__USE_TWILIO` → `use_twilio`
- and so on for all `AppConfig` fields.

You can also point to a custom dotenv file by setting:

- `DOTENV_OVERRIDE=/path/to/.env`

## Config Models

| Struct             | Description                                 |
| ------------------ | ------------------------------------------- |
| `ServerConfig`     | HTTP server host and port                  |
| `DatabaseConfig`   | Database connection URL                    |
| `TwilioConfig`     | Twilio credentials (non-secret)            |
| `StripeConfig`     | Stripe payment settings (non-secret)       |
| `FulfillmentConfig`| Shared secret for fulfillment callbacks    |
| `PayrexxConfig`    | Payrexx integration settings               |
| `CalendlyConfig`   | Calendly OAuth settings                    |
| `GcalConfig`       | Google Calendar service account settings   |
| `AppConfig`        | Unified application configuration          |

## Feature Flags

- `openapi`: Derive `utoipa::ToSchema` on config models for OpenAPI documentation

## Examples

Example `config/default.yaml`:
```yaml
server:
  host: "127.0.0.1"
  port: 8000
use_twilio: false
```

Override via env:
```bash
PREFIX=MYAPP RUN_ENV=production DOTENV_OVERRIDE=.env.prod cargo run
```

## Contributing

Contributions welcome! Please open issues or PRs. Ensure code is formatted with `rustfmt` and tested.

## License

MIT © SwissAppGroup