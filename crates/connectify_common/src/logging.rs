// src/logging.rs
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

pub fn resolve_log_dir(app_name: &str) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".into()))
            .join("Library")
            .join("Logs")
            .join(app_name)
    }
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(env::var("APPDATA").unwrap_or_else(|_| ".".into()))
            .join("AppLogs")
            .join(app_name);
    }
    #[cfg(target_os = "linux")]
    {
        let system_path = PathBuf::from(format!("/var/log/{}", app_name));
        if system_path.exists()
            && system_path.is_dir()
            && fs::metadata(&system_path)
                .map(|m| !m.permissions().readonly())
                .unwrap_or(false)
        {
            return system_path;
        }
        PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".into()))
            .join(format!(".{}", app_name))
    }
}

pub fn init_logging(app_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = resolve_log_dir(app_name);
    fs::create_dir_all(&log_dir)?;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let file_appender = rolling::daily(&log_dir, format!("{}.log", app_name));
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true);

    let console_layer = fmt::layer().with_ansi(true).with_target(true);

    {
        let s = Registry::default()
            .with(filter)
            .with(file_layer)
            .with(console_layer);
        #[cfg(target_os = "linux")]
        {
            if let Ok(journald) = tracing_journald::layer() {
                return tracing::subscriber::set_global_default(s.with(journald))
                    .map_err(Box::<dyn std::error::Error>::from);
            }
        }
        tracing::subscriber::set_global_default(s).map_err(Box::<dyn std::error::Error>::from)
    }?;
    Ok(())
}
