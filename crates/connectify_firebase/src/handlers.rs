//! HTTP handlers for Firebase Cloud Messaging
//!
//! This module provides HTTP handlers for interacting with Firebase Cloud Messaging (FCM)
//! through a REST API. It includes handlers for sending push notifications to devices
//! or topics, as well as the request and response types used by these handlers.
//!
//! The handlers are designed to be used with the Axum web framework and include
//! OpenAPI documentation when the `openapi` feature is enabled.

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::client::{FcmMessage, FirebaseClient, FirebaseError, Message, Notification};

/// Shared state for Firebase handlers
///
/// This struct holds the shared state that is passed to the Firebase handlers.
/// It contains a reference to the Firebase client that is used to send notifications.
#[derive(Clone)]
pub struct FirebaseState {
    /// The Firebase client used to send notifications
    pub client: Arc<FirebaseClient>,
}

/// Request body for sending a notification
///
/// This struct represents the JSON payload that should be sent to the
/// `/send-notification` endpoint to send a push notification.
///
/// Either `token` or `topic` must be provided, but not both.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendNotificationRequest {
    /// Token identifying the target device (for single device targeting)
    ///
    /// This should be the registration token of the device you want to send
    /// the message to. Either token or topic must be provided, but not both.
    pub token: Option<String>,

    /// Topic that the target devices are subscribed to (for topic messaging)
    ///
    /// This should be the name of the topic that the target devices are
    /// subscribed to. Either token or topic must be provided, but not both.
    pub topic: Option<String>,

    /// The title of the notification
    pub title: String,

    /// The body text of the notification
    pub body: String,

    /// Custom key-value data to be sent with the message
    ///
    /// This data will be available to the client app that receives the message.
    pub data: Option<std::collections::HashMap<String, String>>,
}

/// Response body for the send notification endpoint
///
/// This struct represents the JSON response that is returned from the
/// `/send-notification` endpoint after attempting to send a push notification.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendNotificationResponse {
    /// Whether the notification was sent successfully
    pub success: bool,

    /// The ID of the message if it was sent successfully
    ///
    /// This will be a string in the format "projects/{project_id}/messages/{message_id}"
    /// if the message was sent successfully, or None if it failed.
    pub message_id: Option<String>,

    /// Error message if the notification failed to send
    ///
    /// This will be a string describing the error if the message failed to send,
    /// or None if it was sent successfully.
    pub error: Option<String>,
}

/// Request body for registering a device
///
/// This struct represents the JSON payload that should be sent to the
/// `/register-device` endpoint to register a device for push notifications.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RegisterDeviceRequest {
    /// The user ID to associate with the registration
    pub user_id: String,

    /// The device ID to associate with the registration
    pub device_id: String,

    /// The Firebase Cloud Messaging registration token
    pub registration_token: String,
}

/// Response body for the register device endpoint
///
/// This struct represents the JSON response that is returned from the
/// `/register-device` endpoint after attempting to register a device.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RegisterDeviceResponse {
    /// Whether the device was registered successfully
    pub success: bool,

    /// The user ID associated with the registration
    pub user_id: Option<String>,

    /// The device ID associated with the registration
    pub device_id: Option<String>,

    /// Error message if registration failed
    pub error: Option<String>,
}

/// Request body for sending a notification to a user
///
/// This struct represents the JSON payload that should be sent to the
/// `/send-notification-to-user` endpoint to send a push notification to all
/// devices registered for a user.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendNotificationToUserRequest {
    /// The user ID to send notifications to
    pub user_id: String,

    /// The title of the notification
    pub title: String,

    /// The body text of the notification
    pub body: String,

    /// Custom key-value data to be sent with the message
    pub data: Option<std::collections::HashMap<String, String>>,
}

/// Response body for the send notification to user endpoint
///
/// This struct represents the JSON response that is returned from the
/// `/send-notification-to-user` endpoint after attempting to send push
/// notifications to all devices registered for a user.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendNotificationToUserResponse {
    /// Whether the notifications were sent successfully
    pub success: bool,

    /// The number of devices that received the notification
    pub device_count: usize,

    /// The IDs of the messages if they were sent successfully
    pub message_ids: Vec<String>,

    /// Error message if sending notifications failed
    pub error: Option<String>,
}

/// Handler for sending push notifications via Firebase Cloud Messaging
///
/// This handler accepts a JSON payload with notification details and sends
/// a push notification to the specified device token or topic.
///
/// # Request
///
/// The request must include either a `token` (for single device) or a `topic`
/// (for topic messaging), but not both. It must also include a `title` and `body`
/// for the notification, and optionally can include custom `data`.
///
/// # Responses
///
/// - 200 OK: Notification sent successfully
/// - 400 Bad Request: Missing or invalid parameters
/// - 401 Unauthorized: Authentication failed
/// - 500 Internal Server Error: Server-side error
///
/// Handler for registering a device for push notifications
///
/// This handler accepts a JSON payload with device registration details and
/// stores the registration in the database.
///
/// # Request
///
/// The request must include a `user_id`, `device_id`, and `registration_token`.
///
/// # Responses
///
/// - 200 OK: Device registered successfully
/// - 400 Bad Request: Missing or invalid parameters
/// - 500 Internal Server Error: Server-side error
///
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/firebase/register-device",
    request_body = RegisterDeviceRequest,
    responses(
        (status = 200, description = "Device registered successfully", body = RegisterDeviceResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Firebase"
))]
pub async fn register_device_handler(
    State(state): State<Arc<FirebaseState>>,
    Json(payload): Json<RegisterDeviceRequest>,
) -> Response {
    debug!("Registering device for user: {}", payload.user_id);

    match state
        .client
        .register_device(
            payload.user_id.clone(),
            payload.device_id.clone(),
            payload.registration_token,
        )
        .await
    {
        Ok(registration) => {
            info!(
                "Successfully registered device for user: {}",
                registration.user_id
            );
            Json(RegisterDeviceResponse {
                success: true,
                user_id: Some(registration.user_id),
                device_id: Some(registration.device_id),
                error: None,
            })
            .into_response()
        }
        Err(err) => {
            error!("Failed to register device: {:?}", err);
            let status = match &err {
                FirebaseError::AuthError(_) => StatusCode::UNAUTHORIZED,
                FirebaseError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                FirebaseError::RequestError(_) => StatusCode::BAD_REQUEST,
                FirebaseError::ApiError(_) => StatusCode::BAD_REQUEST,
                #[cfg(feature = "database")]
                FirebaseError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };

            (
                status,
                Json(RegisterDeviceResponse {
                    success: false,
                    user_id: Some(payload.user_id),
                    device_id: Some(payload.device_id),
                    error: Some(err.to_string()),
                }),
            )
                .into_response()
        }
    }
}

/// Handler for sending push notifications to all devices registered for a user
///
/// This handler accepts a JSON payload with notification details and sends
/// push notifications to all devices registered for the specified user.
///
/// # Request
///
/// The request must include a `user_id`, `title`, and `body` for the notification,
/// and optionally can include custom `data`.
///
/// # Responses
///
/// - 200 OK: Notifications sent successfully
/// - 400 Bad Request: Missing or invalid parameters
/// - 401 Unauthorized: Authentication failed
/// - 500 Internal Server Error: Server-side error
///
#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/firebase/send-notification-to-user",
    request_body = SendNotificationToUserRequest,
    responses(
        (status = 200, description = "Notifications sent successfully", body = SendNotificationToUserResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Firebase"
))]
pub async fn send_notification_to_user_handler(
    State(state): State<Arc<FirebaseState>>,
    Json(payload): Json<SendNotificationToUserRequest>,
) -> Response {
    debug!(
        "Sending notification to all devices for user: {}",
        payload.user_id
    );

    let notification = Notification {
        title: payload.title,
        body: payload.body,
    };

    match state
        .client
        .send_notification_to_user(&payload.user_id, notification, payload.data)
        .await
    {
        Ok(message_ids) => {
            info!(
                "Successfully sent notifications to {} devices for user: {}",
                message_ids.len(),
                payload.user_id
            );

            Json(SendNotificationToUserResponse {
                success: true,
                device_count: message_ids.len(),
                message_ids,
                error: None,
            })
            .into_response()
        }
        Err(err) => {
            error!("Failed to send notifications to user: {:?}", err);
            let status = match &err {
                FirebaseError::AuthError(_) => StatusCode::UNAUTHORIZED,
                FirebaseError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                FirebaseError::RequestError(_) => StatusCode::BAD_REQUEST,
                FirebaseError::ApiError(_) => StatusCode::BAD_REQUEST,
                #[cfg(feature = "database")]
                FirebaseError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };

            (
                status,
                Json(SendNotificationToUserResponse {
                    success: false,
                    device_count: 0,
                    message_ids: Vec::new(),
                    error: Some(err.to_string()),
                }),
            )
                .into_response()
        }
    }
}

#[axum::debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/firebase/send-notification",
    request_body = SendNotificationRequest,
    responses(
        (status = 200, description = "Notification sent successfully", body = SendNotificationResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Firebase"
))]
pub async fn send_notification_handler(
    State(state): State<Arc<FirebaseState>>,
    Json(payload): Json<SendNotificationRequest>,
) -> Response {
    if payload.token.is_none() && payload.topic.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(SendNotificationResponse {
                success: false,
                message_id: None,
                error: Some("Either token or topic must be provided".to_string()),
            }),
        )
            .into_response();
    }

    if payload.token.is_some() && payload.topic.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(SendNotificationResponse {
                success: false,
                message_id: None,
                error: Some("Cannot provide both token and topic".to_string()),
            }),
        )
            .into_response();
    }

    let message = FcmMessage {
        message: Message {
            token: payload.token,
            topic: payload.topic,
            notification: Some(Notification {
                title: payload.title,
                body: payload.body,
            }),
            data: payload.data,
        },
    };

    match state.client.send_message(message).await {
        Ok(message_id) => {
            info!("Successfully sent FCM notification: {}", message_id);
            Json(SendNotificationResponse {
                success: true,
                message_id: Some(message_id),
                error: None,
            })
            .into_response()
        }
        Err(err) => {
            error!("Failed to send FCM notification: {:?}", err);
            let status = match &err {
                FirebaseError::AuthError(_) => StatusCode::UNAUTHORIZED,
                FirebaseError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                FirebaseError::RequestError(_) => StatusCode::BAD_REQUEST,
                FirebaseError::ApiError(_) => StatusCode::BAD_REQUEST,
                #[cfg(feature = "database")]
                FirebaseError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };

            (
                status,
                Json(SendNotificationResponse {
                    success: false,
                    message_id: None,
                    error: Some(err.to_string()),
                }),
            )
                .into_response()
        }
    }
}
