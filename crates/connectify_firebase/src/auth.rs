//! Authentication module for Firebase Cloud Messaging
//!
//! This module provides functionality to authenticate with Firebase Cloud Messaging
//! using a service account key file. It generates OAuth2 tokens that can be used
//! to authenticate API requests to the Firebase Cloud Messaging API.

use connectify_config::FirebaseConfig;
use std::{error::Error, path::Path};
use yup_oauth2::{read_service_account_key, ServiceAccountAuthenticator};

/// Obtains an OAuth2 access token for Firebase Cloud Messaging
///
/// This function reads a service account key file from the path specified in the
/// FirebaseConfig and uses it to authenticate with Google's OAuth2 service.
/// It requests a token with the appropriate scope for Firebase Cloud Messaging.
///
/// # Arguments
///
/// * `config` - A reference to a FirebaseConfig containing the path to the service account key file
///
/// # Returns
///
/// * `Result<String, Box<dyn Error + Send + Sync>>` - On success, returns the access token as a String.
///   On failure, returns a boxed error.
///
/// # Errors
///
/// This function will return an error if:
/// * The key_path is missing from the FirebaseConfig
/// * The service account key file cannot be read
/// * Authentication with Google's OAuth2 service fails
/// * No token is returned from the authentication service
pub async fn get_firebase_auth_token(
    config: &FirebaseConfig,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let key_path = config
        .key_path
        .as_deref()
        .ok_or("Missing key_path in FirebaseConfig")?;

    let sa_key = read_service_account_key(Path::new(key_path)).await?;

    // FCM requires the "https://www.googleapis.com/auth/firebase.messaging" scope
    let auth = ServiceAccountAuthenticator::builder(sa_key).build().await?;

    // Get an access token with the appropriate scope
    let auth_token = auth
        .token(&["https://www.googleapis.com/auth/firebase.messaging"])
        .await?;
    let fcm_result_token = match auth_token.token() {
        Some(token) => token,
        None => {
            return Err("No token available".into());
        }
    };

    Ok(fcm_result_token.to_string())
}
