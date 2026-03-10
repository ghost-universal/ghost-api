//! Ghost API Server Library
//!
//! Provides the HTTP server with Swagger UI for the Ghost API.
//!
//! All types are imported from `ghost-schema` - the single source of truth.

mod routes;
mod handlers;
mod error;

pub use routes::*;
pub use handlers::*;
pub use error::*;

// Re-export server types from ghost-schema
pub use ghost_schema::{
    HealthResponse, PostQuery, SearchQuery, InjectionHeaders,
    SearchResponse, TimelineResponse, WorkerInfo, WorkerHealthInfo,
    ErrorResponse, NotFoundResponse, GhostContext, GhostPost, GhostUser,
    Platform, Strategy,
};

use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower::ServiceBuilder;

use ghost_core::Ghost;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Ghost engine instance
    pub ghost: Arc<Ghost>,
    /// Server configuration
    pub config: ServerConfig,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server address
    pub addr: std::net::SocketAddr,
    /// Whether to enable CORS
    pub cors_enabled: bool,
    /// Whether to enable Swagger UI
    pub swagger_enabled: bool,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Maximum request body size
    pub max_body_size: usize,
    /// Log level
    pub log_level: String,
}

impl ServerConfig {
    /// Creates a new server config with defaults
    pub fn new() -> Self {
        Self {
            addr: std::net::SocketAddr::from(([0, 0, 0, 0], 3000)),
            cors_enabled: true,
            swagger_enabled: true,
            request_timeout_secs: 30,
            max_body_size: 1024 * 1024, // 1MB
            log_level: "info".to_string(),
        }
    }

    /// Loads configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::new();

        if let Ok(addr_str) = std::env::var("GHOST_ADDR") {
            if let Ok(parsed) = addr_str.parse() {
                config.addr = parsed;
            }
        }

        if let Ok(cors) = std::env::var("GHOST_CORS") {
            config.cors_enabled = cors.parse().unwrap_or(true);
        }

        if let Ok(swagger) = std::env::var("GHOST_SWAGGER") {
            config.swagger_enabled = swagger.parse().unwrap_or(true);
        }

        if let Ok(timeout) = std::env::var("GHOST_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse() {
                config.request_timeout_secs = secs;
            }
        }

        if let Ok(level) = std::env::var("RUST_LOG") {
            config.log_level = level;
        }

        config
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), ServerError> {
        if self.request_timeout_secs == 0 {
            return Err(ServerError::BadRequest("request_timeout_secs must be > 0".into()));
        }

        if self.max_body_size == 0 {
            return Err(ServerError::BadRequest("max_body_size must be > 0".into()));
        }

        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates the Axum application with all routes and middleware
pub async fn create_app(config: ServerConfig) -> Result<Router, ServerError> {
    // Initialize Ghost engine
    let ghost = Ghost::init().await
        .map_err(|e| ServerError::Internal(format!("Failed to initialize Ghost: {}", e)))?;

    // Create application state
    let state = Arc::new(AppState {
        ghost: Arc::new(ghost),
        config: config.clone(),
    });

    // Build API routes
    let api_routes = routes::create_routes(state.clone());

    // Build middleware stack
    let mut app = Router::new()
        .merge(api_routes);

    // Add Swagger UI if enabled
    if config.swagger_enabled {
        let swagger = utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", routes::openapi_spec());
        app = app.merge(swagger);
    }

    // Add CORS if enabled
    let app = if config.cors_enabled {
        app.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
    } else {
        app
    };

    // Add common middleware layers
    let app = app
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(RequestBodyLimitLayer::new(config.max_body_size))
        )
        .with_state(state);

    Ok(app)
}

/// Creates the Axum application with a pre-configured Ghost instance
pub fn create_app_with_ghost(ghost: Arc<Ghost>, config: ServerConfig) -> Router {
    let state = Arc::new(AppState {
        ghost,
        config: config.clone(),
    });

    // Build API routes
    let api_routes = routes::create_routes(state.clone());

    // Build router
    let mut app = Router::new()
        .merge(api_routes);

    // Add Swagger UI if enabled
    if config.swagger_enabled {
        let swagger = utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", routes::openapi_spec());
        app = app.merge(swagger);
    }

    // Add CORS if enabled
    let app = if config.cors_enabled {
        app.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
    } else {
        app
    };

    // Add common middleware layers
    app.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(RequestBodyLimitLayer::new(config.max_body_size))
    )
    .with_state(state)
}
