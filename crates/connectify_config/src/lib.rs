use config::ConfigError;
#[allow(dead_code)]
#[allow(non_upper_case_globals)]
#[allow(clippy::all)]
pub use connectify_config_static::{apply_env_overrides_from_marker,
                                   models::*, AppConfig, ensure_dotenv_loaded};
use serde_json;

include!(concat!(env!("OUT_DIR"), "/generated_config.rs"));
// pub use self::DEFAULT_CONFIG_JSON;

/// Loads the embedded static configuration.
/// This is used by dependent crates so they do not need to know whether the config is dynamic or static.
pub fn load_config() -> Result<AppConfig, ConfigError> {
    ensure_dotenv_loaded();
    let config: AppConfig = serde_json::from_str(DEFAULT_CONFIG_JSON)
        .map_err(|err| ConfigError::Message(format!("failed to parse embedded config: {err}")))?;
    Ok(apply_env_overrides_from_marker(config))
}
