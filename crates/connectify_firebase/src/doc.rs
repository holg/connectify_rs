#![allow(dead_code)]
// #![cfg(feature = "openapi")] // not needed as we do this in lib.rs already!
use utoipa::OpenApi;

use crate::client::{FcmMessage, Message, Notification};
use crate::handlers::{
    RegisterDeviceRequest, RegisterDeviceResponse, SendNotificationRequest,
    SendNotificationResponse,
};

#[utoipa::path(
    post,
    path = "/firebase/send-notification",
    request_body(content = SendNotificationRequest, example = json!({
        "topic": "app_general_alerts",
        "title": "New Message",
        "body": "You have received a new message",
        "data": {
            "message_id": "123456",
            "sender": "John Doe"
        }
    })),
    responses(
        (status = 200, description = "Notification sent successfully", body = SendNotificationResponse,
         example = json!({
             "success": true,
             "message_id": "projects/my-project/messages/1234567890",
             "error": null
         })
        ),
        (status = 400, description = "Bad Request",
         example = json!({
             "success": false,
             "message_id": null,
             "error": "Either token or topic must be provided"
         })
        ),
        (status = 401, description = "Unauthorized",
         example = json!({
             "success": false,
             "message_id": null,
             "error": "Authentication error: Failed to obtain access token"
         })
        ),
        (status = 500, description = "Internal Server Error",
         example = json!({
             "success": false,
             "message_id": null,
             "error": "Failed to send notification"
         })
        )
    ),
    tag = "Firebase"
)]
fn doc_send_notification_handler() {}

#[utoipa::path(
    post,
    path = "/firebase/register-device",
    request_body(content = RegisterDeviceRequest, example = json!({
        "user_id": "user123",
        "device_id": "device456",
        "registration_token": "fcm-registration-token-example"
    })),
    responses(
        (status = 200, description = "Device registered successfully", body = RegisterDeviceResponse,
         example = json!({
             "success": true,
             "user_id": "user123",
             "device_id": "device456",
             "error": null
         })
        ),
        (status = 400, description = "Bad Request",
         example = json!({
             "success": false,
             "user_id": "user123",
             "device_id": "device456",
             "error": "Invalid registration token"
         })
        ),
        (status = 500, description = "Internal Server Error",
         example = json!({
             "success": false,
             "user_id": "user123",
             "device_id": "device456",
             "error": "Failed to register device"
         })
        )
    ),
    tag = "Firebase"
)]
fn doc_register_device_handler() {}

#[derive(OpenApi)]
#[openapi(
    paths(
        doc_send_notification_handler,
        doc_register_device_handler,
    ),
    components(
        schemas(
            SendNotificationRequest,
            SendNotificationResponse,
            RegisterDeviceRequest,
            RegisterDeviceResponse,
            FcmMessage,
            Message,
            Notification,
        )
    ),
    tags(
        (name = "Firebase", description = "Firebase Cloud Messaging API")
    ),
    servers(
        (url = "/api", description = "Firebase Cloud Messaging API server")
    )
)]
pub struct FirebaseApiDoc;
