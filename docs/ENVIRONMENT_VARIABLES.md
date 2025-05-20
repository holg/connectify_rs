# Environment Variables in Connectify

This document describes the environment variable naming scheme used in Connectify and how to migrate from the old naming scheme to the new one.

## Naming Scheme

Connectify uses a standardized naming scheme for environment variables to ensure consistency across all services. There are two types of environment variables:

1. **Configuration Variables**: Used for regular configuration options
2. **Secret Variables**: Used for sensitive information like API keys and passwords

### Configuration Variables

Configuration variables use the following naming pattern:

```
CONNECTIFY__<SECTION>__<KEY>
```

For example:
- `CONNECTIFY__SERVER__HOST` for `server.host`
- `CONNECTIFY__DATABASE__URL` for `database.url`
- `CONNECTIFY__USE_TWILIO` for `use_twilio`

The prefix `CONNECTIFY` can be customized by setting the `PREFIX` environment variable.

### Secret Variables

Secret variables use the following naming pattern:

```
CONNECTIFY_SECRET_<SECTION>_<KEY>
```

For example:
- `CONNECTIFY_SECRET_TWILIO_ACCOUNT_SID` for `twilio.account_sid`
- `CONNECTIFY_SECRET_STRIPE_SECRET_KEY` for `stripe.secret_key`
- `CONNECTIFY_SECRET_GCAL_CLIENT_SECRET` for `gcal.client_secret`

## Backward Compatibility

For backward compatibility, the old naming scheme is still supported. The old naming scheme uses service-specific prefixes for secrets:

- `TWILIO_ACCOUNT_SID` for `twilio.account_sid`
- `STRIPE_SECRET_KEY` for `stripe.secret_key`
- `GCAL_CLIENT_SECRET` for `gcal.client_secret`

When looking up an environment variable, Connectify first tries the new naming scheme, and if the variable is not found, it falls back to the old naming scheme. This allows you to migrate to the new naming scheme gradually.

## Migration Guide

To migrate from the old naming scheme to the new one:

1. Identify all environment variables used in your application
2. For each variable, determine if it's a configuration variable or a secret variable
3. Rename the variable according to the new naming scheme
4. Update your deployment scripts, CI/CD pipelines, and documentation to use the new names

For example, if you're using the following environment variables:

```
HTR__SERVER__HOST=127.0.0.1
HTR__SERVER__PORT=8086
TWILIO_ACCOUNT_SID=your_account_sid
STRIPE_SECRET_KEY=your_secret_key
```

You would rename them to:

```
CONNECTIFY__SERVER__HOST=127.0.0.1
CONNECTIFY__SERVER__PORT=8086
CONNECTIFY_SECRET_TWILIO_ACCOUNT_SID=your_account_sid
CONNECTIFY_SECRET_STRIPE_SECRET_KEY=your_secret_key
```

## Tools

Connectify provides several tools to help you work with environment variables:

### Environment Variable Lookup

The `env_vars` module in the `connectify_config` crate provides functions for looking up environment variables using the standardized naming scheme:

```
use connectify_config::env_vars;

// Get a configuration variable
let host = env_vars::get_config_env_var("server.host");

// Get a secret variable
let account_sid = env_vars::get_secret_env_var("twilio.account_sid");

// Get any variable (automatically determines if it's a configuration or secret)
let value = env_vars::get_env_var("some.path");
```

### Configuration Hot-Reloading

The `config_watcher` tool watches for changes in configuration files and restarts the application when they change:

```bash
cargo run --bin config_watcher -- cargo run --bin connectify-backend
```

### Configuration Migration

The `migrate_config` tool helps you migrate configuration files from one format to another:

```bash
cargo run --bin migrate_config -- config/old_config.yml config/new_config.yml
```

### Secret Encryption

The `encrypt_config` tool encrypts sensitive values in configuration files:

```bash
cargo run --bin encrypt_config -- config/production.yml
```

## Best Practices

1. **Use the new naming scheme** for all new environment variables
2. **Migrate existing environment variables** to the new naming scheme when convenient
3. **Use the `env_vars` module** for looking up environment variables
4. **Encrypt sensitive values** in configuration files
5. **Use the configuration hot-reloading tool** during development
6. **Use the configuration migration tool** when upgrading to a new version
