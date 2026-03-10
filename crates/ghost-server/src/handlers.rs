//! HTTP request handlers
//!
//! Implements all HTTP handlers for the Ghost API server.
//! Types are imported from ghost-schema - the single source of truth.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use axum::response::IntoResponse;
use std::sync::Arc;
use std::time::Instant;

use ghost_schema::{
    GhostContext, GhostPost, Strategy, Platform,
    HealthResponse, PostQuery, SearchQuery, InjectionHeaders,
    SearchResponse, TimelineResponse, WorkerInfo, WorkerHealthInfo,
};

use crate::{AppState, ServerError};

// ============================================================================
// Helper Functions
// ============================================================================

/// Extracts injection headers from HTTP headers
fn extract_injection_headers(headers: &HeaderMap) -> InjectionHeaders {
    InjectionHeaders {
        proxy: headers
            .get("x-ghost-proxy")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string()),
        session: headers
            .get("x-ghost-session")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string()),
        tenant_id: headers
            .get("x-ghost-tenant")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string()),
        bearer: headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string()),
    }
}

/// Builds a GhostContext from query parameters and headers
fn build_context(query: &PostQuery, headers: &HeaderMap) -> GhostContext {
    let injection = extract_injection_headers(headers);
    let mut builder = GhostContext::builder();

    // Priority: query params > headers
    if let Some(ref tenant_id) = query.tenant_id {
        builder = builder.tenant_id(tenant_id);
    } else if let Some(ref tenant_id) = injection.tenant_id {
        builder = builder.tenant_id(tenant_id);
    }

    if let Some(ref proxy) = query.proxy {
        builder = builder.proxy(proxy);
    } else if let Some(ref proxy) = injection.proxy {
        builder = builder.proxy(proxy);
    }

    if let Some(ref session) = injection.session {
        builder = builder.session(session);
    }

    builder = builder.strategy(query.parse_strategy());

    builder.build()
}

/// Builds a GhostContext for search operations
fn build_search_context(query: &SearchQuery, headers: &HeaderMap) -> GhostContext {
    let injection = extract_injection_headers(headers);
    let mut builder = GhostContext::builder();

    if let Some(ref tenant_id) = injection.tenant_id {
        builder = builder.tenant_id(tenant_id);
    }

    if let Some(ref proxy) = injection.proxy {
        builder = builder.proxy(proxy);
    }

    if let Some(ref session) = injection.session {
        builder = builder.session(session);
    }

    // Parse strategy if provided
    if let Some(ref strategy_str) = query.strategy {
        let strategy = match strategy_str.as_str() {
            "health_first" => Strategy::HealthFirst,
            "fastest" => Strategy::Fastest,
            "cost_optimized" => Strategy::CostOptimized,
            "official_first" => Strategy::OfficialFirst,
            "official_only" => Strategy::OfficialOnly,
            "scrapers_only" => Strategy::ScrapersOnly,
            "round_robin" => Strategy::RoundRobin,
            _ => Strategy::default(),
        };
        builder = builder.strategy(strategy);
    }

    builder.build()
}

// ============================================================================
// Health Check Handlers
// ============================================================================

/// Health check endpoint
///
/// Returns the current health status of the server and its workers.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse)
    )
)]
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let health_status = state.ghost.health_status().await;
    let worker_count = state.ghost.worker_count().await;

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        workers: worker_count,
        healthy_workers: health_status.healthy_count,
        uptime_secs: 0, // TODO: Track actual uptime
    })
}

/// Readiness check endpoint
///
/// Returns whether the server is ready to accept requests.
#[utoipa::path(
    get,
    path = "/ready",
    tag = "health",
    responses(
        (status = 200, description = "Server is ready"),
        (status = 503, description = "Server is not ready")
    )
)]
pub async fn ready_check(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ServerError> {
    let health_status = state.ghost.health_status().await;

    if health_status.has_available_workers() || health_status.total_count == 0 {
        Ok(axum::http::StatusCode::OK)
    } else {
        Err(ServerError::ServiceUnavailable("No workers available".into()))
    }
}

// ============================================================================
// X (Twitter) Handlers
// ============================================================================

/// Get a post from X (Twitter) by ID
///
/// Retrieves a single post/tweet by its ID using health-aware routing.
#[utoipa::path(
    get,
    path = "/x/post/{id}",
    tag = "x",
    params(
        ("id" = String, Path, description = "Post/Tweet ID")
    ),
    responses(
        (status = 200, description = "Post found", body = GhostPost),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal error")
    )
)]
pub async fn x_get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<PostQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let start = Instant::now();
    let ctx = build_context(&query, &headers);
    let strategy = query.parse_strategy();

    tracing::info!(
        post_id = %id,
        strategy = ?strategy,
        tenant_id = ?ctx.tenant_id,
        "Fetching X post"
    );

    let post = state.ghost.x()
        .get_post(&id, &ctx, strategy)
        .await?;

    tracing::info!(
        post_id = %id,
        latency_ms = start.elapsed().as_millis(),
        "Successfully fetched X post"
    );

    Ok(Json(post))
}

/// Get a user from X (Twitter) by ID or username
///
/// Retrieves a user profile by ID or username using health-aware routing.
#[utoipa::path(
    get,
    path = "/x/user/{id}",
    tag = "x",
    params(
        ("id" = String, Path, description = "User ID or username")
    ),
    responses(
        (status = 200, description = "User found", body = ghost_schema::GhostUser),
        (status = 404, description = "User not found")
    )
)]
pub async fn x_get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<PostQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let start = Instant::now();
    let ctx = build_context(&query, &headers);
    let strategy = query.parse_strategy();

    tracing::info!(
        user_id = %id,
        strategy = ?strategy,
        "Fetching X user"
    );

    let user = state.ghost.x()
        .get_user(&id, &ctx, strategy)
        .await?;

    tracing::info!(
        user_id = %id,
        latency_ms = start.elapsed().as_millis(),
        "Successfully fetched X user"
    );

    Ok(Json(user))
}

/// Search X (Twitter)
///
/// Search for posts/tweets matching a query.
#[utoipa::path(
    get,
    path = "/x/search",
    tag = "x",
    params(
        ("q" = String, Query, description = "Search query"),
        ("limit" = Option<usize>, Query, description = "Maximum results"),
        ("cursor" = Option<String>, Query, description = "Pagination cursor")
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResponse)
    )
)]
pub async fn x_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let start = Instant::now();
    let ctx = build_search_context(&query, &headers);

    tracing::info!(
        query = %query.q,
        limit = ?query.limit,
        "Searching X"
    );

    let results = state.ghost.x()
        .search(&query.q, &ctx, ctx.strategy)
        .await?;

    // Apply limit if specified
    let results: Vec<GhostPost> = if let Some(limit) = query.limit {
        results.into_iter().take(limit).collect()
    } else {
        results
    };

    tracing::info!(
        query = %query.q,
        result_count = results.len(),
        latency_ms = start.elapsed().as_millis(),
        "X search completed"
    );

    Ok(Json(SearchResponse::with_results(&query.q, results)))
}

/// Get trending content from X (Twitter)
///
/// Retrieves current trending topics/posts.
#[utoipa::path(
    get,
    path = "/x/trending",
    tag = "x",
    responses(
        (status = 200, description = "Trending posts", body = Vec<GhostPost>)
    )
)]
pub async fn x_trending(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PostQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let start = Instant::now();
    let ctx = build_context(&query, &headers);
    let strategy = query.parse_strategy();

    tracing::info!(strategy = ?strategy, "Fetching X trending");

    let results = state.ghost.x()
        .trending(&ctx, strategy)
        .await?;

    tracing::info!(
        result_count = results.len(),
        latency_ms = start.elapsed().as_millis(),
        "X trending fetched"
    );

    Ok(Json(results))
}

/// Get a user's timeline from X (Twitter)
///
/// Retrieves posts from a user's timeline.
#[utoipa::path(
    get,
    path = "/x/timeline/{user_id}",
    tag = "x",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Timeline posts", body = TimelineResponse)
    )
)]
pub async fn x_timeline(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Query(query): Query<PostQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let start = Instant::now();
    let ctx = build_context(&query, &headers);
    let strategy = query.parse_strategy();

    tracing::info!(
        user_id = %user_id,
        strategy = ?strategy,
        "Fetching X timeline"
    );

    let posts = state.ghost.x()
        .timeline(&user_id, &ctx, strategy)
        .await?;

    tracing::info!(
        user_id = %user_id,
        post_count = posts.len(),
        latency_ms = start.elapsed().as_millis(),
        "X timeline fetched"
    );

    Ok(Json(TimelineResponse::with_posts(posts)))
}

// ============================================================================
// Threads Handlers
// ============================================================================

/// Get a post from Threads by ID
///
/// Retrieves a single post by its ID.
#[utoipa::path(
    get,
    path = "/threads/post/{id}",
    tag = "threads",
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Post found", body = GhostPost),
        (status = 404, description = "Post not found")
    )
)]
pub async fn threads_get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<PostQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let start = Instant::now();
    let ctx = build_context(&query, &headers);
    let strategy = query.parse_strategy();

    tracing::info!(
        post_id = %id,
        strategy = ?strategy,
        "Fetching Threads post"
    );

    let post = state.ghost.threads()
        .get_post(&id, &ctx, strategy)
        .await?;

    tracing::info!(
        post_id = %id,
        latency_ms = start.elapsed().as_millis(),
        "Successfully fetched Threads post"
    );

    Ok(Json(post))
}

/// Get a user from Threads by ID or username
///
/// Retrieves a user profile by ID or username.
#[utoipa::path(
    get,
    path = "/threads/user/{id}",
    tag = "threads",
    params(
        ("id" = String, Path, description = "User ID or username")
    ),
    responses(
        (status = 200, description = "User found", body = ghost_schema::GhostUser),
        (status = 404, description = "User not found")
    )
)]
pub async fn threads_get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<PostQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let start = Instant::now();
    let ctx = build_context(&query, &headers);
    let strategy = query.parse_strategy();

    tracing::info!(
        user_id = %id,
        strategy = ?strategy,
        "Fetching Threads user"
    );

    let user = state.ghost.threads()
        .get_user(&id, &ctx, strategy)
        .await?;

    tracing::info!(
        user_id = %id,
        latency_ms = start.elapsed().as_millis(),
        "Successfully fetched Threads user"
    );

    Ok(Json(user))
}

/// Search Threads
///
/// Search for posts matching a query.
#[utoipa::path(
    get,
    path = "/threads/search",
    tag = "threads",
    params(
        ("q" = String, Query, description = "Search query")
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResponse)
    )
)]
pub async fn threads_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let start = Instant::now();
    let ctx = build_search_context(&query, &headers);

    tracing::info!(
        query = %query.q,
        limit = ?query.limit,
        "Searching Threads"
    );

    let results = state.ghost.threads()
        .search(&query.q, &ctx, ctx.strategy)
        .await?;

    // Apply limit if specified
    let results: Vec<GhostPost> = if let Some(limit) = query.limit {
        results.into_iter().take(limit).collect()
    } else {
        results
    };

    tracing::info!(
        query = %query.q,
        result_count = results.len(),
        latency_ms = start.elapsed().as_millis(),
        "Threads search completed"
    );

    Ok(Json(SearchResponse::with_results(&query.q, results)))
}

/// Get a user's timeline from Threads
///
/// Retrieves posts from a user's timeline.
#[utoipa::path(
    get,
    path = "/threads/timeline/{user_id}",
    tag = "threads",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Timeline posts", body = TimelineResponse)
    )
)]
pub async fn threads_timeline(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Query(query): Query<PostQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let start = Instant::now();
    let ctx = build_context(&query, &headers);
    let strategy = query.parse_strategy();

    tracing::info!(
        user_id = %user_id,
        strategy = ?strategy,
        "Fetching Threads timeline"
    );

    let posts = state.ghost.threads()
        .timeline(&user_id, &ctx, strategy)
        .await?;

    tracing::info!(
        user_id = %user_id,
        post_count = posts.len(),
        latency_ms = start.elapsed().as_millis(),
        "Threads timeline fetched"
    );

    Ok(Json(TimelineResponse::with_posts(posts)))
}

// ============================================================================
// Worker Management Handlers
// ============================================================================

/// List all registered workers
///
/// Returns a list of all workers with their status and health.
#[utoipa::path(
    get,
    path = "/workers",
    tag = "workers",
    responses(
        (status = 200, description = "Worker list", body = Vec<WorkerInfo>)
    )
)]
pub async fn list_workers(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ServerError> {
    let health_status = state.ghost.health_status().await;
    let worker_count = state.ghost.worker_count().await;

    // Build worker info list
    let workers: Vec<WorkerInfo> = (0..worker_count)
        .map(|i| WorkerInfo {
            id: format!("worker-{}", i),
            worker_type: "unknown".to_string(),
            capabilities: vec![],
            health_score: health_status.avg_score,
            status: "unknown".to_string(),
        })
        .collect();

    Ok(Json(workers))
}

/// Get worker health information
///
/// Returns detailed health information for a specific worker.
#[utoipa::path(
    get,
    path = "/workers/{id}/health",
    tag = "workers",
    params(
        ("id" = String, Path, description = "Worker ID")
    ),
    responses(
        (status = 200, description = "Worker health info", body = WorkerHealthInfo),
        (status = 404, description = "Worker not found")
    )
)]
pub async fn worker_health(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let health = state.ghost.health_engine().get_health(&id).await;

    Ok(Json(WorkerHealthInfo {
        worker_id: id.clone(),
        health_score: health.score,
        status: if health.is_cooling_down {
            "cooling_down".to_string()
        } else {
            health.tier().to_string()
        },
        avg_latency_ms: health.avg_latency_ms,
        success_rate: health.success_rate,
    }))
}

/// Get worker statistics
///
/// Returns detailed statistics for a specific worker.
#[utoipa::path(
    get,
    path = "/workers/{id}/stats",
    tag = "workers",
    params(
        ("id" = String, Path, description = "Worker ID")
    ),
    responses(
        (status = 200, description = "Worker statistics"),
        (status = 404, description = "Worker not found")
    )
)]
pub async fn worker_stats(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let detailed_stats = state.ghost.health_engine()
        .get_detailed_stats(&id)
        .await;

    match detailed_stats {
        Some(stats) => Ok(Json(serde_json::to_value(stats).unwrap_or_default())),
        None => Err(ServerError::NotFound(format!("Worker '{}' not found", id))),
    }
}

/// Enable a worker
///
/// Enables a previously disabled worker.
#[utoipa::path(
    post,
    path = "/workers/{id}/enable",
    tag = "workers",
    params(
        ("id" = String, Path, description = "Worker ID")
    ),
    responses(
        (status = 200, description = "Worker enabled"),
        (status = 404, description = "Worker not found")
    )
)]
pub async fn enable_worker(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    // Reset circuit breaker for this worker
    state.ghost.health_engine().reset_circuit_breaker(&id).await;

    tracing::info!(worker_id = %id, "Worker enabled");

    Ok(Json(serde_json::json!({
        "status": "enabled",
        "worker_id": id
    })))
}

/// Disable a worker
///
/// Disables a worker (trips its circuit breaker).
#[utoipa::path(
    post,
    path = "/workers/{id}/disable",
    tag = "workers",
    params(
        ("id" = String, Path, description = "Worker ID")
    ),
    responses(
        (status = 200, description = "Worker disabled"),
        (status = 404, description = "Worker not found")
    )
)]
pub async fn disable_worker(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    // Trip circuit breaker for this worker
    state.ghost.health_engine().trip_circuit_breaker(&id).await;

    tracing::info!(worker_id = %id, "Worker disabled");

    Ok(Json(serde_json::json!({
        "status": "disabled",
        "worker_id": id
    })))
}

/// Trigger health check for all workers
///
/// Manually triggers a health check for all registered workers.
#[utoipa::path(
    post,
    path = "/workers/check",
    tag = "workers",
    responses(
        (status = 200, description = "Health check triggered")
    )
)]
pub async fn check_all_workers(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ServerError> {
    state.ghost.check_health().await?;

    let status = state.ghost.health_status().await;

    Ok(Json(serde_json::json!({
        "status": "completed",
        "healthy": status.healthy_count,
        "degraded": status.degraded_count,
        "unhealthy": status.unhealthy_count,
        "total": status.total_count
    })))
}

// ============================================================================
// Metrics Handler
// ============================================================================

/// Prometheus metrics endpoint
///
/// Returns metrics in Prometheus text format.
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "metrics",
    responses(
        (status = 200, description = "Prometheus metrics", content_type = "text/plain")
    )
)]
pub async fn metrics(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let health_status = state.ghost.health_status().await;
    let worker_count = state.ghost.worker_count().await;

    (
        axum::http::StatusCode::OK,
        axum::http::header::HeaderMap::new(),
        format!(
            r#"# HELP ghost_workers_total Total number of registered workers
# TYPE ghost_workers_total gauge
ghost_workers_total {}

# HELP ghost_workers_healthy Number of healthy workers
# TYPE ghost_workers_healthy gauge
ghost_workers_healthy {}

# HELP ghost_workers_degraded Number of degraded workers
# TYPE ghost_workers_degraded gauge
ghost_workers_degraded {}

# HELP ghost_workers_unhealthy Number of unhealthy workers
# TYPE ghost_workers_unhealthy gauge
ghost_workers_unhealthy {}

# HELP ghost_health_score_average Average health score across all workers
# TYPE ghost_health_score_average gauge
ghost_health_score_average {}

# HELP ghost_platform_x_workers Workers available for X platform
# TYPE ghost_platform_x_workers gauge
ghost_platform_x_workers {}

# HELP ghost_platform_threads_workers Workers available for Threads platform
# TYPE ghost_platform_threads_workers gauge
ghost_platform_threads_workers {}
"#,
            worker_count,
            health_status.healthy_count,
            health_status.degraded_count,
            health_status.unhealthy_count,
            health_status.avg_score,
            health_status.platform_status.get(&Platform::X)
                .map(|s| s.available_workers).unwrap_or(0),
            health_status.platform_status.get(&Platform::Threads)
                .map(|s| s.available_workers).unwrap_or(0)
        ),
    )
}

/// API information endpoint
///
/// Returns information about the API.
#[utoipa::path(
    get,
    path = "/api-info",
    tag = "metrics",
    responses(
        (status = 200, description = "API information")
    )
)]
pub async fn api_info() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "Ghost API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "The unified programmatic bridge for X (Twitter) & Threads",
        "endpoints": {
            "x": ["/x/post/{id}", "/x/user/{id}", "/x/search", "/x/trending", "/x/timeline/{user_id}"],
            "threads": ["/threads/post/{id}", "/threads/user/{id}", "/threads/search", "/threads/timeline/{user_id}"],
            "workers": ["/workers", "/workers/{id}/health", "/workers/{id}/stats", "/workers/{id}/enable", "/workers/{id}/disable"],
            "health": ["/health", "/ready"],
            "metrics": ["/metrics"]
        }
    }))
}
