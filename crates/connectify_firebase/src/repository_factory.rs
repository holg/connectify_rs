//! Factory for creating device registration repositories
//!
//! This module provides a factory for creating device registration repositories
//! that are designed to be database agnostic.

use crate::repository::DeviceRegistrationRepository;
#[cfg(feature = "database")]
use connectify_db::{
    DbClient, DeviceRegistrationRepositoryFactory as DbDeviceRegistrationRepositoryFactory,
    RepositoryFactory,
};

#[cfg(not(feature = "database"))]
pub trait RepositoryFactory<R, C> {
    fn create_repository(&self, config: C) -> R;
}

/// Factory for creating device registration repositories
///
/// This factory provides methods for creating device registration repositories
/// using different database clients.
#[derive(Debug, Clone)]
pub struct DeviceRegistrationRepositoryFactory {
    #[cfg(feature = "database")]
    db_factory: DbDeviceRegistrationRepositoryFactory,
}

impl DeviceRegistrationRepositoryFactory {
    /// Create a new device registration repository factory
    ///
    /// # Returns
    ///
    /// A new device registration repository factory
    pub fn new() -> Self {
        #[cfg(feature = "database")]
        {
            Self {
                db_factory: DbDeviceRegistrationRepositoryFactory::new(),
            }
        }

        #[cfg(not(feature = "database"))]
        {
            Self {}
        }
    }
}

impl Default for DeviceRegistrationRepositoryFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "database")]
impl RepositoryFactory<crate::repository::DeviceRegistrationRepository, DbClient>
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
    fn create_repository(&self, db_client: DbClient) -> DeviceRegistrationRepository {
        let inner = self.db_factory.create_repository(db_client);
        DeviceRegistrationRepository::new(inner)
    }
}

#[cfg(not(feature = "database"))]
impl RepositoryFactory<DeviceRegistrationRepository, ()> for DeviceRegistrationRepositoryFactory {
    /// Create a new device registration repository
    ///
    /// This is a stub implementation when the database feature is not enabled.
    ///
    /// # Arguments
    ///
    /// * `_` - Ignored
    ///
    /// # Returns
    ///
    /// A new device registration repository
    fn create_repository(&self, _: ()) -> DeviceRegistrationRepository {
        DeviceRegistrationRepository::new(())
    }
}
