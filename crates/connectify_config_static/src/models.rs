// --- File: crates/connectify_config/src/models.rs ---

use serde::{Deserialize, Serialize};

// #[cfg(feature = "openapi")]
// use utoipa::ToSchema; // , IntoParams};
// --- General Server Config ---
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

// --- Database Config ---
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String, // e.g., DATABASE_URL loaded via APP_DATABASE__URL or DATABASE_URL
}

// --- Twilio Config ---
// Holds non-secret Twilio config. Secrets loaded directly from env vars.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TwilioConfig {
    pub account_sid: String, // Loaded via TWILIO_ACCOUNT_SID
    pub api_key_sid: String, // Loaded via TWILIO_API_KEY_SID
    pub api_key_secret: String,
    // Secret loaded directly from env var: TWILIO_API_KEY_SECRET
}

#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PriceTier {
    /// Duration in minutes for this price tier.
    pub duration_minutes: i64,
    /// Price in the smallest currency unit (e.g., cents).
    pub unit_amount: i64,
    /// Optional product name specific to this tier.
    pub product_name: Option<String>,
    /// Optional currency code for this tier.
    pub currency: Option<String>,
    // You could add a Price ID here if you manage prices directly in Stripe Dashboard
    // pub price_id: Option<String>,
}

// --- Stripe Config ---
// Holds non-secret Stripe config. Secret key loaded directly from env var.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StripeConfig {
    pub success_url: String, // Mandatory
    pub cancel_url: String,  // Mandatory
    pub default_currency: Option<String>,
    pub unit_amount: Option<i64>,
    pub product_name: Option<String>,
    pub payment_success_url: String, // Mandatory
    /// List of price tiers for different durations.
    #[serde(default)] // Defaults to an empty vec if not present in config
    pub price_tiers: Vec<PriceTier>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FulfillmentConfig {
    pub shared_secret: Option<String>    // Secret key loaded directly from env var: FULFIL_SHARED_SECRET
}

// --- Payrexx Config ---
// Holds non-secret Payrexx config. Secret key loaded directly from env var.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PayrexxConfig {
    pub instance_name: String, // Mandatory
    pub success_url: String,   // Mandatory
    pub failed_url: String,    // Mandatory
    pub cancel_url: String,    // Mandatory
    pub currency: Option<String>,
    pub unit_amount: Option<i64>,
    pub product_name: Option<String>,
    /// List of price tiers for different durations.
    #[serde(default)] // Defaults to an empty vec if not present in config
    pub price_tiers: Vec<PriceTier>,
    // API Secret loaded directly from env var: PAYREXX_API_SECRET
}

// --- Calendly Config ---
// Holds non-secret Calendly config. Secrets/Keys loaded directly from env vars.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CalendlyConfig {
    pub client_id: String, // Mandatory
    pub redirect_uri: String, // Mandatory
                           // Secrets loaded directly from env vars:
                           // CALENDLY_CLIENT_SECRET
                           // CSRF_STATE_SECRET
                           // ENCRYPTION_KEY
}

// --- Google Calendar Config ---
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GcalConfig {
    // pub api_key: String, // Mandatory
    // pub key_path: Option<String>,
    // pub calendar_id: String, // Mandatory
    pub key_path: Option<String>, // Mandatory
    pub calendar_id: Option<String>, // Mandatory
    pub time_slot_duration: Option<u16>, // In minutes                                  
}

// --- Unified App Configuration ---
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    // Server config is mandatory
    pub server: ServerConfig,

    // --- Runtime Flags (optional in config file, default to false) ---
    #[serde(default)]
    pub use_twilio: bool,
    #[serde(default)]
    pub use_stripe: bool,
    #[serde(default)]
    pub use_gcal: bool,
    #[serde(default)]
    pub use_fulfillment: bool,
    #[serde(default)]
    pub use_payrexx: bool,
    #[serde(default)]
    pub use_calendly: bool,

    // --- Optional Feature Configurations ---
    #[serde(default)]
    pub database: Option<DatabaseConfig>, // Central DB config
    #[serde(default)]
    pub twilio: Option<TwilioConfig>,
    #[serde(default)]
    pub stripe: Option<StripeConfig>,
    #[serde(default)]
    pub fulfillment: Option<FulfillmentConfig>,
    #[serde(default)]
    pub payrexx: Option<PayrexxConfig>,
    #[serde(default)]
    pub gcal: Option<GcalConfig>,
}
