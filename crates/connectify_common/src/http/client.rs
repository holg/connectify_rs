    // --- File: crates/connectify_common/src/http/client.rs ---
use once_cell::sync::Lazy;
use reqwest::{Client, Error as ReqwestError, Response};
use std::time::Duration;

/// Default timeout for HTTP requests in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// A static HTTP client that can be reused across the application.
/// This client is configured with a default timeout and follows redirects.
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .expect("Failed to create HTTP client")
});

/// Creates a new HTTP client with custom configuration.
///
/// # Arguments
///
/// * `timeout_secs` - The timeout in seconds for the client
/// * `follow_redirects` - Whether the client should follow redirects
///
/// # Returns
///
/// A new reqwest::Client instance with the specified configuration
pub fn create_client(timeout_secs: u64, follow_redirects: bool) -> Result<Client, ReqwestError> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .redirect(if follow_redirects {
            reqwest::redirect::Policy::default()
        } else {
            reqwest::redirect::Policy::none()
        })
        .build()
}

/// A utility function to make a GET request to the specified URL.
///
/// # Arguments
///
/// * `url` - The URL to make the request to
///
/// # Returns
///
/// A Result containing the Response or an Error
pub async fn get(url: &str) -> Result<Response, ReqwestError> {
    HTTP_CLIENT.get(url).send().await
}

/// A utility function to make a POST request to the specified URL with the specified body.
///
/// # Arguments
///
/// * `url` - The URL to make the request to
/// * `body` - The body of the request
///
/// # Returns
///
/// A Result containing the Response or an Error
pub async fn post<T: serde::Serialize>(url: &str, body: &T) -> Result<Response, ReqwestError> {
    HTTP_CLIENT.post(url).json(body).send().await
}

/// A utility function to make a PUT request to the specified URL with the specified body.
///
/// # Arguments
///
/// * `url` - The URL to make the request to
/// * `body` - The body of the request
///
/// # Returns
///
/// A Result containing the Response or an Error
pub async fn put<T: serde::Serialize>(url: &str, body: &T) -> Result<Response, ReqwestError> {
    HTTP_CLIENT.put(url).json(body).send().await
}

/// A utility function to make a DELETE request to the specified URL.
///
/// # Arguments
///
/// * `url` - The URL to make the request to
///
/// # Returns
///
/// A Result containing the Response or an Error
pub async fn delete(url: &str) -> Result<Response, ReqwestError> {
    HTTP_CLIENT.delete(url).send().await
}

/// A utility function to make a PATCH request to the specified URL with the specified body.
///
/// # Arguments
///
/// * `url` - The URL to make the request to
/// * `body` - The body of the request
///
/// # Returns
///
/// A Result containing the Response or an Error
pub async fn patch<T: serde::Serialize>(url: &str, body: &T) -> Result<Response, ReqwestError> {
    HTTP_CLIENT.patch(url).json(body).send().await
}
