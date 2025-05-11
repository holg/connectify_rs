// File: services/connectify_backend/src/main.rs
use axum::{routing::get, Router};
use connectify_config::load_config;
#[cfg(feature = "twilio")]
use connectify_twilio;
#[cfg(feature = "gcal")]
use connectify_gcal::{self, handlers::GcalState, auth::create_calendar_hub};
// #[cfg(feature = "stripe")]
// use connectify_stripe::routes as stripe_routes;
// #[cfg(feature = "fulfillment")]
// use connectify_fulfillment::routes as fulfillment_routes;
// #[cfg(feature = "payrexx")]
// use connectify_payrexx::routes as payrexx_routes;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use connectify_config::AppConfig;
// use axum::{extract::State, Json};


#[derive(Clone)]
struct AppState {
    #[allow(dead_code)]
    config: Arc<AppConfig>,
    #[cfg(feature = "gcal")]
    gcal_state: Option<Arc<GcalState>>, // This state is specific to GCal routes
    // Add other feature-specific states here if needed by their routes
    // e.g., http_client for features that need a shared reqwest client
}




// #[axum::debug_handler]
// async fn show_config(State(config): State<Arc<AppConfig>>) -> Json<AppConfig> {
//     Json(config.as_ref().clone())
// }



#[tokio::main]
async fn main() {
    let config = Arc::new(load_config().expect("Failed to load config"));
    println!("✅ Configuration loaded.");

    // --- Initialize Feature-Specific State Conditionally ---
    #[allow(unused_mut)] // GCal state might not be used if feature is off
    let mut gcal_state_instance: Option<Arc<GcalState>> = None;
    #[cfg(feature = "gcal")]
    {
        if config.use_gcal && config.gcal.is_some() {
            println!("ℹ️ Initializing Google Calendar client...");
            match create_calendar_hub(config.gcal.as_ref().unwrap()).await {
                Ok(hub) => {
                    gcal_state_instance = Some(Arc::new(GcalState {
                        config: config.clone(),
                        calendar_hub: Arc::new(hub),
                    }));
                    println!("✅ Google Calendar client initialized.");
                }
                Err(e) => {
                    eprintln!("🚨 Failed to initialize Google Calendar client: {}. GCal routes disabled.", e);
                }
            }
        } else {
            println!("ℹ️ GCal feature compiled, but disabled via runtime config or missing gcal config section.");
        }
    }

    // Create the final AppState
    let app_state = AppState {
        config: config.clone(),
        #[cfg(feature = "gcal")]
        gcal_state: gcal_state_instance.clone(),
    };

    let mut api_router = Router::new()
        .route("/", get(|| async { "Welcome to Connectify-Rs API!" }));
        // .route("/config", get(show_config))
        // .with_state(config.clone()); we manage the State now with app_state
    // Conditionally merge Twilio routes
    #[cfg(feature = "twilio")]
    {
        if config.use_twilio && config.twilio.is_some() {
            println!("🔌 Merging Twilio routes...");
            // Twilio routes take Arc<AppConfig> as state
            api_router = api_router.merge(connectify_twilio::routes::routes(config.clone()));
        }
    }
    // Conditionally merge GCal routes
    #[cfg(feature = "gcal")]
    {
        // GCal routes take Arc<GcalState> as state
        #[allow(unused_variables)]
        if let Some(gcal_state_ref) = app_state.gcal_state.as_ref() { // Check if GcalState was initialized
            if config.use_gcal && config.gcal.is_some() { // Also check runtime flags
                println!("🔌 Merging GCal routes...");
                api_router = api_router.merge(connectify_gcal::routes::routes(config.clone()).await);
            }
        } else if config.use_gcal && config.gcal.is_some() { // Log if enabled but state failed
            println!("ℹ️ GCal routes not merged (GCal state initialization failed).");
        }
    }
    // Conditionally merge Payrexx routes
    #[cfg(feature = "payrexx")]
    {
        if config.use_payrexx && config.payrexx.is_some() {
            println!("🔌 Merging Payrexx routes...");
            // Payrexx routes take Arc<AppConfig> as state
            api_router = api_router.merge(connectify_payrexx::routes(config.clone()));
        }
    }
    // Conditionally merge Stripe routes
    #[cfg(feature = "stripe")]
    {
        if config.use_stripe && config.stripe.is_some() {
            println!("🔌 Merging Stripe routes...");
            // Stripe routes take Arc<AppConfig> as state
            api_router = api_router.merge(connectify_stripe::routes(config.clone()));
        }
    }
    // Conditionally merge Fulfillment routes
    #[cfg(feature = "fulfillment")]
    {
        if config.use_fulfillment && config.fulfillment.is_some() {
            println!("🔌 Merging Fulfillment routes...");
            // Fulfillment routes take Arc<AppConfig> and potentially other states like GcalState
            #[cfg(not(feature = "gcal"))]
            {
                // When gcal feature is not enabled, call with just one argument
                api_router = api_router.merge(connectify_fulfillment::routes(
                    config.clone(),
                ));
            }
            #[cfg(feature = "gcal")]
            {
                println!("🔌 Merging GCal Fulfillment routes...");
                api_router = api_router.merge(connectify_fulfillment::routes(
                    config.clone(),
                    app_state.gcal_state,
                ));
            }



        }
    }

    // --- Create Main App Router ---
    // Nest all API routes under /api
    let mut app = Router::new().nest("/api", api_router);

    // Conditionally add Swagger UI and JSON endpoint if openapi feature enabled
    #[cfg(feature = "openapi")]
    {
        #[cfg(feature = "twilio")]
        use connectify_twilio::doc::TwilioApiDoc;
        #[cfg(feature = "gcal")]
        use connectify_gcal::doc::GcalApiDoc;
        #[cfg(feature = "stripe")]
        use connectify_stripe::doc::StripeApiDoc;
        #[cfg(feature = "fulfillment")]
        use connectify_fulfillment::doc::FulfillmentApiDoc;
        #[cfg(feature = "payrexx")]
        use connectify_payrexx::doc::PayrexxApiDoc;
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
        println!("📖 Adding Swagger UI at /api/docs");

        // Create the Swagger UI route, referencing the merged doc
        let swagger_ui =
            SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", openapi_doc.clone());
        // Merge the Swagger UI into the main app router
        app = app.merge(swagger_ui);
    }

    // Serve static files in dev mode
    if cfg!(debug_assertions) {
        println!("Running in development mode, serving static files from ../../dist");

        // Serve static files at a specific path
        let static_router = Router::new().nest_service("/static", ServeDir::new("../../dist"));
        app = app.merge(static_router);

        // You can also keep the fallback service for non-matched routes
        app = app.fallback_service(ServeDir::new("../dist"));
    }

    // 6. Bind and serve
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Starting server at http://{}", addr);
    println!("API endpoints available at http://{}/api", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
