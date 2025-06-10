//! Database integration for Connectify
//!
//! This crate provides a database client that is designed to be database agnostic,
//! using SQLx as the underlying database library. It supports SQLite, PostgreSQL,
//! and MySQL databases through feature flags.
//!
//! # Features
//!
//! - Database agnostic design
//! - Connection pooling
//! - Integration with the Connectify configuration system
//! - Support for SQLite, PostgreSQL, and MySQL
//!
//! # Usage
//!
//! Add the crate to your dependencies:
//!
//! ```toml
//! [dependencies]
//! connectify-db = { version = "0.1.0" }
//! ```
//!
//! To use a specific database backend:
//!
//! ```toml
//! [dependencies]
//! connectify-db = { version = "0.1.0", features = ["postgres"] }
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use connectify_config::AppConfig;
//! use connectify_db::DbClient;
//! use std::sync::Arc;
//!
//! async fn setup_db() -> Result<DbClient, Box<dyn std::error::Error>> {
//!     let config = Arc::new(AppConfig::default());
//!     let db_client = DbClient::new(&config).await?;
//!     Ok(db_client)
//! }
//! ```

pub mod client;
pub mod error;
pub mod factory;
pub mod repositories;
pub mod repository;

// Register the SQLite driver when the crate is loaded
#[cfg(feature = "sqlite")]
mod sqlite_driver {
    // This import ensures the SQLite driver is linked and registered
    #[allow(unused_imports)]
    use sqlx::sqlite::SqlitePoolOptions as _;
}

// Re-export the client, factory, and repository traits for ease of use
pub use client::DbClient;
pub use factory::DbClientFactory;
pub use repository::{Repository, RepositoryFactory};

// Re-export the repositories module components for ease of use
pub use repositories::{
    DeviceRegistration, DeviceRegistrationRepository, DeviceRegistrationRepositoryFactory,
    SqlDeviceRegistrationRepository,
};
