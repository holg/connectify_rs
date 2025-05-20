// --- File: crates/connectify_common/src/http.rs ---
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::error::{ConnectifyError, HttpStatusCode};

// Include the client module
pub mod client;

/// Extension trait for ConnectifyError to convert it to an Axum HTTP response.
pub trait IntoHttpResponse {
    /// Converts the error into an Axum HTTP response.
    fn into_http_response(self) -> Response;
}

impl IntoHttpResponse for ConnectifyError {
    fn into_http_response(self) -> Response {
        let status_code =
            StatusCode::from_u16(self.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let error_message = self.to_string();

        // Create a JSON response with the error message
        let body = Json(json!({
            "error": {
                "message": error_message,
                "code": status_code.as_u16(),
            }
        }));

        // Combine the status code and body into a response
        (status_code, body).into_response()
    }
}

/// Implement IntoResponse for ConnectifyError to make it easier to use in Axum handlers.
impl IntoResponse for ConnectifyError {
    fn into_response(self) -> Response {
        self.into_http_response()
    }
}

/// A utility function to convert a Result<T, ConnectifyError> to a Result<T, Response>.
/// This is useful for Axum handlers that return a Result.
pub fn handle_result<T>(result: Result<T, ConnectifyError>) -> Result<T, Response>
where
    T: IntoResponse,
{
    result.map_err(|err| err.into_response())
}

/// A utility function to convert a Result<Json<T>, ConnectifyError> to a Result<Json<T>, Response>.
/// This is useful for Axum handlers that return a JSON response.
pub fn handle_json_result<T>(result: Result<T, ConnectifyError>) -> Result<Json<T>, Response>
where
    T: serde::Serialize,
{
    result.map(Json).map_err(|err| err.into_response())
}

/// A utility function to convert a Result<T, E> to a Result<T, Response> using a custom error mapper.
/// This is useful for Axum handlers that need to convert domain-specific errors to HTTP responses.
pub fn map_error<T, E, F>(result: Result<T, E>, f: F) -> Result<T, Response>
where
    T: IntoResponse,
    F: FnOnce(E) -> ConnectifyError,
{
    result.map_err(|err| f(err).into_response())
}

/// A utility function to convert a Result<T, E> to a Result<Json<T>, Response> using a custom error mapper.
/// This is useful for Axum handlers that need to convert domain-specific errors to HTTP responses.
pub fn map_json_error<T, E, F>(result: Result<T, E>, f: F) -> Result<Json<T>, Response>
where
    T: serde::Serialize,
    F: FnOnce(E) -> ConnectifyError,
{
    result.map(Json).map_err(|err| f(err).into_response())
}
