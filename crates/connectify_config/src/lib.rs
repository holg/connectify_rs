use config::{Config, ConfigError, Environment, File};
use once_cell::sync::OnceCell;
use std::env;
mod models;
use dotenv;
pub use models::*;
pub fn load_config() -> Result<AppConfig, ConfigError> {
    ensure_dotenv_loaded();

    let run_env = env::var("RUN_ENV").unwrap_or_else(|_| "debug".to_string());
    let prefix = env::var("PREFIX").unwrap_or_else(|_| "HTR".to_string());
    let builder = Config::builder()
        .add_source(File::with_name("config/default").required(false))
        .add_source(File::with_name(&format!("config/{}", run_env)).required(false))
        .add_source(Environment::with_prefix(&prefix).separator("__"));

    builder.build()?.try_deserialize()
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
