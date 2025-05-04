// File: crates/connectify_payrexx/src/doc.rs
#![allow(dead_code)] // Allow dead code for doc functions

#[cfg(feature = "openapi")]
use utoipa::OpenApi;
#[cfg(feature = "openapi")]
use crate::logic::{CreateGatewayRequest, CreateGatewayResponse}; // Import schemas

// Define a dummy function with the handler's attributes for utoipa
#[cfg(feature = "openapi")]
#[utoipa::path(
    post,
    path = "/api/payrexx/create-gateway",
    request_body = CreateGatewayRequest,
    responses(
        (status = 200, description = "Gateway created successfully", body = CreateGatewayResponse),
        (status = 400, description = "Bad request (e.g., invalid input)"),
        (status = 500, description = "Internal server error or Payrexx API error")
    ),
    tag = "Payrexx" // Group under Payrexx tag
)]
fn doc_create_gateway_handler() {}


#[cfg(feature = "openapi")]
#[derive(OpenApi)]
#[openapi(
    paths(
        doc_create_gateway_handler // Add the doc function here
    ),
    components(
        schemas(CreateGatewayRequest, CreateGatewayResponse) // Add request/response schemas
    ),
    tags(
        (name = "Payrexx", description = "Payrexx Payment Gateway API")
    )
)]
pub struct PayrexxApiDoc;
