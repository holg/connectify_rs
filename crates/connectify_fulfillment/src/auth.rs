// --- File: crates/connectify_fulfillment/src/auth.rs ---

use axum::{
    extract::State,
    body::Body as AxumBody,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use connectify_config::AppConfig; // To access the shared secret
use constant_time_eq::constant_time_eq; // For secure string comparison

// The state that this auth middleware will have access to.
// It needs the AppConfig to get the shared secret.
#[derive(Clone)]
pub struct FulfillmentAuthState {
    pub config: Arc<AppConfig>,
}

const INTERNAL_AUTH_HEADER: &str = "X-Internal-Auth-Secret";

/// Axum middleware to authenticate internal fulfillment requests.
/// Checks for a shared secret in the `X-Internal-Auth-Secret` header.
pub async fn fulfillment_auth_middleware<B>( // B is the request body type
                                             State(auth_state): State<Arc<FulfillmentAuthState>>, // Specific state for this middleware
                                             req: Request<AxumBody>, // Request is generic over B
                                             next: Next,      // Next is not generic
) -> Response // Changed to always return Response
where
    B: Send + 'static, // Add bound for B as Request<B> is passed to next.run()
{

    // 1. Get the expected shared secret from config as &str
    // Ensure the fulfillment config and shared_secret field exist in AppConfig
    let expected_secret_as_str: String = match auth_state.config.fulfillment.as_ref().and_then(|f_cfg| f_cfg.shared_secret.clone()) {
        Some(secret) => secret,
        None => {
            eprintln!("🚨 Fulfillment shared secret not configured in AppConfig!");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Server configuration error for fulfillment auth.".to_string()).into_response();
        }
    };



    // 2. Get the token from the request header as Option<&str>
    let provided_secret_as_str_opt: Option<&str> = req.headers()
        .get(INTERNAL_AUTH_HEADER)
        .and_then(|value| value.to_str().ok());

    // 3. Validate the token
    match provided_secret_as_str_opt {
        Some(provided_secret_val) => { // provided_secret_val is &str
            // Both are now &str, so .as_bytes() will work correctly
            if constant_time_eq(provided_secret_val.as_bytes(), expected_secret_as_str.as_bytes()) {
                // Token is valid, proceed to the next handler
                println!("✅ Fulfillment request authenticated successfully.");
                next.run(req).await // This already returns a Response
            } else {
                eprintln!("🚨 Fulfillment request: Invalid secret provided.");
                (StatusCode::UNAUTHORIZED, "Unauthorized: Invalid credentials.".to_string()).into_response()
            }
        }
        None => {
            eprintln!("🚨 Fulfillment request: Missing '{}' header.", INTERNAL_AUTH_HEADER);
            (StatusCode::UNAUTHORIZED, format!("Unauthorized: Missing {} header.", INTERNAL_AUTH_HEADER)).into_response()
        }
    }
}
