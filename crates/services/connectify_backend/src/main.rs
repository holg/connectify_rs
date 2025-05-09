// File: services/connectify_backend/src/main.rs
use axum::{routing::get, Router};
use connectify_config::load_config;
#[cfg(feature = "gcal")]
use connectify_gcal::routes as gcal_routes;
#[cfg(feature = "payrexx")]
use connectify_payrexx::routes as payrexx_routes;
#[cfg(feature = "stripe")]
use connectify_stripe::routes as stripe_routes;
#[cfg(feature = "twilio")]
use connectify_twilio::routes as twilio_routes;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use connectify_config::AppConfig;
use axum::{extract::State, Json};
#[axum::debug_handler]
async fn show_config(State(config): State<Arc<AppConfig>>) -> Json<AppConfig> {
    Json(config.as_ref().clone())
}

#[tokio::main]
async fn main() {
    let config = Arc::new(load_config().expect("Failed to load config"));

    let api_router = Router::new()
        .route("/", get(|| async { "Welcome to Connectify-Rs API!" }))
        // .route("/config", get(show_config))
        .with_state(config.clone());
    #[cfg(feature = "twilio")]
    let twilio_router = twilio_routes::routes(config.clone());
    #[cfg(feature = "payrexx")]
    let payrexx_router = payrexx_routes::routes(config.clone());
    #[cfg(feature = "stripe")]
    let stripe_router = stripe_routes::routes(config.clone());
    #[cfg(feature = "gcal")]
    let gcal_router = gcal_routes::routes(config.clone()).await;

    let api_router = Router::new().nest("/api", {
        #[allow(unused_mut)] // for the features it needs to be mutable
        let mut router = api_router;
        #[cfg(feature = "twilio")]
        {
            router = router.merge(twilio_router);
        }
        #[cfg(feature = "gcal")]
        {
            router = router.merge(gcal_router);
        }
        #[cfg(feature = "stripe")]
        {
            router = router.merge(stripe_router);
        }
        #[cfg(feature = "payrexx")]
        {
            router = router.merge(payrexx_router);
        }
        router
    });

    let mut app = api_router;

    // Conditionally add Swagger UI and JSON endpoint if openapi feature enabled
    #[cfg(feature = "openapi")]
    {
        #[cfg(feature = "twilio")]
        use connectify_twilio::doc::TwilioApiDoc;
        #[cfg(feature = "gcal")]
        use connectify_gcal::doc::GcalApiDoc;
        #[cfg(feature = "stripe")]
        use connectify_stripe::doc::StripeApiDoc;
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
