// --- File: crates/connectify_adhoc/src/doc.rs ---
#![allow(dead_code)]
use utoipa::OpenApi;
// use serde_json::json;
// Import all relevant schemas from logic.rs and handlers.rs
use crate::logic::{
    InitiateAdhocSessionRequest,
    InitiateAdhocSessionResponse,
    // AdhocSessionError
};

/// Documentation for the initiate_adhoc_session_handler endpoint
/// This endpoint allows users to initiate an ad-hoc session with a specified duration.
/// It checks availability, creates a Stripe checkout session, and returns session details.
#[utoipa::path(
    post,
    path = "/adhoc/initiate-session", // Path relative to /api
    request_body(content = InitiateAdhocSessionRequest, example = json!({
        "duration_minutes": 30
    })),
    responses(
        (status = 200, description = "Adhoc session initiated, Stripe URL returned", body = InitiateAdhocSessionResponse),
        (status = 400, description = "Invalid request (e.g., bad duration)"),
        (status = 403, description = "Adhoc sessions admin-disabled"),
        (status = 409, description = "Slot unavailable"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Adhoc Sessions"
)]
fn doc_initiate_adhoc_session_handler() {}

/// OpenAPI documentation for the Adhoc Sessions API
#[derive(OpenApi)]
#[openapi(
    paths(
        doc_initiate_adhoc_session_handler
    ),
    components(
        schemas(
            InitiateAdhocSessionRequest,
            InitiateAdhocSessionResponse
        )
    ),
    tags(
        (name = "Adhoc Sessions", description = "API for initiating and managing ad-hoc sessions")
    )
)]
pub struct AdhocApiDoc;
