//! Feature flag handling for the Connectify application.
//!
//! This module provides utilities for working with feature flags in a more
//! maintainable way. It includes documentation on what each feature does and
//! helper functions for checking if features are enabled.
//!
//! ## Available Features
//!
//! - `openapi`: Enables OpenAPI documentation generation
//! - `gcal`: Enables Google Calendar integration
//! - `stripe`: Enables Stripe payment processing
//! - `twilio`: Enables Twilio notifications
//! - `payrexx`: Enables Payrexx payment processing
//! - `fulfillment`: Enables fulfillment functionality
//! - `gcal_fulfillment`: Enables both Google Calendar and fulfillment functionality
//!
//! ## Usage
//!
//! Feature flags are used in two ways in the Connectify application:
//!
//! 1. Compile-time feature flags using `#[cfg(feature = "...")]`
//! 2. Runtime feature flags using configuration values
//!
//! This module provides helper functions for checking if features are enabled
//! at runtime based on configuration values.

use connectify_config::AppConfig;
use std::sync::Arc;

/// Check if a feature is enabled at runtime based on configuration.
///
/// This function checks if a feature is enabled based on the configuration
/// values. It's used to conditionally enable features at runtime.
///
/// # Arguments
///
/// * `config` - The application configuration
/// * `use_feature` - The configuration flag that enables the feature
/// * `feature_config` - The configuration section for the feature
///
/// # Returns
///
/// `true` if the feature is enabled, `false` otherwise
pub fn is_feature_enabled<T>(
    _config: &Arc<AppConfig>,
    use_feature: bool,
    feature_config: Option<&T>,
) -> bool {
    use_feature && feature_config.is_some()
}

/// Check if the Google Calendar feature is enabled at runtime.
///
/// # Arguments
///
/// * `config` - The application configuration
///
/// # Returns
///
/// `true` if the Google Calendar feature is enabled, `false` otherwise
#[cfg(feature = "gcal")]
pub fn is_gcal_enabled(config: &Arc<AppConfig>) -> bool {
    is_feature_enabled(config, config.use_gcal, config.gcal.as_ref())
}

/// Check if the Stripe feature is enabled at runtime.
///
/// # Arguments
///
/// * `config` - The application configuration
///
/// # Returns
///
/// `true` if the Stripe feature is enabled, `false` otherwise
#[cfg(feature = "stripe")]
pub fn is_stripe_enabled(config: &Arc<AppConfig>) -> bool {
    is_feature_enabled(config, config.use_stripe, config.stripe.as_ref())
}

/// Check if the Twilio feature is enabled at runtime.
///
/// # Arguments
///
/// * `config` - The application configuration
///
/// # Returns
///
/// `true` if the Twilio feature is enabled, `false` otherwise
#[cfg(feature = "twilio")]
pub fn is_twilio_enabled(config: &Arc<AppConfig>) -> bool {
    is_feature_enabled(config, config.use_twilio, config.twilio.as_ref())
}

/// Check if the Payrexx feature is enabled at runtime.
///
/// # Arguments
///
/// * `config` - The application configuration
///
/// # Returns
///
/// `true` if the Payrexx feature is enabled, `false` otherwise
#[cfg(feature = "payrexx")]
pub fn is_payrexx_enabled(config: &Arc<AppConfig>) -> bool {
    is_feature_enabled(config, config.use_payrexx, config.payrexx.as_ref())
}

/// Check if the Fulfillment feature is enabled at runtime.
///
/// # Arguments
///
/// * `config` - The application configuration
///
/// # Returns
///
/// `true` if the Fulfillment feature is enabled, `false` otherwise
#[cfg(feature = "fulfillment")]
pub fn is_fulfillment_enabled(config: &Arc<AppConfig>) -> bool {
    is_feature_enabled(config, config.use_fulfillment, config.fulfillment.as_ref())
}

#[cfg(feature = "adhoc")]
pub fn is_adhoc_enabled(config: &Arc<AppConfig>) -> bool {
    is_feature_enabled(
        config,
        config.use_adhoc_session,
        config.use_adhoc_session.as_ref(),
    )
}
