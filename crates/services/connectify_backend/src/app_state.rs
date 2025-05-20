// --- File: crates/services/connectify_backend/src/app_state.rs ---
use std::sync::Arc;
use connectify_config::AppConfig;
use connectify_common::services::ServiceFactory;

use connectify_backend::service_factory::ConnectifyServiceFactory;

#[cfg(feature = "gcal")]
use connectify_gcal::handlers::GcalState;

/// Application state that is shared across all routes.
/// 
/// This struct is a central part of the application's architecture and is used to manage shared state
/// across all routes and services. It follows the dependency injection pattern described in
/// the project's documentation.
/// 
/// While some fields (like `config` and `service_factory`) may not be directly accessed in every part
/// of the current implementation, they are essential to the architecture:
/// 
/// - They provide a centralized place to store configuration and service instances
/// - They enable dependency injection for better testability
/// - They support the service factory pattern used throughout the application
/// - They allow for future extensions without changing the core architecture
/// 
/// As the application evolves, these fields will become increasingly important for managing
/// the growing complexity of service dependencies.
#[derive(Clone)]
pub struct AppState {
    /// The application configuration.
    /// 
    /// This field stores the application configuration that was loaded at startup.
    /// While it may not be directly accessed in every part of the current implementation,
    /// it's kept here to:
    /// 
    /// 1. Provide a single source of truth for configuration
    /// 2. Make configuration available to any component that needs it
    /// 3. Support the dependency injection pattern
    /// 4. Enable future extensions that may need configuration access
    #[allow(dead_code)]
    pub config: Arc<AppConfig>,

    /// Service factory for accessing external services.
    /// 
    /// This field provides access to all external services used by the application through
    /// the ServiceFactory trait. It's a cornerstone of the dependency injection pattern used
    /// in this application.
    /// 
    /// While it may not be directly accessed in every part of the current implementation,
    /// it's essential because:
    /// 
    /// 1. It centralizes service access through a single interface
    /// 2. It enables dependency injection and improves testability
    /// 3. It allows services to be conditionally initialized based on configuration
    /// 4. It provides a consistent pattern for accessing different types of services
    /// 
    /// As more services are added to the application, this field will become increasingly
    /// important for managing service dependencies.
    #[allow(dead_code)]
    pub service_factory: Arc<dyn ServiceFactory>,

    /// Google Calendar state, available when the "gcal" feature is enabled.
    /// This is kept for backward compatibility during transition to the new architecture.
    #[cfg(feature = "gcal")]
    pub gcal_state: Option<Arc<GcalState>>,

    // Add other feature-specific states here as needed
}

/// Builder for AppState to provide a cleaner initialization pattern.
/// This struct is part of the application's architecture and follows the builder pattern.
/// 
/// While this builder is not currently used in the main application flow (which uses the
/// simpler `AppState::new` method directly), it is kept for several important reasons:
/// 
/// 1. It provides a more flexible way to construct AppState objects with different
///    configurations and dependencies, which will be useful as the application grows.
/// 2. It follows the builder pattern, which is a standard design pattern for constructing
///    complex objects with many optional parameters.
/// 3. It makes testing easier by allowing test code to construct AppState objects with
///    specific mock services.
/// 4. It supports future extensibility by providing a clear pattern for adding new
///    dependencies to AppState.
/// 
/// As the application evolves and more services are added, the builder pattern will
/// become increasingly valuable for managing the complexity of AppState initialization.
#[allow(dead_code)]
pub struct AppStateBuilder {
    config: Arc<AppConfig>,
    service_factory: Option<Arc<dyn ServiceFactory>>,

    #[cfg(feature = "gcal")]
    gcal_state: Option<Arc<GcalState>>,
}

impl AppStateBuilder {
    /// Create a new AppStateBuilder with the given configuration.
    #[allow(dead_code)]
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            service_factory: None,
            #[cfg(feature = "gcal")]
            gcal_state: None,
        }
    }

    /// Set the service factory.
    #[allow(dead_code)]
    pub fn with_service_factory(mut self, service_factory: Arc<dyn ServiceFactory>) -> Self {
        self.service_factory = Some(service_factory);
        self
    }

    /// Set the Google Calendar state.
    #[cfg(feature = "gcal")]
    #[allow(dead_code)]
    pub fn with_gcal_state(mut self, gcal_state: Option<Arc<GcalState>>) -> Self {
        self.gcal_state = gcal_state;
        self
    }

    /// Build the AppState.
    #[allow(dead_code)]
    pub fn build(self) -> AppState {
        assert!(self.service_factory.is_some(), "Service factory must be set");

        AppState {
            config: self.config,
            service_factory: self.service_factory.unwrap(),
            #[cfg(feature = "gcal")]
            gcal_state: self.gcal_state,
        }
    }
}

impl AppState {
    /// Create a new AppStateBuilder with the given configuration.
    /// 
    /// This method is the entry point to the builder pattern for AppState construction.
    /// While it's not currently used in the main application flow (which uses the `new` method directly),
    /// it's kept for the same reasons as the AppStateBuilder:
    /// 
    /// - It provides a more flexible way to construct AppState objects
    /// - It supports testing by allowing easy construction of AppState with mock services
    /// - It follows standard design patterns for complex object construction
    /// - It enables future extensibility as the application grows
    /// 
    /// Example usage (for future or test code):
    /// ```
    /// let app_state = AppState::builder(config.clone())
    ///     .with_service_factory(Arc::new(my_service_factory))
    ///     .build();
    /// ```
    #[allow(dead_code)]
    pub fn builder(config: Arc<AppConfig>) -> AppStateBuilder {
        AppStateBuilder::new(config)
    }

    /// Create a new AppState with the given configuration.
    /// This is a convenience method that creates a service factory and builds the AppState.
    pub async fn new(config: Arc<AppConfig>) -> Self {
        let service_factory = Arc::new(ConnectifyServiceFactory::new(config.clone()).await);

        #[cfg(feature = "gcal")]
        let gcal_state = if config.use_gcal && config.gcal.is_some() {
            // For backward compatibility, create GcalState if needed
            if let Some(_calendar_service) = service_factory.calendar_service() {
                // We don't have direct access to the hub, but we can create it again
                // This is not ideal but necessary during the transition
                match connectify_gcal::auth::create_calendar_hub(config.gcal.as_ref().unwrap()).await {
                    Ok(hub) => {
                        Some(Arc::new(GcalState {
                            config: config.clone(),
                            calendar_hub: Arc::new(hub),
                        }))
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        Self {
            config,
            service_factory,
            #[cfg(feature = "gcal")]
            gcal_state,
        }
    }
}
