//! Repository for Firebase device registrations
//!
//! This module provides a wrapper around the generic device registration repository
//! from connectify_db.

use crate::client::FirebaseError;
use crate::models::DeviceRegistration;

#[cfg(feature = "database")]
use connectify_db::{
    repositories::device_registration_sql::SqlDeviceRegistrationRepository,
    DeviceRegistrationRepository as DbDeviceRegistrationRepository,
};

/// Repository for Firebase device registrations
///
/// This struct wraps the generic device registration repository from connectify_db
/// and provides methods specific to Firebase device registrations.
#[derive(Debug, Clone)]
pub struct DeviceRegistrationRepository {
    #[cfg(feature = "database")]
    inner: SqlDeviceRegistrationRepository,
}

impl DeviceRegistrationRepository {
    /// Create a new device registration repository
    ///
    /// # Arguments
    ///
    /// * `inner` - The inner device registration repository
    ///
    /// # Returns
    ///
    /// A new device registration repository
    #[cfg(feature = "database")]
    pub fn new(inner: SqlDeviceRegistrationRepository) -> Self {
        Self { inner }
    }

    /// Create a new device registration repository without database support
    ///
    /// # Returns
    ///
    /// A new device registration repository
    #[cfg(not(feature = "database"))]
    pub fn new(_: ()) -> Self {
        Self {}
    }

    /// Initialize the database schema
    ///
    /// This function creates the necessary tables for storing device registrations
    /// if they don't already exist.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the schema was initialized successfully, or an error if it failed
    pub async fn init_schema(&self) -> Result<(), FirebaseError> {
        #[cfg(feature = "database")]
        {
            self.inner
                .init_schema()
                .await
                .map_err(FirebaseError::DbError)
        }

        #[cfg(not(feature = "database"))]
        {
            Ok(())
        }
    }

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
    pub async fn register_device(
        &self,
        registration: DeviceRegistration,
    ) -> Result<DeviceRegistration, FirebaseError> {
        #[cfg(feature = "database")]
        {
            // Register the device
            let result = self
                .inner
                .register_device(registration)
                .await
                .map_err(FirebaseError::DbError)?;

            Ok(result)
        }

        #[cfg(not(feature = "database"))]
        {
            Err(FirebaseError::ConfigError(
                "Database feature is not enabled".to_string(),
            ))
        }
    }

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
    pub async fn find_by_user_and_device(
        &self,
        _user_id: &str,
        _device_id: &str,
    ) -> Result<Option<DeviceRegistration>, FirebaseError> {
        #[cfg(feature = "database")]
        {
            let result = self
                .inner
                .find_by_user_and_device(_user_id, _device_id)
                .await
                .map_err(FirebaseError::DbError)?;

            Ok(result)
        }

        #[cfg(not(feature = "database"))]
        {
            Ok(None)
        }
    }

    /// Find all device registrations for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// A list of device registrations for the user
    pub async fn find_by_user(
        &self,
        _user_id: &str,
    ) -> Result<Vec<DeviceRegistration>, FirebaseError> {
        #[cfg(feature = "database")]
        {
            let results = self
                .inner
                .find_by_user(_user_id)
                .await
                .map_err(FirebaseError::DbError)?;

            Ok(results)
        }

        #[cfg(not(feature = "database"))]
        {
            Ok(Vec::new())
        }
    }

    /// Find all device registrations
    ///
    /// # Returns
    ///
    /// A list of all device registrations
    pub async fn find_all(&self) -> Result<Vec<DeviceRegistration>, FirebaseError> {
        #[cfg(feature = "database")]
        {
            let results = self
                .inner
                .find_all()
                .await
                .map_err(FirebaseError::DbError)?;

            Ok(results)
        }

        #[cfg(not(feature = "database"))]
        {
            Ok(Vec::new())
        }
    }

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
    pub async fn delete_registration(
        &self,
        user_id: &str,
        device_id: &str,
    ) -> Result<bool, FirebaseError> {
        #[cfg(feature = "database")]
        {
            self.inner
                .delete_registration(user_id, device_id)
                .await
                .map_err(FirebaseError::DbError)
        }

        #[cfg(not(feature = "database"))]
        {
            Ok(false)
        }
    }
}
