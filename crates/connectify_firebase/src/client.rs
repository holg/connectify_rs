//! Firebase Cloud Messaging client module
//!
//! This module provides a client for interacting with the Firebase Cloud Messaging (FCM) HTTP v1 API.
//! It includes functionality for sending push notifications to specific devices using FCM tokens
//! or to topics that devices can subscribe to.
//!
//! The main component is the `FirebaseClient` struct, which handles authentication and
//! communication with the FCM API. It also includes data structures for representing
//! FCM messages, notifications, and responses.

use crate::auth::get_firebase_auth_token;
use connectify_config::FirebaseConfig;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur when interacting with the Firebase Cloud Messaging API
#[derive(Error, Debug)]
pub enum FirebaseError {
    /// Error during authentication with Firebase
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Error during HTTP request to Firebase API
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Missing required configuration
    #[error("Missing configuration: {0}")]
    ConfigError(String),

    /// Error returned by the Firebase API
    #[error("Firebase API error: {0}")]
    ApiError(String),
}

/// A message to be sent via Firebase Cloud Messaging
///
/// This is the top-level structure that wraps a Message object
/// according to the FCM HTTP v1 API format.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FcmMessage {
    /// The message payload
    pub message: Message,
}

/// The message payload for Firebase Cloud Messaging
///
/// This structure contains the details of the message to be sent,
/// including the target (token or topic), notification content,
/// and optional custom data.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Message {
    /// Token identifying the target device (for single device targeting)
    ///
    /// This should be the registration token of the device you want to send
    /// the message to. Either token or topic must be provided, but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Topic that the target devices are subscribed to (for topic messaging)
    ///
    /// This should be the name of the topic that the target devices are
    /// subscribed to. Either token or topic must be provided, but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    /// The notification to be displayed on the user's device
    ///
    /// If provided, this will be displayed as a notification on the device.
    /// If not provided, the message will be a data-only message.
    pub notification: Option<Notification>,

    /// Custom key-value data to be sent with the message
    ///
    /// This data will be available to the client app that receives the message.
    pub data: Option<std::collections::HashMap<String, String>>,
}

/// The notification to be displayed on the user's device
///
/// This structure contains the title and body of the notification
/// that will be displayed on the user's device.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Notification {
    /// The title of the notification
    pub title: String,

    /// The body text of the notification
    pub body: String,
}

/// Response from the Firebase Cloud Messaging API
///
/// This structure contains the response from the FCM API
/// after a successful message send.
#[derive(Debug, Deserialize)]
pub struct FcmResponse {
    /// The unique ID of the message
    ///
    /// This is a string in the format "projects/{project_id}/messages/{message_id}"
    pub name: String,
}

/// Client for interacting with the Firebase Cloud Messaging API
///
/// This struct handles authentication and communication with the Firebase Cloud Messaging
/// HTTP v1 API. It provides methods for sending push notifications to devices or topics.
pub struct FirebaseClient {
    /// HTTP client for making requests to the FCM API
    client: Client,

    /// Configuration for Firebase, including project ID and service account key path
    config: FirebaseConfig,
}

impl FirebaseClient {
    /// Creates a new Firebase client with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The Firebase configuration, including project ID and service account key path
    ///
    /// # Returns
    ///
    /// A new `FirebaseClient` instance
    pub fn new(config: FirebaseConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Sends a push notification message via Firebase Cloud Messaging
    ///
    /// This method authenticates with Firebase, constructs the appropriate HTTP request,
    /// and sends the message to the FCM API.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send, including target (token or topic), notification content, and data
    ///
    /// # Returns
    ///
    /// * `Result<String, FirebaseError>` - On success, returns the message ID as a String.
    ///   On failure, returns a FirebaseError.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The project_id is missing from the FirebaseConfig
    /// * Authentication fails
    /// * The HTTP request fails
    /// * The FCM API returns an error response
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use connectify_firebase::client::{FcmMessage, FirebaseClient, Message, Notification};
    /// use connectify_config::FirebaseConfig;
    /// use std::collections::HashMap;
    ///
    /// async fn send_notification() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = FirebaseConfig {
    ///         project_id: Some("my-project-id".to_string()),
    ///         key_path: Some("/path/to/service-account.json".to_string()),
    ///     };
    ///     
    ///     let client = FirebaseClient::new(config);
    ///     
    ///     let mut data = HashMap::new();
    ///     data.insert("key1".to_string(), "value1".to_string());
    ///     
    ///     let message = FcmMessage {
    ///         message: Message {
    ///             token: Some("device-token".to_string()),
    ///             topic: None,
    ///             notification: Some(Notification {
    ///                 title: "Hello".to_string(),
    ///                 body: "World".to_string(),
    ///             }),
    ///             data: Some(data),
    ///         },
    ///     };
    ///     
    ///     let message_id = client.send_message(message).await?;
    ///     println!("Message sent with ID: {}", message_id);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn send_message(&self, message: FcmMessage) -> Result<String, FirebaseError> {
        let project_id = self.config.project_id.as_deref().ok_or_else(|| {
            FirebaseError::ConfigError("Missing project_id in FirebaseConfig".to_string())
        })?;

        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            project_id
        );

        let token = get_firebase_auth_token(&self.config)
            .await
            .map_err(|e| FirebaseError::AuthError(e.to_string()))?;

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .json(&message)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(FirebaseError::ApiError(error_text));
        }

        let fcm_response: FcmResponse = response.json().await?;
        Ok(fcm_response.name)
    }
}
