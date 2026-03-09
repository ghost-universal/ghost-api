//! HTTP request handlers
//!
//! Types imported from ghost-schema - the single source of truth.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use std::sync::Arc;

use ghost_core::Ghost;
use ghost_schema::{
    GhostContext, GhostPost, GhostUser, Strategy, Platform,
    HealthResponse, PostQuery, SearchQuery, InjectionHeaders,
    SearchResponse, WorkerInfo, WorkerHealthInfo,
};

use crate::ServerError;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse)
    )
)]
pub async fn health_check(State(_ghost): State<Arc<Ghost>>) -> Json<HealthResponse> {
    // TODO: Implement health check
    Json(HealthResponse::new())
}

/// Get a post from X (Twitter)
#[utoipa::path(
    get,
    path = "/x/post/{id}",
    params(
        ("id" = String, Path, description = "Post ID"),
    ),
    responses(
        (status = 200, description = "Post found", body = GhostPost),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal error")
    )
)]
pub async fn x_get_post(
    State(_ghost): State<Arc<Ghost>>,
    Path(_id): Path<String>,
    Query(_query): Query<PostQuery>,
    _headers: HeaderMap,
) -> Result<Json<GhostPost>, ServerError> {
    // TODO: Implement X post retrieval
    Err(ServerError::NotImplemented("x_get_post".into()))
}

/// Get a user from X (Twitter)
#[utoipa::path(
    get,
    path = "/x/user/{id}",
    params(
        ("id" = String, Path, description = "User ID or username"),
    ),
    responses(
        (status = 200, description = "User found", body = GhostUser),
        (status = 404, description = "User not found")
    )
)]
pub async fn x_get_user(
    State(_ghost): State<Arc<Ghost>>,
    Path(_id): Path<String>,
    Query(_query): Query<PostQuery>,
    _headers: HeaderMap,
) -> Result<Json<GhostUser>, ServerError> {
    // TODO: Implement X user retrieval
    Err(ServerError::NotImplemented("x_get_user".into()))
}

/// Search X (Twitter)
#[utoipa::path(
    get,
    path = "/x/search",
    params(
        ("q" = String, Query, description = "Search query"),
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResponse)
    )
)]
pub async fn x_search(
    State(_ghost): State<Arc<Ghost>>,
    Query(query): Query<SearchQuery>,
    _headers: HeaderMap,
) -> Result<Json<SearchResponse>, ServerError> {
    // TODO: Implement X search
    Ok(Json(SearchResponse::new(&query.q)))
}

/// Get trending from X (Twitter)
pub async fn x_trending(
    State(_ghost): State<Arc<Ghost>>,
    _headers: HeaderMap,
) -> Result<Json<Vec<GhostPost>>, ServerError> {
    // TODO: Implement X trending
    Ok(Json(Vec::new()))
}

/// Get a post from Threads
pub async fn threads_get_post(
    State(_ghost): State<Arc<Ghost>>,
    Path(_id): Path<String>,
    _headers: HeaderMap,
) -> Result<Json<GhostPost>, ServerError> {
    // TODO: Implement Threads post retrieval
    Err(ServerError::NotImplemented("threads_get_post".into()))
}

/// Get a user from Threads
pub async fn threads_get_user(
    State(_ghost): State<Arc<Ghost>>,
    Path(_id): Path<String>,
    _headers: HeaderMap,
) -> Result<Json<GhostUser>, ServerError> {
    // TODO: Implement Threads user retrieval
    Err(ServerError::NotImplemented("threads_get_user".into()))
}

/// Search Threads
pub async fn threads_search(
    State(_ghost): State<Arc<Ghost>>,
    Query(query): Query<SearchQuery>,
    _headers: HeaderMap,
) -> Result<Json<SearchResponse>, ServerError> {
    // TODO: Implement Threads search
    Ok(Json(SearchResponse::new(&query.q)))
}

/// List all workers
pub async fn list_workers(
    State(_ghost): State<Arc<Ghost>>,
) -> Result<Json<Vec<WorkerInfo>>, ServerError> {
    // TODO: Implement worker listing
    Ok(Json(Vec::new()))
}

/// Get worker health
pub async fn worker_health(
    State(_ghost): State<Arc<Ghost>>,
    Path(id): Path<String>,
) -> Result<Json<WorkerHealthInfo>, ServerError> {
    // TODO: Implement worker health check
    Ok(Json(WorkerHealthInfo::new(&id)))
}

/// Enable a worker
pub async fn enable_worker(
    State(_ghost): State<Arc<Ghost>>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, ServerError> {
    // TODO: Implement worker enabling
    Ok(Json(serde_json::json!({"status": "enabled"})))
}

/// Disable a worker
pub async fn disable_worker(
    State(_ghost): State<Arc<Ghost>>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, ServerError> {
    // TODO: Implement worker disabling
    Ok(Json(serde_json::json!({"status": "disabled"})))
}

/// Metrics endpoint
pub async fn metrics(State(_ghost): State<Arc<Ghost>>) -> String {
    // TODO: Implement Prometheus metrics
    "# HELP ghost_workers_total Total number of workers\n# TYPE ghost_workers_total gauge\nghost_workers_total 0\n".to_string()
}
