// File: crates/connectify_gcal/src/doc.rs

#![allow(dead_code)]
#![cfg(feature = "openapi")]
use crate::logic::BookedEventsResponse;
use utoipa::OpenApi;
use utoipa;

use crate::logic::{
    AvailabilityQuery, AvailableSlotsResponse, BookSlotRequest, BookedEvent, BookedEventsQuery,
    BookingResponse, CancelBookingRequest, CancellationResponse,
};
#[utoipa::path(
    get,
    path = "/availability",
    params(
        ("start_date" = String, Query, description = "Start date in YYYY-MM-DD format", example = "2025-05-05", format="date"),
        ("end_date" = String, Query, description = "End date in YYYY-MM-DD format", example = "2025-05-24", format="date"),
        ("duration_minutes" = i64, Query, description = "Duration in minutes", example = 60)
    ),
    responses(
        (status = 200, description = "Available time slots", body = AvailableSlotsResponse),
        (status = 500, description = "Internal error", body = String)
    )
)]
fn doc_get_availability_handler() {}

#[utoipa::path(
    post,
    path = "/book",
    request_body(content = BookSlotRequest, example = json!({
        "start_time": "2025-05-15T10:00:00Z",
        "end_time": "2025-05-15T11:00:00Z",
        "summary": "Meeting with Client",
        "description": "Discussion about the new project requirements"
    })),
    responses(
        (status = 200, description = "Booking result", body = BookingResponse,
         example = json!({
             "success": true,
             "event_id": "abc123xyz456",
             "message": "Appointment booked successfully."
         })
        ),
        (status = 409, description = "Slot already booked",
         example = json!({
             "success": false,
             "event_id": null,
             "message": "Requested time slot is no longer available."
         })
        ),
        (status = 500, description = "Booking failed",
         example = json!({
             "success": false,
             "event_id": null,
             "message": "Failed to book appointment."
         })
        )
    )
)]
fn doc_book_slot_handler() {}

#[utoipa::path(
    delete,
    path = "admin/delete/{event_id}",
    params(
        ("event_id" = String, Path, description = "The ID of the event to cancel"),
        ("notify_attendees" = bool, Query, description = "Whether to send cancellation notifications to attendees")
    ),
    responses(
        (status = 200, description = "Cancellation result", body = CancellationResponse,
         example = json!({
             "success": true,
             "message": "Appointment deleted successfully."
         })
        ),
        (status = 404, description = "Event not found",
         example = json!({
             "success": false,
             "message": "Event not found."
         })
        ),
        (status = 500, description = "Cancellation failed",
         example = json!({
             "success": false,
             "message": "Failed to cancel appointment."
         })
        )
    )
)]
fn doc_cancel_booking_handler() {}

#[utoipa::path(
    get,
    path = "/admin/bookings",
    params(
        ("start_date" = String, Query, description = "Start date in YYYY-MM-DD format", example = "2025-05-15", format="date"),
        ("end_date" = String, Query, description = "End date in YYYY-MM-DD format", example = "2025-05-20", format="date"),
        ("include_cancelled" = bool, Query, description = "Whether to include cancelled events", example = false)
    ),
    responses(
        (status = 200, description = "List of booked events", body = BookedEventsResponse,
         example = json!({
             "events": [
                 {
                     "event_id": "abc123xyz456",
                     "summary": "Meeting with Client",
                     "description": "Discussion about the new project requirements",
                     "start_time": "2025-05-15T10:00:00Z",
                     "end_time": "2025-05-15T11:00:00Z",
                     "status": "confirmed",
                     "created": "2025-05-10T09:00:00Z",
                     "updated": "2025-05-10T09:00:00Z"
                 }
             ]
         })
        ),
        (status = 400, description = "Invalid date format",
         example = json!("Invalid start_date format (YYYY-MM-DD)")
        ),
        (status = 500, description = "Failed to fetch events",
         example = json!("Failed to fetch booked events")
        )
    )
)]
fn doc_get_booked_events_handler() {}

#[utoipa::path(
    patch,
    path = "/admin/mark_cancelled/{event_id}",
    params(
        ("event_id" = String, Path, description = "The ID of the event to mark as cancelled"),
        ("notify_attendees" = bool, Query, description = "Whether to send cancellation notifications to attendees")
    ),
    responses(
        (status = 200, description = "Cancellation result", body = CancellationResponse,
         example = json!({
             "success": true,
             "message": "Appointment marked as cancelled successfully."
         })
        ),
        (status = 404, description = "Event not found",
         example = json!({
             "success": false,
             "message": "Event not found."
         })
        ),
        (status = 500, description = "Marking as cancelled failed",
         example = json!({
             "success": false,
             "message": "Failed to mark appointment as cancelled."
         })
        )
    ),
)]
fn doc_mark_booking_cancelled_handler() {}

#[derive(OpenApi)]
#[openapi(
    paths(
        doc_get_availability_handler,
        doc_book_slot_handler,
        doc_cancel_booking_handler,
        doc_get_booked_events_handler,
        doc_mark_booking_cancelled_handler
    ),
    components(
        schemas(
            AvailabilityQuery,
            AvailableSlotsResponse,
            BookSlotRequest,
            BookingResponse,
            CancelBookingRequest,
            CancellationResponse,
            BookedEventsQuery,
            BookedEvent,
            BookedEventsResponse
        )
    ),
    tags(
        (name = "gcal", description = "Google Calendar Booking API")
    ),
    servers(
        (url = "/api", description = "Google Calendar API server")
    )
)]
pub struct GcalApiDoc;
