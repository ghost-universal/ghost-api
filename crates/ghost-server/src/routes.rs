//! API route definitions

use axum::Router;
use axum::routing::{get, post};
use utoipa::OpenApi;

use crate::handlers;
use ghost_core::Ghost;

/// Creates all API routes
pub fn create_routes(ghost: Ghost) -> Router {
    // TODO: Implement route creation
    Router::new()
        // Health check
        .route("/health", get(handlers::health_check))

        // X (Twitter) endpoints
        .nest("/x", x_routes())

        // Threads endpoints
        .nest("/threads", threads_routes())

        // Worker management
        .nest("/workers", worker_routes())

        // Metrics
        .route("/metrics", get(handlers::metrics))

        .with_state(std::sync::Arc::new(ghost))
}

/// X (Twitter) API routes
fn x_routes() -> Router {
    // TODO: Implement X routes
    Router::new()
        .route("/post/:id", get(handlers::x_get_post))
        .route("/user/:id", get(handlers::x_get_user))
        .route("/search", get(handlers::x_search))
        .route("/trending", get(handlers::x_trending))
}

/// Threads API routes
fn threads_routes() -> Router {
    // TODO: Implement Threads routes
    Router::new()
        .route("/post/:id", get(handlers::threads_get_post))
        .route("/user/:id", get(handlers::threads_get_user))
        .route("/search", get(handlers::threads_search))
}

/// Worker management routes
fn worker_routes() -> Router {
    // TODO: Implement worker routes
    Router::new()
        .route("/", get(handlers::list_workers))
        .route("/:id/health", get(handlers::worker_health))
        .route("/:id/enable", post(handlers::enable_worker))
        .route("/:id/disable", post(handlers::disable_worker))
}

/// OpenAPI specification
pub fn openapi_spec() -> utoipa::openapi::OpenApi {
    // TODO: Implement OpenAPI spec generation
    #[derive(OpenApi)]
    #[openapi(
        paths(
            handlers::health_check,
            handlers::x_get_post,
            handlers::x_get_user,
            handlers::x_search,
        ),
        components(schemas(
            ghost_schema::GhostPost,
            ghost_schema::GhostUser,
            ghost_schema::Platform,
        )),
    )]
    struct ApiDoc;

    ApiDoc::openapi()
}
