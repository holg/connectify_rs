//! Repository modules for database access
//!
//! This module contains repository traits and implementations for different
//! database entities.

pub mod device_registration;
pub mod device_registration_factory;
pub mod device_registration_sql;

// Re-export the device registration repository and factory for ease of use
pub use device_registration::{DeviceRegistration, DeviceRegistrationRepository};
pub use device_registration_factory::DeviceRegistrationRepositoryFactory;
pub use device_registration_sql::SqlDeviceRegistrationRepository;
