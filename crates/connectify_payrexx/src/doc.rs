// File: crates/connectify_payrexx/src/doc.rs

#![allow(dead_code)] // Allow dead code for doc functions
#![cfg(feature = "openapi")]

use utoipa::OpenApi;
use crate::logic::{CreateGatewayRequest, CreateGatewayResponse, PayrexxWebhookPayload}; // Import schemas

// Define a dummy function with the handler's attributes for utoipa
#[utoipa::path(
    post,
    path = "/payrexx/create-gateway",
    request_body = CreateGatewayRequest,
    responses(
        (status = 200, description = "Gateway created successfully", body = CreateGatewayResponse),
        (status = 400, description = "Bad request (e.g., invalid input)"),
        (status = 500, description = "Internal server error or Payrexx API error")
    ),
    tag = "Payrexx" // Group under Payrexx tag
)]
fn doc_create_gateway_handler() {}

#[utoipa::path(
    post,
    path = "/payrexx/webhook",
    // Describe the webhook payload structure Payrexx sends
    request_body(content = PayrexxWebhookPayload, description = "Webhook payload sent by Payrexx", content_type = "application/json"),
    responses(
        // Payrexx typically expects a simple success/failure status code
        (status = 200, description = "Webhook received and acknowledged successfully"),
        (status = 400, description = "Bad Request (e.g., invalid signature, malformed payload)"),
        (status = 500, description = "Internal Server Error (failed to process webhook)")
        // Payrexx might ignore response body, so no body needed here usually
    ),
    tag = "Payrexx" // Group under the same tag
)]
fn doc_payrexx_webhook_handler() {}

// --- Main OpenAPI Definition ---
#[derive(OpenApi)]
#[openapi(
    paths(
        // List all documented paths for this feature
        doc_create_gateway_handler,
        doc_payrexx_webhook_handler
    ),
    components(
        // List all schemas used in the paths
        schemas(CreateGatewayRequest, CreateGatewayResponse, PayrexxWebhookPayload)
    ),
    tags(
        // Define the tag used above for grouping endpoints
        (name = "Payrexx", description = "Payrexx Payment Gateway API")
    )
)]
pub struct PayrexxApiDoc;
