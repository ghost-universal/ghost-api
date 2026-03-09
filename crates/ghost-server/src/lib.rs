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
    ServerConfig, HealthResponse,
    PostQuery, SearchQuery, InjectionHeaders,
    SearchResponse, TimelineResponse,
    WorkerInfo, WorkerHealthInfo,
    ErrorResponse, NotFoundResponse,
};

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Creates the Axum application
pub async fn create_app(config: ServerConfig) -> Result<Router, ServerError> {
    // TODO: Implement app creation
    let ghost = ghost_core::Ghost::init().await?;

    // Build API routes
    let api_routes = routes::create_routes(ghost);

    // Build Swagger UI
    let swagger = utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", routes::openapi_spec());

    let app = Router::new()
        .merge(api_routes)
        .merge(swagger)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http());

    Ok(app)
}
