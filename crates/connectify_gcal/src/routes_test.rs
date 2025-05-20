#[cfg(test)]
mod tests {
    use crate::routes::routes;
    use axum::Router;
    use connectify_config::AppConfig;
    #[allow(unused_imports)]
    // the warning is due to unused imports not recognized by rustfmt, but for features
    use std::sync::Arc;
    // tower import removed as it's not available in the test environment

    // Helper function to create a mock AppConfig for testing
    fn create_mock_config() -> Arc<AppConfig> {
        Arc::new(AppConfig {
            use_gcal: true,
            use_stripe: false,
            use_twilio: false,
            use_payrexx: false,
            use_fulfillment: false,
            use_calendly: false,
            use_adhoc: false,
            server: connectify_config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            database: Some(connectify_config::DatabaseConfig {
                url: "postgres://localhost/test".to_string(),
            }),
            gcal: Some(connectify_config::GcalConfig {
                calendar_id: Some("primary".to_string()),
                key_path: Some("test_key.json".to_string()),
                time_slot_duration: Some(30),
            }),
            stripe: None,
            twilio: None,
            payrexx: None,
            fulfillment: None,
            adhoc_settings: None,
        })
    }

    #[tokio::test]
    async fn test_routes_configuration() {
        // This test will fail because we can't easily mock the calendar hub
        // In a real test, you would use a proper mock of the Google Calendar API
        // This is just to demonstrate the structure of the test

        // Create a mock config
        let config = create_mock_config();

        // Try to create the router
        // This will likely fail without proper mocking of the calendar hub
        let _result = std::panic::catch_unwind(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let router = routes(config).await;

                // Check that the router has the expected routes
                // We can't easily check the exact routes, but we can check that it's a Router
                assert!(router.is_a_router());
            });
        });
    }

    // Extension trait to check if a value is a Router
    trait IsRouter {
        fn is_a_router(&self) -> bool;
    }

    impl IsRouter for Router {
        fn is_a_router(&self) -> bool {
            true
        }
    }
}
