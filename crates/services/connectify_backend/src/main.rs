// File: services/connectify_backend/src/main.rs
use axum::{routing::get, Router};
use connectify_config::load_config;
use connectify_gcal::routes as gcal_routes;
use connectify_payrexx::routes as payrexx_routes;
use connectify_twilio::routes as twilio_routes;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

// #[axum::debug_handler]
// async fn show_config(State(config): State<Arc<AppConfig>>) -> Json<AppConfig> {
//     Json(config.as_ref().clone())
// }

#[tokio::main]
async fn main() {
    let config = Arc::new(load_config().expect("Failed to load config"));

    let api_router = Router::new()
        .route("/", get(|| async { "Welcome to Connectify-Rs API!" }))
        // .route("/config", get(show_config))
        .with_state(config.clone());

    let twilio_router = twilio_routes::routes(config.clone()); // Or whatever function provides the Router
    let payrexx_router = payrexx_routes::routes(config.clone()); // Or whatever function provides the Router
    let gcal_router = gcal_routes::routes(config.clone()).await; // Or whatever function provides the Router
    // Combine all API routes and nest them under /api
    let api_router = Router::new().nest(
        "/api",
        api_router
            .merge(twilio_router)
            .merge(gcal_router)
            .merge(payrexx_router),
    );

    let mut app = api_router;
    /*
        #[cfg(feature = "openapi")]
        {
            use connectify_gcal::doc::GcalApiDoc;
            use connectify_twilio::doc::TwilioApiDoc;
            use connectify_payrexx::doc::PayrexxApiDoc;
            use utoipa::OpenApi;

            // Create the OpenAPI spec
            let mut openapi = GcalApiDoc::openapi();

            // Ensure the servers array is correctly set
            if openapi
                .servers
                .as_ref()
                .map_or(true, |servers| servers.is_empty())
            {
                openapi.servers = vec![utoipa::openapi::Server::new("/api")].into();
            }

            // Set up Swagger UI with the modified spec
            use utoipa_swagger_ui::SwaggerUi;

            let swagger_router =
                SwaggerUi::new("/api/docs/gcal")
                    .url("/api/api-docs/gcal.json", GcalApiDoc::openapi())
                    .url("/api/docs/twilio", TwilioApiDoc::openapi())
                    .url("/api/api-docs/payrexx.json", PayrexxApiDoc::openapi());

            app = app.merge(swagger_router);
        }

    */
    // Conditionally add Swagger UI and JSON endpoint if openapi feature enabled
    #[cfg(feature = "openapi")]
    {
        use connectify_gcal::doc::GcalApiDoc;
        use connectify_payrexx::doc::PayrexxApiDoc;
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
        let mut openapi_doc = ApiDoc::openapi();
        openapi_doc.merge(GcalApiDoc::openapi());
        openapi_doc.merge(TwilioApiDoc::openapi());
        openapi_doc.merge(PayrexxApiDoc::openapi());
        println!("📖 Adding Swagger UI at /api/docs");

        // Create the Swagger UI route, referencing the merged doc
        let swagger_ui =
            SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", openapi_doc.clone());

        // Merge the Swagger UI into the main app router
        app = app.merge(swagger_ui);

        // Add a route to serve the generated OpenAPI JSON spec
        // app = app.route(
        //     "/api/docs/openapi.json",
        //     get(|| async { Json(openapi_doc) }) // Handler returns the spec as JSON
        // );
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
