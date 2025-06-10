# Connectify DB

A database integration library for the Connectify platform.

## Overview

Connectify DB provides a database abstraction layer for the Connectify platform. It supports multiple database backends through SQLx, including SQLite, PostgreSQL, and MySQL.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
connectify-db = { path = "../connectify_db" }
```

## Important: Database Drivers

**You must explicitly include at least one database driver in your dependencies!**

The error message:
```
No drivers installed. Please see the documentation in `sqlx::any` for details.
```

occurs when you're using the `sqlx::any` module without any database drivers installed. To fix this, you need to enable at least one of the following features in your project:

- `sqlite` - SQLite support
- `postgres` - PostgreSQL support
- `mysql` - MySQL support

For example, to enable SQLite support:

```toml
[dependencies]
connectify-db = { path = "../connectify_db", features = ["sqlite"] }
```

Or to enable multiple drivers:

```toml
[dependencies]
connectify-db = { path = "../connectify_db", features = ["sqlite", "postgres"] }
```

## Features

The crate provides the following features:

- `default = ["sqlite"]` - Enables SQLite support by default
- `sqlite` - Enables SQLite support
- `postgres` - Enables PostgreSQL support
- `mysql` - Enables MySQL support
- `openapi` - Enables OpenAPI documentation

## Usage

### Creating a Database Client

```rust
use connectify_db::{DbClient, DbClientFactory};
use connectify_config::DatabaseConfig;

async fn create_client() -> Result<DbClient, connectify_db::error::DbError> {
    // Create a database client factory
    let factory = DbClientFactory::new();
    
    // Create a database client from a URL
    let client = factory.from_url("sqlite::memory:").await?;
    
    // Or create a client from a configuration
    let config = DatabaseConfig {
        url: Some("sqlite::memory:".to_string()),
        max_connections: Some(5),
        ..Default::default()
    };
    let client = factory.from_db_config(&config).await?;
    
    Ok(client)
}
```

### Using Repositories

```rust
use connectify_db::{
    DbClient,
    repositories::device_registration_factory::DeviceRegistrationRepositoryFactory,
    RepositoryFactory,
};

async fn use_repository(db_client: DbClient) {
    // Create a repository factory
    let factory = DeviceRegistrationRepositoryFactory::new();
    
    // Create a repository
    let repository = factory.create_repository(db_client);
    
    // Initialize the schema
    repository.init_schema().await.expect("Failed to initialize schema");
    
    // Use the repository
    let registration = connectify_common::models::DeviceRegistration::new(
        "user123".to_string(),
        "device456".to_string(),
        "token789".to_string(),
    );
    
    let result = repository.register_device(registration).await
        .expect("Failed to register device");
    
    println!("Registered device with ID: {:?}", result.id);
}
```

## Database URL Format

The database URL format depends on the database driver you're using:

- SQLite: `sqlite:path/to/database.db` or `sqlite::memory:` for in-memory database
- PostgreSQL: `postgres://username:password@localhost:5432/database`
- MySQL: `mysql://username:password@localhost:3306/database`

## Error Handling

The crate provides a `DbError` enum for error handling:

```rust
use connectify_db::error::DbError;

fn handle_error(error: DbError) {
    match error {
        DbError::ConnectionError(msg) => println!("Connection error: {}", msg),
        DbError::QueryError(msg) => println!("Query error: {}", msg),
        DbError::ConfigError(msg) => println!("Configuration error: {}", msg),
        DbError::MigrationError(msg) => println!("Migration error: {}", msg),
    }
}
```

## Configuration

The crate uses the `DatabaseConfig` struct from `connectify-config` for configuration:

```rust
use connectify_config::DatabaseConfig;

fn create_config() -> DatabaseConfig {
    DatabaseConfig {
        url: Some("sqlite::memory:".to_string()),
        max_connections: Some(5),
        min_connections: Some(1),
        max_lifetime: Some(30 * 60), // 30 minutes
        idle_timeout: Some(10 * 60), // 10 minutes
        connect_timeout: Some(30),   // 30 seconds
    }
}
```