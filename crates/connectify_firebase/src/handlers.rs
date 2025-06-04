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
use tracing::{error, info};

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
