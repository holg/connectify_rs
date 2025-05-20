use config::ConfigError;
#[allow(dead_code)]
#[allow(non_upper_case_globals)]
#[allow(clippy::all)]
pub use connectify_config_static::{
    apply_env_overrides_from_marker, ensure_dotenv_loaded, models::*, AppConfig,
};
use serde_json;
use std::fmt;

// Secret management module
pub mod secrets;

// Environment variable handling module
pub mod env_vars;

include!(concat!(env!("OUT_DIR"), "/generated_config.rs"));
// pub use self::DEFAULT_CONFIG_JSON;

/// Custom error type for configuration errors with improved context
#[derive(Debug)]
pub enum ConfigurationError {
    /// Error parsing the embedded configuration
    ParseError(String),
    /// Error applying environment overrides
    EnvOverrideError(String),
    /// Error validating configuration values
    ValidationError(String),
    /// Error decrypting configuration values
    DecryptionError(String),
    /// Underlying config error
    ConfigError(ConfigError),
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigurationError::ParseError(msg) => write!(f, "Configuration parse error: {}", msg),
            ConfigurationError::EnvOverrideError(msg) => {
                write!(f, "Environment override error: {}", msg)
            }
            ConfigurationError::ValidationError(msg) => {
                write!(f, "Configuration validation error: {}", msg)
            }
            ConfigurationError::DecryptionError(msg) => {
                write!(f, "Configuration decryption error: {}", msg)
            }
            ConfigurationError::ConfigError(err) => write!(f, "Configuration error: {}", err),
        }
    }
}

impl std::error::Error for ConfigurationError {}

impl From<ConfigError> for ConfigurationError {
    fn from(err: ConfigError) -> Self {
        ConfigurationError::ConfigError(err)
    }
}

impl From<serde_json::Error> for ConfigurationError {
    fn from(err: serde_json::Error) -> Self {
        ConfigurationError::ParseError(format!(
            "{} at line {}, column {}",
            err.to_string(),
            err.line(),
            err.column()
        ))
    }
}

/// Loads the embedded static configuration.
/// This is used by dependent crates so they do not need to know whether the config is dynamic or static.
pub fn load_config() -> Result<AppConfig, ConfigurationError> {
    // Ensure .env file is loaded
    // We don't use the return value, but it's important to call this function
    // to ensure that environment variables from the .env file are loaded
    let _dotenv_path = ensure_dotenv_loaded();

    // Parse the embedded configuration
    let config: AppConfig = serde_json::from_str(DEFAULT_CONFIG_JSON)
        .map_err(|err| {
            ConfigurationError::ParseError(format!(
                "Failed to parse embedded configuration: {} at line {}, column {}. This is likely a bug in the build script.",
                err.to_string(), err.line(), err.column()
            ))
        })?;

    // Apply environment overrides
    let config = apply_env_overrides(config)?;

    // Decrypt any encrypted values
    let config = decrypt_config(config)?;

    // Validate the configuration
    validate_config(&config)?;

    Ok(config)
}

/// Applies environment overrides to the configuration
fn apply_env_overrides(config: AppConfig) -> Result<AppConfig, ConfigurationError> {
    // Convert the config to JSON for processing
    let mut json_value = serde_json::to_value(&config).map_err(|err| {
        ConfigurationError::ParseError(format!("Failed to serialize config to JSON: {}", err))
    })?;

    // Process the JSON value, replacing "secret_from_env" strings with values from environment variables
    env_vars::inject_env_vars(&mut json_value);

    // Convert back to AppConfig
    let config_with_env_vars = serde_json::from_value(json_value).map_err(|err| {
        ConfigurationError::ParseError(format!(
            "Failed to deserialize config with env vars: {}",
            err
        ))
    })?;

    Ok(config_with_env_vars)
}

/// Decrypts any encrypted values in the configuration
fn decrypt_config(config: AppConfig) -> Result<AppConfig, ConfigurationError> {
    // Convert the config to JSON for processing
    let mut json_value = serde_json::to_value(&config).map_err(|err| {
        ConfigurationError::ParseError(format!("Failed to serialize config to JSON: {}", err))
    })?;

    // Process the JSON value, decrypting all encrypted strings
    secrets::process_json_for_decryption(&mut json_value).map_err(|err| {
        ConfigurationError::DecryptionError(format!("Failed to decrypt config: {}", err))
    })?;

    // Convert back to AppConfig
    let decrypted_config = serde_json::from_value(json_value).map_err(|err| {
        ConfigurationError::ParseError(format!("Failed to deserialize decrypted config: {}", err))
    })?;

    Ok(decrypted_config)
}

/// Validates the configuration values and returns meaningful error messages
fn validate_config(config: &AppConfig) -> Result<(), ConfigurationError> {
    // Validate server configuration
    if config.server.port == 0 {
        return Err(ConfigurationError::ValidationError(
            "Server port cannot be 0".to_string(),
        ));
    }

    // Validate feature-specific configurations if the feature is enabled
    if config.use_twilio && config.twilio.is_none() {
        return Err(ConfigurationError::ValidationError(
            "Twilio is enabled but no Twilio configuration is provided".to_string(),
        ));
    }

    if config.use_stripe && config.stripe.is_none() {
        return Err(ConfigurationError::ValidationError(
            "Stripe is enabled but no Stripe configuration is provided".to_string(),
        ));
    }

    if config.use_gcal && config.gcal.is_none() {
        return Err(ConfigurationError::ValidationError(
            "Google Calendar is enabled but no GCal configuration is provided".to_string(),
        ));
    }

    if config.use_payrexx && config.payrexx.is_none() {
        return Err(ConfigurationError::ValidationError(
            "Payrexx is enabled but no Payrexx configuration is provided".to_string(),
        ));
    }

    if config.use_fulfillment && config.fulfillment.is_none() {
        return Err(ConfigurationError::ValidationError(
            "Fulfillment is enabled but no Fulfillment configuration is provided".to_string(),
        ));
    }

    if config.use_adhoc && config.adhoc_settings.is_none() {
        return Err(ConfigurationError::ValidationError(
            "Fulfillment is enabled but no Fulfillment configuration is provided".to_string(),
        ));
    }

    // Validate Stripe configuration if present
    if let Some(stripe_config) = &config.stripe {
        if stripe_config.success_url.is_empty() {
            return Err(ConfigurationError::ValidationError(
                "Stripe success_url cannot be empty".to_string(),
            ));
        }

        if stripe_config.cancel_url.is_empty() {
            return Err(ConfigurationError::ValidationError(
                "Stripe cancel_url cannot be empty".to_string(),
            ));
        }

        if stripe_config.payment_success_url.is_empty() {
            return Err(ConfigurationError::ValidationError(
                "Stripe payment_success_url cannot be empty".to_string(),
            ));
        }
    }

    // Validate Payrexx configuration if present
    if let Some(payrexx_config) = &config.payrexx {
        if payrexx_config.instance_name.is_empty() {
            return Err(ConfigurationError::ValidationError(
                "Payrexx instance_name cannot be empty".to_string(),
            ));
        }

        if payrexx_config.success_url.is_empty() {
            return Err(ConfigurationError::ValidationError(
                "Payrexx success_url cannot be empty".to_string(),
            ));
        }

        if payrexx_config.failed_url.is_empty() {
            return Err(ConfigurationError::ValidationError(
                "Payrexx failed_url cannot be empty".to_string(),
            ));
        }

        if payrexx_config.cancel_url.is_empty() {
            return Err(ConfigurationError::ValidationError(
                "Payrexx cancel_url cannot be empty".to_string(),
            ));
        }
    }

    Ok(())
}
