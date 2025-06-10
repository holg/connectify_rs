//! Factory for creating device registration repositories
//!
//! This module provides a factory for creating device registration repositories
//! that are designed to be database agnostic.

use crate::repositories::device_registration_sql::SqlDeviceRegistrationRepository;
use crate::{DbClient, RepositoryFactory};

/// Factory for creating device registration repositories
///
/// This factory provides methods for creating device registration repositories
/// using different database clients.
#[derive(Debug, Clone)]
pub struct DeviceRegistrationRepositoryFactory;

impl DeviceRegistrationRepositoryFactory {
    /// Create a new device registration repository factory
    ///
    /// # Returns
    ///
    /// A new device registration repository factory
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeviceRegistrationRepositoryFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl RepositoryFactory<SqlDeviceRegistrationRepository, DbClient>
    for DeviceRegistrationRepositoryFactory
{
    /// Create a new device registration repository
    ///
    /// # Arguments
    ///
    /// * `db_client` - The database client to use
    ///
    /// # Returns
    ///
    /// A new device registration repository
    fn create_repository(&self, db_client: DbClient) -> SqlDeviceRegistrationRepository {
        SqlDeviceRegistrationRepository::new(db_client)
    }
}
