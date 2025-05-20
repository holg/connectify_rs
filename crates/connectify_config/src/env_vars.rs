//! Environment variable handling for the Connectify application.
//!
//! This module provides utilities for working with environment variables in a
//! standardized way. It includes functions for getting environment variables
//! with consistent naming patterns and for converting between different naming
//! patterns.

use std::env;

/// The default prefix for configuration environment variables
pub const DEFAULT_PREFIX: &str = "CONNECTIFY";

/// The prefix for secret environment variables
pub const SECRET_PREFIX: &str = "CONNECTIFY_SECRET";

/// The separator for configuration environment variables
pub const CONFIG_SEPARATOR: &str = "__";

/// The separator for secret environment variables
pub const SECRET_SEPARATOR: &str = "_";

/// Get the prefix for configuration environment variables
pub fn get_config_prefix() -> String {
    env::var("PREFIX").unwrap_or_else(|_| DEFAULT_PREFIX.to_string())
}

/// Convert a configuration path to an environment variable name
///
/// # Arguments
///
/// * `path` - The configuration path (e.g., "server.host")
///
/// # Returns
///
/// The environment variable name (e.g., "CONNECTIFY__SERVER__HOST")
pub fn config_path_to_env_var(path: &str) -> String {
    let prefix = get_config_prefix();
    let path = path.replace('.', CONFIG_SEPARATOR);
    format!("{}{}{}", prefix, CONFIG_SEPARATOR, path).to_uppercase()
}

/// Convert a secret path to an environment variable name
///
/// # Arguments
///
/// * `path` - The secret path (e.g., "twilio.account_sid")
///
/// # Returns
///
/// The environment variable name (e.g., "CONNECTIFY_SECRET_TWILIO_ACCOUNT_SID")
pub fn secret_path_to_env_var(path: &str) -> String {
    let path = path.replace('.', SECRET_SEPARATOR);
    format!("{}{}{}", SECRET_PREFIX, SECRET_SEPARATOR, path).to_uppercase()
}

/// Convert a legacy secret path to an environment variable name
///
/// This function is used for backward compatibility with the old naming pattern.
///
/// # Arguments
///
/// * `path` - The secret path (e.g., "twilio.account_sid")
///
/// # Returns
///
/// The environment variable name (e.g., "TWILIO_ACCOUNT_SID")
pub fn legacy_secret_path_to_env_var(path: &str) -> String {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() < 2 {
        return path.to_uppercase();
    }

    let service = parts[0];
    let key = parts[1..].join(SECRET_SEPARATOR);
    format!("{}_{}", service, key).to_uppercase()
}

/// Get an environment variable for a configuration path
///
/// This function tries to get the environment variable using the new naming pattern.
/// If the variable is not found, it falls back to the old naming pattern.
///
/// # Arguments
///
/// * `path` - The configuration path (e.g., "server.host")
///
/// # Returns
///
/// The environment variable value, if found
pub fn get_config_env_var(path: &str) -> Option<String> {
    let env_var = config_path_to_env_var(path);
    env::var(&env_var).ok()
}

/// Get an environment variable for a secret path
///
/// This function tries to get the environment variable using the new naming pattern.
/// If the variable is not found, it falls back to the old naming pattern.
///
/// # Arguments
///
/// * `path` - The secret path (e.g., "twilio.account_sid")
///
/// # Returns
///
/// The environment variable value, if found
pub fn get_secret_env_var(path: &str) -> Option<String> {
    // Try the new naming pattern
    let env_var = secret_path_to_env_var(path);
    if let Ok(value) = env::var(&env_var) {
        return Some(value);
    }

    // Fall back to the old naming pattern
    let legacy_env_var = legacy_secret_path_to_env_var(path);
    env::var(&legacy_env_var).ok()
}

/// Check if a path is a secret path
///
/// This function checks if a path is a secret path based on its name.
/// Paths containing "secret", "key", "password", "token", or "sid" are considered secret.
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// `true` if the path is a secret path, `false` otherwise
pub fn is_secret_path(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    path_lower.contains("secret") ||
    path_lower.contains("key") ||
    path_lower.contains("password") ||
    path_lower.contains("token") ||
    path_lower.contains("sid")
}

/// Get an environment variable for a path
///
/// This function determines if the path is a secret path and calls the appropriate
/// function to get the environment variable.
///
/// # Arguments
///
/// * `path` - The path (e.g., "server.host" or "twilio.account_sid")
///
/// # Returns
///
/// The environment variable value, if found
pub fn get_env_var(path: &str) -> Option<String> {
    if is_secret_path(path) {
        get_secret_env_var(path)
    } else {
        get_config_env_var(path)
    }
}

/// Inject environment variables into a JSON value
///
/// This function recursively processes a JSON value, replacing "secret_from_env" strings
/// with values from environment variables using the standardized naming scheme.
///
/// # Arguments
///
/// * `value` - The JSON value to process
///
/// # Returns
///
/// `true` if any values were replaced, `false` otherwise
pub fn inject_env_vars(value: &mut serde_json::Value) -> bool {
    use serde_json::Value;

    fn walk(path: Vec<String>, obj: &mut Value) -> bool {
        let mut replaced = false;

        match obj {
            Value::Object(map) => {
                for (k, v) in map.iter_mut() {
                    let mut new_path = path.clone();
                    new_path.push(k.to_string());
                    replaced |= walk(new_path, v);
                }
            }
            Value::Array(arr) => {
                for (i, v) in arr.iter_mut().enumerate() {
                    let mut new_path = path.clone();
                    new_path.push(i.to_string());
                    replaced |= walk(new_path, v);
                }
            }
            Value::String(s) if s == "secret_from_env" => {
                let path_str = path.join(".");
                if let Some(env_val) = get_env_var(&path_str) {
                    *s = env_val;
                    replaced = true;
                } else {
                    eprintln!("Warning: env var for {} not found", path_str);
                }
            }
            _ => {}
        }

        replaced
    }

    walk(vec![], value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path_to_env_var() {
        assert_eq!(
            config_path_to_env_var("server.host"),
            "CONNECTIFY__SERVER__HOST"
        );
        assert_eq!(
            config_path_to_env_var("database.url"),
            "CONNECTIFY__DATABASE__URL"
        );
    }

    #[test]
    fn test_secret_path_to_env_var() {
        assert_eq!(
            secret_path_to_env_var("twilio.account_sid"),
            "CONNECTIFY_SECRET_TWILIO_ACCOUNT_SID"
        );
        assert_eq!(
            secret_path_to_env_var("stripe.secret_key"),
            "CONNECTIFY_SECRET_STRIPE_SECRET_KEY"
        );
    }

    #[test]
    fn test_legacy_secret_path_to_env_var() {
        assert_eq!(
            legacy_secret_path_to_env_var("twilio.account_sid"),
            "TWILIO_ACCOUNT_SID"
        );
        assert_eq!(
            legacy_secret_path_to_env_var("stripe.secret_key"),
            "STRIPE_SECRET_KEY"
        );
    }

    #[test]
    fn test_is_secret_path() {
        assert!(is_secret_path("twilio.account_sid"));
        assert!(is_secret_path("stripe.secret_key"));
        assert!(is_secret_path("gcal.client_secret"));
        assert!(is_secret_path("fulfillment.shared_secret"));
        assert!(!is_secret_path("server.host"));
        assert!(!is_secret_path("database.url"));
    }
}
