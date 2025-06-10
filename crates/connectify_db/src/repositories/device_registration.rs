//! Repository for device registrations
//!
//! This module provides a generic interface for storing and retrieving device registration tokens
//! in the database.

use crate::error::DbError;
use sqlx::FromRow;

// Re-export DeviceRegistration from connectify_common for convenience
pub use connectify_common::models::DeviceRegistration;

// Define a DB-specific wrapper for DeviceRegistration that implements FromRow
#[derive(Debug, Clone, FromRow)]
pub struct DbDeviceRegistration {
    pub id: Option<i64>,
    pub user_id: String,
    pub device_id: String,
    pub registration_token: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<DbDeviceRegistration> for DeviceRegistration {
    fn from(db: DbDeviceRegistration) -> Self {
        Self {
            id: db.id,
            user_id: db.user_id,
            device_id: db.device_id,
            registration_token: db.registration_token,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

impl From<DeviceRegistration> for DbDeviceRegistration {
    fn from(dr: DeviceRegistration) -> Self {
        Self {
            id: dr.id,
            user_id: dr.user_id,
            device_id: dr.device_id,
            registration_token: dr.registration_token,
            created_at: dr.created_at,
            updated_at: dr.updated_at,
        }
    }
}

/// Repository for device registrations
///
/// This trait defines the interface for storing and retrieving device registration tokens
/// in the database.
pub trait DeviceRegistrationRepository {
    /// Initialize the database schema
    ///
    /// This function creates the necessary tables for storing device registrations
    /// if they don't already exist.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the schema was initialized successfully, or an error if it failed
    fn init_schema(&self) -> impl std::future::Future<Output = Result<(), DbError>> + Send;

    /// Register a device
    ///
    /// This function stores a device registration token in the database.
    /// If a registration already exists for the given user and device, it will be updated.
    ///
    /// # Arguments
    ///
    /// * `registration` - The device registration to store
    ///
    /// # Returns
    ///
    /// The stored device registration with its ID and timestamps set
    fn register_device(
        &self,
        registration: DeviceRegistration,
    ) -> impl std::future::Future<Output = Result<DeviceRegistration, DbError>> + Send;

    /// Find a device registration by user ID and device ID
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `device_id` - The device ID
    ///
    /// # Returns
    ///
    /// The device registration if found, or None if not found
    fn find_by_user_and_device(
        &self,
        user_id: &str,
        device_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<DeviceRegistration>, DbError>> + Send;

    /// Find all device registrations for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// A list of device registrations for the user
    fn find_by_user(
        &self,
        user_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<DeviceRegistration>, DbError>> + Send;

    /// Find all device registrations
    ///
    /// # Returns
    ///
    /// A list of all device registrations
    fn find_all(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<DeviceRegistration>, DbError>> + Send;

    /// Delete a device registration
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `device_id` - The device ID
    ///
    /// # Returns
    ///
    /// `true` if a registration was deleted, `false` if no registration was found
    fn delete_registration(
        &self,
        user_id: &str,
        device_id: &str,
    ) -> impl std::future::Future<Output = Result<bool, DbError>> + Send;
}
