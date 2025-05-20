// --- File: crates/connectify_payrexx/src/auth.rs ---
// use axum::{
//     extract::{State}, // Use State extractor
//     response::{Json},
//     http::StatusCode,
// };

// use std::sync::Arc;
// use connectify_config::AppConfig; // Use the unified config from the config crate
// use crate::logic::{
//     create_gateway_request, CreateGatewayRequest, CreateGatewayResponse, PayrexxError // Import logic function and types
// };

// Define the shared application state struct expected by this handler
// This should match the AppState defined in connectify_backend/src/main.rs
// Or you can extract fields directly if AppState is guaranteed to contain them.
// For simplicity, let's assume AppState is passed directly.
// Ensure AppState derives Clone.
// use crate::AppState; // Assuming AppState is defined in a shared location or backend
//
// /// Axum handler to create a Payrexx payment gateway.
// #[axum::debug_handler] // Add debug handler for better error messages during development
// #[cfg_attr(feature = "openapi", utoipa::path( // Keep OpenAPI docs if using that feature
//     post,
//     path = "/api/payrexx/create-gateway",
//     request_body = CreateGatewayRequest,
//     responses(
//         (status = 200, description = "Gateway created successfully", body = CreateGatewayResponse),
//         (status = 400, description = "Bad request (e.g., invalid input)"),
//         (status = 500, description = "Internal server error or Payrexx API error")
//     ),
//     tag = "Payrexx"
// ))]
// pub async fn create_gateway_handler(
//     // Extract the full shared AppState
//     State(app_state): State<Arc<AppConfig>>, // Assuming AppConfig is the top-level state for now
//     // If using a nested AppState: State(state): State<Arc<AppState>>,
//     Json(payload): Json<CreateGatewayRequest>, // Extract JSON body
// ) -> Result<Json<CreateGatewayResponse>, (StatusCode, String)> {
//
//     // Check the runtime flag from the shared config
//     if !app_state.use_payrexx {
//         return Err((StatusCode::SERVICE_UNAVAILABLE, "Payrexx service is disabled.".to_string()));
//     }
//
//     if let Some(payrexx_config) = app_state.payrexx.as_ref() {
//         match create_gateway_request(payrexx_config, payload).await {
//             Ok(response) => Ok(Json(response)), // Return 200 OK with JSON body
//             Err(PayrexxError::ConfigError) => {
//                 Err((StatusCode::INTERNAL_SERVER_ERROR, "Payrexx configuration error on server.".to_string()))
//             }
//             Err(PayrexxError::RequestError(e)) => {
//                 info!("Payrexx Reqwest Error: {}", e);
//                 Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to communicate with payment provider.".to_string()))
//             }
//             Err(PayrexxError::ParseError(e)) => {
//                 info!("Payrexx Parse Error: {}", e);
//                 Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to understand payment provider response.".to_string()))
//             }
//             Err(PayrexxError::ApiError { status, message }) => {
//                 // Could potentially map specific Payrexx errors to different HTTP statuses if needed
//                 Err((StatusCode::BAD_GATEWAY, format!("Payment provider error: {} - {}", status, message)))
//             }
//             Err(PayrexxError::InternalError(msg)) => {
//                 Err((StatusCode::INTERNAL_SERVER_ERROR, msg))
//             }
//         }
//
//     } else {
//         // This case means use_payrexx was true, but config loading failed earlier
//         Err((StatusCode::INTERNAL_SERVER_ERROR, "Payrexx configuration error (details missing).".to_string()))
//     }
// }
//
