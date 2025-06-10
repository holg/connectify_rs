// --- File: crates/connectify_common/src/models.rs ---

// This file contains data structures and models that are common across the application.
// Examples include:
// - Common request/response types
// - Shared data structures
// - Error types
// - Configuration models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a device registration token
///
/// This struct is used to store and retrieve device registration tokens.
/// Each token is associated with a user ID and a device ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegistration {
    /// The unique identifier for this registration
    pub id: Option<i64>,

    /// The user ID associated with this registration
    pub user_id: String,

    /// The device ID associated with this registration
    pub device_id: String,

    /// The registration token
    pub registration_token: String,

    /// The timestamp when this registration was created
    pub created_at: Option<DateTime<Utc>>,

    /// The timestamp when this registration was last updated
    pub updated_at: Option<DateTime<Utc>>,
}

impl DeviceRegistration {
    /// Create a new device registration
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID associated with this registration
    /// * `device_id` - The device ID associated with this registration
    /// * `registration_token` - The registration token
    ///
    /// # Returns
    ///
    /// A new device registration
    pub fn new(user_id: String, device_id: String, registration_token: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            user_id,
            device_id,
            registration_token,
            created_at: Some(now),
            updated_at: Some(now),
        }
    }
}
