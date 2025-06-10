//! SQL implementation of the device registration repository
//!
//! This module provides a SQL implementation of the DeviceRegistrationRepository trait.

use crate::error::DbError;
use crate::repositories::device_registration::{DeviceRegistration, DeviceRegistrationRepository};
use crate::DbClient;
use sqlx::Row;
use tracing::{debug, error, info};

/// SQL implementation of the device registration repository
#[derive(Debug, Clone)]
pub struct SqlDeviceRegistrationRepository {
    /// The database client
    db_client: DbClient,
}

impl SqlDeviceRegistrationRepository {
    /// Create a new SQL device registration repository
    ///
    /// # Arguments
    ///
    /// * `db_client` - The database client
    ///
    /// # Returns
    ///
    /// A new SQL device registration repository
    pub fn new(db_client: DbClient) -> Self {
        Self { db_client }
    }
}

impl DeviceRegistrationRepository for SqlDeviceRegistrationRepository {
    async fn init_schema(&self) -> Result<(), DbError> {
        debug!("Initializing device registration schema");

        // Create the device_registrations table if it doesn't exist
        let query = r#"
            CREATE TABLE IF NOT EXISTS device_registrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                device_id TEXT NOT NULL,
                registration_token TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(user_id, device_id)
            )
        "#;

        self.db_client.execute(query).await?;

        info!("Device registration schema initialized successfully");
        Ok(())
    }

    async fn register_device(
        &self,
        registration: DeviceRegistration,
    ) -> Result<DeviceRegistration, DbError> {
        debug!("Registering device for user: {}", registration.user_id);

        // Check if a registration already exists for this user and device
        let existing = self
            .find_by_user_and_device(&registration.user_id, &registration.device_id)
            .await?;

        if let Some(_existing) = existing {
            // Update the existing registration
            debug!(
                "Updating existing registration for user: {} and device: {}",
                registration.user_id, registration.device_id
            );

            let query = r#"
                UPDATE device_registrations
                SET registration_token = $1, updated_at = CURRENT_TIMESTAMP
                WHERE user_id = $2 AND device_id = $3
                RETURNING id, user_id, device_id, registration_token, created_at, updated_at
            "#;

            // Use a manual row mapping approach instead of query_as to avoid issues with DateTime<Utc>
            let row = sqlx::query(query)
                .bind(&registration.registration_token)
                .bind(&registration.user_id)
                .bind(&registration.device_id)
                .fetch_one(self.db_client.pool())
                .await
                .map_err(|e| {
                    error!("Failed to update device registration: {}", e);
                    DbError::QueryError(e.to_string())
                })?;

            let updated = DeviceRegistration {
                id: row.try_get("id").ok(),
                user_id: row.try_get("user_id").unwrap_or_default(),
                device_id: row.try_get("device_id").unwrap_or_default(),
                registration_token: row.try_get("registration_token").unwrap_or_default(),
                created_at: None, // DateTime<Utc> doesn't implement Decode for sqlx::Any
                updated_at: None, // DateTime<Utc> doesn't implement Decode for sqlx::Any
            };

            info!("Device registration updated successfully");
            Ok(updated)
        } else {
            // Insert a new registration
            debug!(
                "Creating new registration for user: {} and device: {}",
                registration.user_id, registration.device_id
            );

            let query = r#"
                INSERT INTO device_registrations (user_id, device_id, registration_token)
                VALUES ($1, $2, $3)
                RETURNING id, user_id, device_id, registration_token, created_at, updated_at
            "#;

            // Use a manual row mapping approach instead of query_as to avoid issues with DateTime<Utc>
            let row = sqlx::query(query)
                .bind(&registration.user_id)
                .bind(&registration.device_id)
                .bind(&registration.registration_token)
                .fetch_one(self.db_client.pool())
                .await
                .map_err(|e| {
                    error!("Failed to insert device registration: {}", e);
                    DbError::QueryError(e.to_string())
                })?;

            let inserted = DeviceRegistration {
                id: row.try_get("id").ok(),
                user_id: row.try_get("user_id").unwrap_or_default(),
                device_id: row.try_get("device_id").unwrap_or_default(),
                registration_token: row.try_get("registration_token").unwrap_or_default(),
                created_at: None, // DateTime<Utc> doesn't implement Decode for sqlx::Any
                updated_at: None, // DateTime<Utc> doesn't implement Decode for sqlx::Any
            };

            info!("Device registration created successfully");
            Ok(inserted)
        }
    }

    async fn find_by_user_and_device(
        &self,
        user_id: &str,
        device_id: &str,
    ) -> Result<Option<DeviceRegistration>, DbError> {
        debug!(
            "Finding device registration for user: {} and device: {}",
            user_id, device_id
        );

        let query = r#"
            SELECT id, user_id, device_id, registration_token, created_at, updated_at
            FROM device_registrations
            WHERE user_id = $1 AND device_id = $2
        "#;

        let result = sqlx::query(query)
            .bind(user_id)
            .bind(device_id)
            .fetch_optional(self.db_client.pool())
            .await
            .map_err(|e| {
                error!("Failed to find device registration: {}", e);
                DbError::QueryError(e.to_string())
            })?;

        if let Some(row) = result {
            Ok(Some(DeviceRegistration {
                id: row.try_get("id").ok(),
                user_id: row.try_get("user_id").unwrap_or_default(),
                device_id: row.try_get("device_id").unwrap_or_default(),
                registration_token: row.try_get("registration_token").unwrap_or_default(),
                created_at: None, // DateTime<Utc> doesn't implement Decode for sqlx::Any
                updated_at: None, // DateTime<Utc> doesn't implement Decode for sqlx::Any
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_user(&self, user_id: &str) -> Result<Vec<DeviceRegistration>, DbError> {
        debug!("Finding all device registrations for user: {}", user_id);

        let query = r#"
            SELECT id, user_id, device_id, registration_token, created_at, updated_at
            FROM device_registrations
            WHERE user_id = $1
        "#;

        let rows = sqlx::query(query)
            .bind(user_id)
            .fetch_all(self.db_client.pool())
            .await
            .map_err(|e| {
                error!("Failed to find device registrations: {}", e);
                DbError::QueryError(e.to_string())
            })?;

        let results = rows
            .into_iter()
            .map(|row| {
                DeviceRegistration {
                    id: row.try_get("id").ok(),
                    user_id: row.try_get("user_id").unwrap_or_default(),
                    device_id: row.try_get("device_id").unwrap_or_default(),
                    registration_token: row.try_get("registration_token").unwrap_or_default(),
                    created_at: None, // DateTime<Utc> doesn't implement Decode for sqlx::Any
                    updated_at: None, // DateTime<Utc> doesn't implement Decode for sqlx::Any
                }
            })
            .collect();

        Ok(results)
    }

    async fn find_all(&self) -> Result<Vec<DeviceRegistration>, DbError> {
        debug!("Finding all device registrations");

        let query = r#"
            SELECT id, user_id, device_id, registration_token, created_at, updated_at
            FROM device_registrations
        "#;

        let rows = sqlx::query(query)
            .fetch_all(self.db_client.pool())
            .await
            .map_err(|e| {
                error!("Failed to find device registrations: {}", e);
                DbError::QueryError(e.to_string())
            })?;

        let results = rows
            .into_iter()
            .map(|row| {
                DeviceRegistration {
                    id: row.try_get("id").ok(),
                    user_id: row.try_get("user_id").unwrap_or_default(),
                    device_id: row.try_get("device_id").unwrap_or_default(),
                    registration_token: row.try_get("registration_token").unwrap_or_default(),
                    created_at: None, // DateTime<Utc> doesn't implement Decode for sqlx::Any
                    updated_at: None, // DateTime<Utc> doesn't implement Decode for sqlx::Any
                }
            })
            .collect();

        Ok(results)
    }

    async fn delete_registration(&self, user_id: &str, device_id: &str) -> Result<bool, DbError> {
        debug!(
            "Deleting device registration for user: {} and device: {}",
            user_id, device_id
        );

        let query = r#"
            DELETE FROM device_registrations
            WHERE user_id = $1 AND device_id = $2
        "#;

        let result = sqlx::query(query)
            .bind(user_id)
            .bind(device_id)
            .execute(self.db_client.pool())
            .await
            .map_err(|e| {
                error!("Failed to delete device registration: {}", e);
                DbError::QueryError(e.to_string())
            })?;

        Ok(result.rows_affected() > 0)
    }
}
