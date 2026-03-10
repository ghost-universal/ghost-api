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
fn x_routes() -> Router {
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
fn threads_routes() -> Router {
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
fn worker_routes() -> Router {
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
    #[derive(OpenApi)]
    #[openapi(
        info(
            title = "Ghost API",
            version = "0.1.0",
            description = "The unified programmatic bridge for X (Twitter) & Threads.\n\nA type-safe Rust orchestration engine that wraps polyglot scrapers (Python, TS, Go) into a single interface with health-based routing and per-request multi-tenant injection.",
            license(
                name = "MIT",
                url = "https://opensource.org/licenses/MIT"
            ),
            contact(
                name = "Ghost API Contributors",
                url = "https://github.com/ghost-universal/ghost-api"
            )
        ),
        tags(
            (name = "health", description = "Health check endpoints"),
            (name = "x", description = "X (Twitter) API endpoints"),
            (name = "threads", description = "Threads API endpoints"),
            (name = "workers", description = "Worker management endpoints"),
            (name = "metrics", description = "Metrics and observability")
        ),
        paths(
            // Health
            handlers::health_check,
            handlers::ready_check,
            
            // X (Twitter)
            handlers::x_get_post,
            handlers::x_get_user,
            handlers::x_search,
            handlers::x_trending,
            handlers::x_timeline,
            
            // Threads
            handlers::threads_get_post,
            handlers::threads_get_user,
            handlers::threads_search,
            handlers::threads_timeline,
            
            // Workers
            handlers::list_workers,
            handlers::worker_health,
            handlers::worker_stats,
            handlers::enable_worker,
            handlers::disable_worker,
            handlers::check_all_workers,
            
            // Metrics
            handlers::metrics,
            handlers::api_info
        ),
        components(schemas(
            // Core types
            ghost_schema::GhostPost,
            ghost_schema::GhostUser,
            ghost_schema::GhostMedia,
            ghost_schema::MediaType,
            ghost_schema::Platform,
            ghost_schema::Strategy,
            ghost_schema::GhostContext,
            
            // Request/Response types
            ghost_schema::HealthResponse,
            ghost_schema::PostQuery,
            ghost_schema::SearchQuery,
            ghost_schema::SearchResponse,
            ghost_schema::TimelineResponse,
            ghost_schema::InjectionHeaders,
            
            // Worker types
            ghost_schema::WorkerInfo,
            ghost_schema::WorkerHealthInfo,
            ghost_schema::WorkerHealth,
            ghost_schema::WorkerStatus,
            ghost_schema::HealthTier,
            
            // Error types
            ghost_schema::ErrorResponse,
            ghost_schema::NotFoundResponse,
            
            // Capability types
            ghost_schema::Capability,
            ghost_schema::CapabilityTier,
            ghost_schema::WorkerType,
        ))
    )]
    struct ApiDoc;

    ApiDoc::openapi()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generation() {
        let spec = openapi_spec();
        // Verify the spec has paths
        assert!(!spec.paths.paths.is_empty());
    }
}
