use config::{Config, ConfigError, Environment, File};
use once_cell::sync::OnceCell;
use std::env;
use std::path::PathBuf;
use serde_json::{Value};
pub mod models;
use dotenv;
pub use models::*;

pub fn load_config() -> Result<AppConfig, ConfigError> {
    ensure_dotenv_loaded();

    let run_env = env::var("RUN_ENV").unwrap_or_else(|_| "debug".to_string());
    let prefix = env::var("PREFIX").unwrap_or_else(|_| "HTR".to_string());

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir
        .ancestors()
        .nth(2) // go from crates/connectify_config to workspace root
        .unwrap()
        .to_path_buf();

    let default_path = workspace_root.join("config/default");
    let env_path = workspace_root.join(format!("config/{}", run_env));

    eprintln!("build.rs: workspace_root: {}", workspace_root.display());
    eprintln!("build.rs: default_path: {}", default_path.display());
    eprintln!("build.rs: env_path: {}", env_path.display());

    let builder = Config::builder()
        .add_source(File::with_name(default_path.to_str().unwrap()).required(false))
        .add_source(File::with_name(env_path.to_str().unwrap()).required(false))
        .add_source(Environment::with_prefix(&prefix).separator("__"));

    eprintln!("build.rs: starting config load builder: {builder:?}");

    let raw_config: AppConfig = builder.build()?.try_deserialize()?;
    Ok(raw_config)
    // Ok(apply_env_overrides_from_marker(raw_config))
}

/// Recursively replaces all "secret_from_env" string values with environment variable values
fn inject_env_secrets(value: &mut Value) {
    fn walk(path: Vec<String>, obj: &mut Value) {
        match obj {
            Value::Object(map) => {
                for (k, v) in map.iter_mut() {
                    let mut new_path = path.clone();
                    new_path.push(k.to_string());
                    walk(new_path, v);
                }
            }
            Value::String(s) if s == "secret_from_env" => {
                let env_key = path.join("_").to_uppercase();
                if let Ok(env_val) = std::env::var(&env_key) {
                    *obj = Value::String(env_val);
                } else {
                    eprintln!("Warning: env var {} not found for secret_from_env", env_key);
                }
            }
            _ => {}
        }
    }

    walk(vec![], value);
}

/// Applies environment overrides based on "secret_from_env" markers in serialized config
pub fn apply_env_overrides_from_marker(config: AppConfig) -> AppConfig {
    let mut json = serde_json::to_value(&config).expect("AppConfig must be serializable");
    inject_env_secrets(&mut json);
    serde_json::from_value(json).expect("AppConfig must remain deserializable")
}

static INIT_DOTENV: OnceCell<()> = OnceCell::new();

/// Ensures that the dotenv file is loaded into the environment variables.
///
/// This function checks if the dotenv file has already been loaded using a `OnceCell`.
/// If not, it attempts to load the dotenv file specified by the first command line argument.
/// If no argument is provided, it defaults to loading a file named ".env".
///
/// # Parameters
///
/// This function does not take any parameters.
///
/// # Return
///
/// This function does not return any value. It ensures that the environment variables
/// from the dotenv file are loaded into the process's environment.
pub fn ensure_dotenv_loaded() -> String {
    let dotenv_path_override = std::env::var("DOTENV_OVERRIDE").ok();
    let dotenv_path_arg = env::args().nth(1).filter(|s| s.starts_with(".env"));

    let dotenv_path = dotenv_path_override
        .or(dotenv_path_arg)
        .unwrap_or_else(|| ".env".to_string());

    INIT_DOTENV.get_or_init(|| {
        dotenv::from_filename(&dotenv_path).ok();
    });

    dotenv_path
}
