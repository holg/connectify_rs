// File: crates/connectify_payrexx/src/doc.rs
#![allow(dead_code)] // Allow dead code for doc functions
#![cfg(feature = "openapi")]
use crate::handlers::RedirectQuery;
use crate::logic::{
    CreateGatewayRequest, CreateGatewayResponse, PayrexxWebhookContact, PayrexxWebhookCustomField,
    PayrexxWebhookInstance, PayrexxWebhookInvoice, PayrexxWebhookInvoiceProduct,
    PayrexxWebhookPayload, PayrexxWebhookPayment, PayrexxWebhookTransaction,
};
use utoipa::OpenApi; // Import schemas

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

// --- Doc function for Webhook ---
#[utoipa::path(
    post,
    path = "/payrexx/webhook", // Path relative to /api
    request_body(content = PayrexxWebhookPayload, description = "Webhook payload sent by Payrexx", content_type = "application/json",
        // Provide an example matching the structure you received
        example = json!({
            "transaction": {
                "id": 1,
                "psp": null,
                "lang": "de",
                "mode": "TEST",
                "time": "2025-04-01 12:00:00",
                "type": "E-Commerce",
                "uuid": null,
                "pspId": 1,
                "amount": 10000,
                "status": "confirmed",
                "contact": {
                    "id": null, "zip": "1234", "uuid": null, "email": "swissapp@swissappgroup.com",
                    "phone": "+41 123456789", "place": "Testhausen", "title": "2", "street": "Testweg 1",
                    "company": "Some Group GmbH", "country": "Schweiz", "countryISO": "CH",
                    "lastname": "Mustermann", "firstname": "Max", "date_of_birth": "01.01.1970",
                    "delivery_zip": "", "delivery_place": "", "delivery_title": "", "delivery_street": "",
                    "delivery_company": "", "delivery_country": "", "delivery_lastname": "",
                    "delivery_firstname": "", "delivery_countryISO": ""
                 },
                "invoice": {
                    "test": 1, "number": "123456", "currency": "CHF", "discount": null,
                    "products": [ { "sku": null, "name": "123456", "price": 100, "vatRate": null, "quantity": 1, "description": null } ],
                    "paymentLink": null, "referenceId": null,
                    "custom_fields": [ { "name": "Hobby", "type": "text", "value": "Tinker" } ],
                    "originalAmount": 10000, "refundedAmount": 0, "shippingAmount": null,
                    "paymentRequestId": null, "googleAnalyticProducts": []
                },
                "payment": { "brand": null, "wallet": null, "purchaseOnInvoiceInformation": null },
                "instance": { "name": "someapp", "uuid": "1a0bf1a3" },
                "metadata": {}, "pageUuid": null, "payoutUuid": null, "payrexxFee": 0,
                "refundable": false, "referenceId": null, "subscription": null,
                "posSerialNumber": "", "posTerminalName": "", "partiallyRefundable": false
            }
        })
    ),
    responses(
        (status = 200, description = "Webhook received and acknowledged successfully"),
        (status = 400, description = "Bad Request (e.g., invalid signature, malformed payload)"),
        (status = 500, description = "Internal Server Error (failed to process webhook)")
    ),
    tag = "Payrexx"
)]
fn doc_payrexx_webhook_handler() {}

// --- Doc functions for Redirects ---
#[utoipa::path(
    get,
    path = "/payrexx/webhook/success", // Path relative to /api
    params(RedirectQuery),
    responses(
        (status = 200, description = "Success page shown to user after payment", content_type = "text/html")
    ),
    tag = "Payrexx Redirects"
)]
fn doc_payrexx_success_handler() {}

#[utoipa::path(
    get,
    path = "/payrexx/webhook/failure", // Path relative to /api
    params(RedirectQuery),
    responses(
        (status = 200, description = "Failure page shown to user after payment", content_type = "text/html")
    ),
    tag = "Payrexx Redirects"
)]
fn doc_payrexx_failure_handler() {}

#[utoipa::path(
    get,
    path = "/payrexx/webhook/cancel", // Path relative to /api
    params(RedirectQuery),
    responses(
        (status = 200, description = "Cancellation page shown to user", content_type = "text/html")
    ),
    tag = "Payrexx Redirects"
)]
fn doc_payrexx_cancel_handler() {}

// --- Main OpenAPI Definition ---
#[derive(OpenApi)]
#[openapi(
    paths(
        doc_create_gateway_handler,
        doc_payrexx_webhook_handler,
        doc_payrexx_success_handler,
        doc_payrexx_failure_handler,
        doc_payrexx_cancel_handler
    ),
    components(
        schemas(
            CreateGatewayRequest, CreateGatewayResponse,
            PayrexxWebhookPayload,
            RedirectQuery,
            PayrexxWebhookTransaction,
            PayrexxWebhookContact,
            PayrexxWebhookInvoice,
            PayrexxWebhookInvoiceProduct,
            PayrexxWebhookCustomField,
            PayrexxWebhookPayment,
            PayrexxWebhookInstance,
        )
    ),
    tags(
        // Define tags used above
        // (name = "Payrexx", description = "Payrexx Payment Gateway API"),
        (name = "Payrexx Redirects", description = "User-facing redirect pages for Payrexx")
    )
)]
pub struct PayrexxApiDoc;
