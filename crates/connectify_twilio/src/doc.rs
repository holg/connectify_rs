// --- File: crates/connectify_twilio/src/doc.rs ---

// Only compile this module if the 'openapi' feature is enabled
#![cfg(feature = "openapi")]
// Allow dead code for the dummy function used by the macro
#![allow(dead_code)]

use utoipa::OpenApi;
// Import the request query and response structs from the twilio_token module
// Ensure these structs derive utoipa::ToSchema (needs to be added there)
use crate::twilio_token::{TokenRequestQuery, TokenResponse};

// Define a dummy function with the utoipa::path macro to document the endpoint
#[utoipa::path(
    get, // HTTP method
    path = "/generate-token", // The actual path registered in main.rs
    // Describe query parameters.
    // Note: TokenRequestQuery needs #[derive(utoipa::IntoParams)] for this to work directly,
    // or list params manually as below.
    params(
        ("identity" = String, Query, description = "User identity for the token", example = "User_12345"),
        ("roomName" = String, Query, description = "Room name for the token", example = "MyCoolRoom")
    ),
    responses(
        (status = 200, description = "Successfully generated Twilio Access Token", body = TokenResponse),
        (status = 500, description = "Token generation failed due to server error", body = String, example = json!("Failed to generate token")),
        (status = 503, description = "Twilio service disabled by configuration", body = String, example = json!("Twilio service is disabled by configuration."))
    ),
    tag = "Twilio" // Group this endpoint under the "Twilio" tag in Swagger UI
)]
fn doc_generate_token() {
    // This function body is never executed, it's just an anchor for the macro.
}


// Define the main OpenAPI documentation structure for this crate/feature
#[derive(OpenApi)]
#[openapi(
    paths(
        doc_generate_token // List the path-documenting functions
    ),
    components(
        // List all schemas used in the paths (request/response bodies, parameters)
        schemas(TokenRequestQuery, TokenResponse)
    ),
    tags(
        // Define the tag used above for grouping endpoints
        (name = "Twilio", description = "Twilio Token Generation API")
    )
    // No servers needed here, defined in the main backend doc
)]
pub struct TwilioApiDoc;
