

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
- **Env Prefix**: Supports environment variables prefixed with `PREFIX` or default `CONNECTIFY`
- **Dotenv Support**: Auto-loads a `.env` file once per process
- **Serde-powered**: Deserialize into Rust structs via `serde`
- **Optional OpenAPI Schemas**: Derive `utoipa::ToSchema` when `openapi` feature enabled
- **Secret Management**: Encrypts sensitive values in configuration files
- **Hot-Reloading**: Watches for changes in configuration files and restarts the application
- **Configuration Migration**: Helps migrate configuration files from one format to another

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

Connectify uses a standardized naming scheme for environment variables to ensure consistency across all services. There are two types of environment variables:

1. **Configuration Variables**: Used for regular configuration options
2. **Secret Variables**: Used for sensitive information like API keys and passwords

### Configuration Variables

Configuration variables use the following naming pattern:

```
CONNECTIFY__<SECTION>__<KEY>
```

For example:
- `CONNECTIFY__SERVER__HOST` → `server.host`
- `CONNECTIFY__SERVER__PORT` → `server.port`
- `CONNECTIFY__USE_TWILIO` → `use_twilio`

The prefix `CONNECTIFY` can be customized by setting the `PREFIX` environment variable.

### Secret Variables

Secret variables use the following naming pattern:

```
CONNECTIFY_SECRET_<SECTION>_<KEY>
```

For example:
- `CONNECTIFY_SECRET_TWILIO_ACCOUNT_SID` → `twilio.account_sid`
- `CONNECTIFY_SECRET_STRIPE_SECRET_KEY` → `stripe.secret_key`
- `CONNECTIFY_SECRET_GCAL_CLIENT_SECRET` → `gcal.client_secret`

For backward compatibility, the old naming scheme is still supported. See the [Environment Variables documentation](../../docs/ENVIRONMENT_VARIABLES.md) for more details.

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

### Detailed Configuration Options

#### Server Configuration

The server configuration controls the HTTP server settings.

| Option | Type | Description | Default | Environment Variable |
|--------|------|-------------|---------|---------------------|
| `server.host` | String | The host address to bind to | `"127.0.0.1"` | `HTR__SERVER__HOST` |
| `server.port` | Integer | The port to listen on | `8086` | `HTR__SERVER__PORT` |

#### Feature Flags

Feature flags control which features are enabled in the application.

| Option | Type | Description | Default | Environment Variable |
|--------|------|-------------|---------|---------------------|
| `use_twilio` | Boolean | Enable Twilio integration | `true` | `HTR__USE_TWILIO` |
| `use_stripe` | Boolean | Enable Stripe integration | `true` | `HTR__USE_STRIPE` |
| `use_payrexx` | Boolean | Enable Payrexx integration | `true` | `HTR__USE_PAYREXX` |
| `use_gcal` | Boolean | Enable Google Calendar integration | `false` | `HTR__USE_GCAL` |
| `use_fulfillment` | Boolean | Enable fulfillment service | `true` | `HTR__USE_FULFILLMENT` |
| `use_calendly` | Boolean | Enable Calendly integration | `false` | `HTR__USE_CALENDLY` |

#### Database Configuration

The database configuration controls the database connection.

| Option | Type | Description | Default | Environment Variable |
|--------|------|-------------|---------|---------------------|
| `database.url` | String | The database connection URL | `sqlite://example.db` | `HTR__DATABASE__URL` or `DATABASE_URL` |

#### Twilio Configuration

The Twilio configuration controls the Twilio integration for notifications.

| Option | Type | Description | Default | Environment Variable |
|--------|------|-------------|---------|---------------------|
| `twilio.account_sid` | String | Twilio account SID | `secret_from_env` | `TWILIO_ACCOUNT_SID` |
| `twilio.api_key_sid` | String | Twilio API key SID | `secret_from_env` | `TWILIO_API_KEY_SID` |
| `twilio.api_key_secret` | String | Twilio API key secret | `secret_from_env` | `TWILIO_API_KEY_SECRET` |
| `twilio.verify_service_sid` | String | Twilio Verify service SID | `secret_from_env` | `TWILIO_VERIFY_SERVICE_SID` |

#### Stripe Configuration

The Stripe configuration controls the Stripe integration for payments.

| Option | Type | Description | Default | Environment Variable |
|--------|------|-------------|---------|---------------------|
| `stripe.secret_key` | String | Stripe secret key | `secret_from_env` | `STRIPE_SECRET_KEY` |
| `stripe.success_url` | String | URL to redirect to after successful payment | `"https://example.com/api/stripe/success"` | `HTR__STRIPE__SUCCESS_URL` |
| `stripe.cancel_url` | String | URL to redirect to after cancelled payment | `"https://example.com/api/stripe/cancel"` | `HTR__STRIPE__CANCEL_URL` |
| `stripe.payment_success_url` | String | URL to redirect to after payment success | `"https://example.com/payment-success.html"` | `HTR__STRIPE__PAYMENT_SUCCESS_URL` |
| `stripe.default_currency` | String | Default currency for payments | `"CHF"` | `HTR__STRIPE__DEFAULT_CURRENCY` |
| `stripe.price_tiers` | Array | List of price tiers for different durations | See example | N/A |

#### Payrexx Configuration

The Payrexx configuration controls the Payrexx integration for payments.

| Option | Type | Description | Default | Environment Variable |
|--------|------|-------------|---------|---------------------|
| `payrexx.api_key` | String | Payrexx API key | `secret_from_env` | `PAYREXX_API_KEY` |
| `payrexx.secret_key` | String | Payrexx secret key | `secret_from_env` | `PAYREXX_SECRET_KEY` |
| `payrexx.instance_name` | String | Payrexx instance name | `"My Payrexx Instance"` | `HTR__PAYREXX__INSTANCE_NAME` |
| `payrexx.success_url` | String | URL to redirect to after successful payment | `"https://example.com/api/payrexx/success"` | `HTR__PAYREXX__SUCCESS_URL` |
| `payrexx.failed_url` | String | URL to redirect to after failed payment | `"https://example.com/api/payrexx/failed"` | `HTR__PAYREXX__FAILED_URL` |
| `payrexx.cancel_url` | String | URL to redirect to after cancelled payment | `"https://example.com/api/payrexx/cancel"` | `HTR__PAYREXX__CANCEL_URL` |
| `payrexx.currency` | String | Default currency for payments | `"EUR"` | `HTR__PAYREXX__CURRENCY` |

#### Google Calendar Configuration

The Google Calendar configuration controls the Google Calendar integration.

| Option | Type | Description | Default | Environment Variable |
|--------|------|-------------|---------|---------------------|
| `gcal.client_id` | String | Google API client ID | `secret_from_env` | `GCAL_CLIENT_ID` |
| `gcal.client_secret` | String | Google API client secret | `secret_from_env` | `GCAL_CLIENT_SECRET` |
| `gcal.refresh_token` | String | Google API refresh token | `secret_from_env` | `GCAL_REFRESH_TOKEN` |
| `gcal.key_path` | String | Path to service account key file | `"./service_account_key.json"` | `HTR__GCAL__KEY_PATH` |
| `gcal.calendar_id` | String | ID of the calendar to use | `null` | `HTR__GCAL__CALENDAR_ID` |
| `gcal.time_slot_duration` | Integer | Duration of time slots in minutes | `null` | `HTR__GCAL__TIME_SLOT_DURATION` |

#### Calendly Configuration

The Calendly configuration controls the Calendly integration.

| Option | Type | Description | Default | Environment Variable |
|--------|------|-------------|---------|---------------------|
| `calendly.client_id` | String | Calendly client ID | `secret_from_env` | `CALENDLY_CLIENT_ID` |
| `calendly.client_secret` | String | Calendly client secret | `secret_from_env` | `CALENDLY_CLIENT_SECRET` |
| `calendly.refresh_token` | String | Calendly refresh token | `secret_from_env` | `CALENDLY_REFRESH_TOKEN` |
| `calendly.redirect_uri` | String | Redirect URI for OAuth | `"https://example.com/api/calendly/callback"` | `HTR__CALENDLY__REDIRECT_URI` |

#### Fulfillment Configuration

The Fulfillment configuration controls the fulfillment service.

| Option | Type | Description | Default | Environment Variable |
|--------|------|-------------|---------|---------------------|
| `fulfillment.shared_secret` | String | Shared secret for fulfillment service | `secret_from_env` | `FULFILLMENT_SHARED_SECRET` |

## Feature Flags

- `openapi`: Derive `utoipa::ToSchema` on config models for OpenAPI documentation

## Tools

Connectify provides several tools to help you manage your configuration:

### Configuration Hot-Reloading

The `config_watcher` tool watches for changes in configuration files and restarts the application when they change:

```bash
cargo run --bin config_watcher -- cargo run --bin connectify-backend
```

This is useful during development when you want to test configuration changes without manually restarting the application.

### Secret Encryption

The `encrypt_config` tool encrypts sensitive values in configuration files:

```bash
cargo run --bin encrypt_config -- config/production.yml
```

This tool identifies sensitive values based on their names (e.g., "secret", "key", "password") and encrypts them using AES-GCM. The encryption key is stored in a file named `.connectify_key` by default, but you can also set it using the `CONNECTIFY_ENCRYPTION_KEY` environment variable.

### Configuration Migration

The `migrate_config` tool helps you migrate configuration files from one format to another:

```bash
cargo run --bin migrate_config -- config/old_config.yml config/new_config.yml
```

This tool reads the source file, merges it with the target file (if it exists), and writes the result to the target file. It supports YAML, JSON, and TOML formats.

## Examples

The repository includes several example configuration files:

- `config/default.yml`: Default configuration used as a base
- `config/development.yml`: Configuration for local development
- `config/testing.yml`: Configuration for automated testing
- `config/production.yml`: Configuration for production deployment

### Example: Default Configuration

```yaml
server:
  host: "127.0.0.1"
  port: 8086
use_twilio: true
use_stripe: true
use_payrexx: true
use_fulfillment: true
```

### Example: Development Configuration

The development configuration is optimized for local development:

```yaml
server:
  host: "127.0.0.1"
  port: 8086

# Feature flags
use_twilio: true
use_stripe: true
use_payrexx: false
use_gcal: true
use_fulfillment: true
use_calendly: false

# Database configuration
database:
  url: "sqlite://development.db"
```

### Example: Testing Configuration

The testing configuration is designed for automated tests:

```yaml
server:
  host: "127.0.0.1"
  port: 8087

# Feature flags
use_twilio: true
use_stripe: true
use_payrexx: false
use_gcal: true
use_fulfillment: true
use_calendly: false

# Database configuration
database:
  url: "sqlite::memory:"
```

### Environment Variable Overrides

You can override any configuration value using environment variables:

```bash
PREFIX=MYAPP RUN_ENV=production DOTENV_OVERRIDE=.env.prod cargo run
```

This will:
1. Use `MYAPP` as the prefix for environment variables
2. Load configuration from `config/production.yml`
3. Load environment variables from `.env.prod`

## Contributing

Contributions welcome! Please open issues or PRs. Ensure code is formatted with `rustfmt` and tested.

## License

MIT © SwissAppGroup
