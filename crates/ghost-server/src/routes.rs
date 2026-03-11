//! API route definitions
//!
//! Defines all HTTP routes for the Ghost API server.
//! Types are imported from ghost-schema - the single source of truth.

use axum::Router;
use axum::routing::{get, post};
use utoipa::OpenApi;
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

/// Creates all API routes
pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check endpoint
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::ready_check))

        // X (Twitter) endpoints
        .nest("/x", x_routes())

        // Threads endpoints
        .nest("/threads", threads_routes())

        // Worker management
        .nest("/workers", worker_routes())

        // Metrics endpoint (Prometheus format)
        .route("/metrics", get(handlers::metrics))

        // API info endpoint
        .route("/api-info", get(handlers::api_info))

        .with_state(state)
}

/// X (Twitter) API routes
fn x_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Post operations
        .route("/post/{id}", get(handlers::x_get_post))
        
        // User operations
        .route("/user/{id}", get(handlers::x_get_user))
        
        // Search
        .route("/search", get(handlers::x_search))
        
        // Trending
        .route("/trending", get(handlers::x_trending))
        
        // Timeline
        .route("/timeline/{user_id}", get(handlers::x_timeline))
}

/// Threads API routes
fn threads_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Post operations
        .route("/post/{id}", get(handlers::threads_get_post))
        
        // User operations
        .route("/user/{id}", get(handlers::threads_get_user))
        
        // Search
        .route("/search", get(handlers::threads_search))
        
        // Timeline
        .route("/timeline/{user_id}", get(handlers::threads_timeline))
}

/// Worker management routes
fn worker_routes() -> Router<Arc<AppState>> {
    Router::new()
        // List all workers
        .route("/", get(handlers::list_workers))
        
        // Worker health
        .route("/{id}/health", get(handlers::worker_health))
        
        // Worker stats
        .route("/{id}/stats", get(handlers::worker_stats))
        
        // Enable worker
        .route("/{id}/enable", post(handlers::enable_worker))
        
        // Disable worker
        .route("/{id}/disable", post(handlers::disable_worker))
        
        // Trigger health check
        .route("/check", post(handlers::check_all_workers))
}

/// OpenAPI specification for the Ghost API
pub fn openapi_spec() -> utoipa::openapi::OpenApi {
    // Temporarily disabled due to ToSchema trait requirements
    // The full OpenAPI spec requires ToSchema derives on all types
    utoipa::openapi::OpenApiBuilder::new()
        .info(utoipa::openapi::InfoBuilder::new()
            .title("Ghost API")
            .version("0.1.0")
            .description(Some("Ghost API")))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generation() {
        let spec = openapi_spec();
        // Verify the spec has basic info (paths are empty due to ToSchema trait requirements)
        assert_eq!(spec.info.title, "Ghost API");
        assert_eq!(spec.info.version, "0.1.0");
    }
}
