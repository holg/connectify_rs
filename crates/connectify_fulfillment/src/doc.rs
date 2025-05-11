// --- File: crates/connectify_fulfillment/src/doc.rs ---

// Only compile this module if the 'openapi' feature is enabled
#![cfg(feature = "openapi")]
// Allow dead code for the dummy functions used by utoipa macros
#![allow(dead_code)]

use utoipa::OpenApi;
// Import request/response schemas from the logic module
// These structs will need to derive utoipa::ToSchema in logic.rs
use crate::logic::{GcalBookingFulfillmentRequest, FulfillmentResponse};
// use serde_json::json; // For examples

// --- Dummy function for GCal Booking Fulfillment Endpoint ---
#[utoipa::path(
    post,
    path = "/fulfill/gcal-booking", // Path relative to where this router is nested (e.g., /api)
    request_body(
        content = GcalBookingFulfillmentRequest,
        description = "Details for fulfilling a Google Calendar booking",
        example = json!({
            "start_time": "2025-06-10T10:00:00Z",
            "end_time": "2025-06-10T11:00:00Z",
            "summary": "Confirmed Booking: Project Kick-off",
            "description": "This booking was confirmed via payment.",
            "original_reference_id": "stripe_cs_123abc"
        })
    ),
    params(
        ("X-Internal-Auth-Secret" = String, Header, description = "Shared secret for internal API authentication.", example = "your_super_secret_key_here")
    ),
    responses(
        (status = 200, description = "Booking fulfilled successfully", body = FulfillmentResponse, example = json!({
            "success": true,
            "message": "Google Calendar event booked successfully.",
            "event_id": "gcal_event_id_789xyz"
        })),
        (status = 400, description = "Bad Request - Invalid payload for fulfillment"),
        (status = 401, description = "Unauthorized - Missing or invalid internal auth token", body = String, examples(
            ("MissingHeader" = (
                summary = "Missing X-Internal-Auth-Secret header",
                value = json!("Unauthorized: Missing X-Internal-Auth-Secret header.")
            )),
            ("InvalidSecret" = (
                summary = "Invalid secret provided",
                value = json!("Unauthorized: Invalid credentials.")
            ))
        )),
        (status = 409, description = "Booking conflict in Google Calendar"),
        (status = 500, description = "Internal Server Error - Fulfillment failed")
    ),
    tag = "Fulfillment" // Group this endpoint under the "Fulfillment" tag
)]
fn doc_handle_gcal_booking_fulfillment() {
    // This function body is never executed.
}

// TODO: Add doc_handle_twilio_adhoc_fulfillment() when that handler is implemented.

// --- Main OpenAPI Definition for the Fulfillment Service ---
#[derive(OpenApi)]
#[openapi(
    paths(
        doc_handle_gcal_booking_fulfillment
        // TODO: Add other doc_... functions here
    ),
    components(
        // List all schemas used in the paths
        schemas(
            GcalBookingFulfillmentRequest,
            FulfillmentResponse
            // TODO: Add other request/response schemas here
        )
    ),
    tags(
        // Define the tag used above for grouping endpoints
        (name = "Fulfillment", description = "Internal Fulfillment Service API")
    )
    // No 'servers' needed here, as this will be merged into the main backend's ApiDoc
)]
pub struct FulfillmentApiDoc;
