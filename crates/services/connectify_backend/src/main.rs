// File: services/connectify_backend/src/main.rs
use axum::{routing::get, Router};
#[cfg(feature = "adhoc")]
use connectify_adhoc;
#[allow(unused_imports)]
use connectify_common::{is_feature_enabled, logging};
use connectify_config::load_config;
#[cfg(feature = "gcal")]
use connectify_gcal;
#[cfg(feature = "twilio")]
use connectify_twilio;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
#[allow(unused_imports)]
use tracing::{info, warn}; // we shall add more warns, even so now only some feature uses it right nwo
                           // use axum::{extract::State, Json};

// Import the AppState and AppStateBuilder from the app_state module
mod app_state;
use app_state::AppState;

// #[axum::debug_handler]
// async fn show_config(State(config): State<Arc<AppConfig>>) -> Json<AppConfig> {
//     Json(config.as_ref().clone())
// }

#[tokio::main]
async fn main() {
    // Initialize logging with default level (INFO)
    logging::init();

    info!("Starting Connectify backend service");

    let config = Arc::new(load_config().expect("Failed to load config"));
    info!("✅ Configuration loaded.");

    // Create the AppState with the config
    // This will initialize all services based on the configuration
    #[allow(unused_variables)]
    let app_state = AppState::new(config.clone()).await;
    #[allow(unused_mut)]
    let mut api_router =
        Router::new().route("/", get(|| async { "Welcome to Connectify-Rs API!" }));
    // .route("/config", get(show_config))
    // .with_state(config.clone()); we manage the State now with app_state
    // Conditionally merge Twilio routes
    #[cfg(feature = "twilio")]
    {
        if is_feature_enabled(&config, config.use_twilio, config.twilio.as_ref()) {
            info!("🔌 Merging Twilio routes...");
            // Twilio routes take Arc<AppConfig> as state
            api_router = api_router.merge(connectify_twilio::routes::routes(config.clone()));
        }
    }
    // Conditionally merge GCal routes
    #[cfg(feature = "gcal")]
    {
        // GCal routes take Arc<GcalState> as state
        #[allow(unused_variables)]
        if let Some(gcal_state_ref) = app_state.gcal_state.as_ref() {
            // Check if GcalState was initialized
            if is_feature_enabled(&config, config.use_gcal, config.gcal.as_ref()) {
                // Check runtime flags
                info!("🔌 Merging GCal routes...");
                api_router =
                    api_router.merge(connectify_gcal::routes::routes(config.clone()).await);
            }
        } else if is_feature_enabled(&config, config.use_gcal, config.gcal.as_ref()) {
            // Log if enabled but state failed
            warn!("ℹ️ GCal routes not merged (GCal state initialization failed).");
        }
    }
    // Conditionally merge Payrexx routes
    #[cfg(feature = "payrexx")]
    {
        if is_feature_enabled(&config, config.use_payrexx, config.payrexx.as_ref()) {
            info!("🔌 Merging Payrexx routes...");
            // Payrexx routes take Arc<AppConfig> as state
            api_router = api_router.merge(connectify_payrexx::routes(config.clone()));
        }
    }
    // Conditionally merge Stripe routes
    #[cfg(feature = "stripe")]
    {
        if is_feature_enabled(&config, config.use_stripe, config.stripe.as_ref()) {
            info!("🔌 Merging Stripe routes...");
            // Stripe routes take Arc<AppConfig> as state
            api_router = api_router.merge(connectify_stripe::routes(config.clone()));
        }
    }
    // Conditionally merge Fulfillment routes
    #[cfg(feature = "fulfillment")]
    {
        if is_feature_enabled(&config, config.use_fulfillment, config.fulfillment.as_ref()) {
            info!("🔌 Merging Fulfillment routes...");
            // Fulfillment routes take Arc<AppConfig> and potentially other states like GcalState
            #[cfg(not(feature = "gcal"))]
            {
                // When gcal feature is not enabled, call with just one argument
                api_router = api_router.merge(connectify_fulfillment::routes(config.clone()));
            }
            #[cfg(feature = "gcal")]
            {
                info!("🔌 Merging GCal Fulfillment routes...");
                api_router = api_router.merge(connectify_fulfillment::routes(
                    config.clone(),
                    app_state.gcal_state.clone(),
                ));
            }
        }
    }

    // Conditionally merge Adhoc routes
    #[cfg(feature = "adhoc")]
    {
        if is_feature_enabled(&config, config.use_adhoc, config.adhoc_settings.as_ref()) {
            info!("🔌 Merging Adhoc routes...");
            // Adhoc routes take Arc<AppConfig> and potentially GcalState
            #[cfg(not(feature = "gcal"))]
            {
                // When gcal feature is not enabled, call with just one argument
                api_router = api_router.merge(connectify_adhoc::routes(config.clone()));
            }
            // When both adhoc and gcal features are enabled, but not adhoc_gcal
            #[cfg(feature = "gcal")]
            {
                info!("🔌 Merging GCal Adhoc routes...");
                let gcal_hub_option = app_state
                    .gcal_state
                    .as_ref()
                    .map(|state| state.calendar_hub.clone());
                api_router =
                    api_router.merge(connectify_adhoc::routes(config.clone(), gcal_hub_option));
            }
        }
    }

    // --- Create Main App Router ---
    // Nest all API routes under /api
    let mut app = Router::new().nest("/api", api_router);

    // Conditionally add Swagger UI and JSON endpoint if openapi feature enabled
    #[cfg(feature = "openapi")]
    {
        #[cfg(feature = "adhoc")]
        use connectify_adhoc::doc::AdhocApiDoc;
        #[cfg(feature = "fulfillment")]
        use connectify_fulfillment::doc::FulfillmentApiDoc;
        #[cfg(feature = "gcal")]
        use connectify_gcal::doc::GcalApiDoc;
        #[cfg(feature = "payrexx")]
        use connectify_payrexx::doc::PayrexxApiDoc;
        #[cfg(feature = "stripe")]
        use connectify_stripe::doc::StripeApiDoc;
        #[cfg(feature = "twilio")]
        use connectify_twilio::doc::TwilioApiDoc;
        use utoipa::OpenApi;
        use utoipa_swagger_ui::SwaggerUi;

        // Define the Merged OpenAPI Documentation struct
        #[derive(OpenApi)]
        #[openapi(
            info(
                title = "Connectify API",
                version = "0.1.0",
                description = "Connectify Service API Docs",
                license(name = "MIT", url = "https://opensource.org/licenses/MIT")
            ),
            components(),
            tags( (name = "Connectify", description = "Core service endpoints")),
            servers( (url = "/api", description = "Main API Prefix")),
        )]
        struct ApiDoc;

        // Create the merged OpenAPI document
        #[allow(unused_mut)] // for the features it needs to be mutable
        let mut openapi_doc = ApiDoc::openapi();
        #[cfg(feature = "gcal")]
        openapi_doc.merge(GcalApiDoc::openapi());
        #[cfg(feature = "twilio")]
        openapi_doc.merge(TwilioApiDoc::openapi());
        #[cfg(feature = "stripe")]
        openapi_doc.merge(StripeApiDoc::openapi());
        #[cfg(feature = "fulfillment")]
        openapi_doc.merge(FulfillmentApiDoc::openapi());
        #[cfg(feature = "payrexx")]
        openapi_doc.merge(PayrexxApiDoc::openapi());
        #[cfg(all(feature = "adhoc", feature = "openapi"))]
        openapi_doc.merge(AdhocApiDoc::openapi());
        info!("📖 Adding Swagger UI at /admin/api/docs");

        // Create the Swagger UI route, referencing the merged doc
        let swagger_ui = SwaggerUi::new("/admin/api/docs")
            .url("/admin/api/docs/openapi.json", openapi_doc.clone());
        // Merge the Swagger UI into the main app router
        app = app.merge(swagger_ui);
    }

    // Serve static files in dev mode
    if cfg!(debug_assertions) {
        info!("Running in development mode, serving static files from ../../dist");

        // Serve static files at a specific path
        let static_router = Router::new().nest_service("/static", ServeDir::new("../../dist"));
        app = app.merge(static_router);

        // You can also keep the fallback service for non-matched routes
        app = app.fallback_service(ServeDir::new("../dist"));
    }

    // 6. Bind and serve
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    info!("Starting server at http://{}", addr);
    info!("API endpoints available at http://{}/api", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
