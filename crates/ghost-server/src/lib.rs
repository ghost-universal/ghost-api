//! Ghost API Server Library
//!
//! Provides the HTTP server with Swagger UI for the Ghost API.

mod routes;
mod handlers;
mod error;
mod config;

pub use routes::*;
pub use handlers::*;
pub use error::*;
pub use config::*;

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
