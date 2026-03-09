//! Request routing logic
//!
//! This module handles routing requests to appropriate workers
//! based on strategy, health, and capability.

use std::sync::Arc;

use ghost_schema::{
    Capability, GhostContext, GhostError, GhostPost, GhostUser, Platform, Strategy,
    CapabilityTier, WorkerSelection, PayloadBlob, RawContext,
};

use crate::{FallbackEngine, GhostConfig, HealthEngine, WorkerRegistry};

/// Router for dispatching requests to appropriate workers
///
/// The router is responsible for:
/// - Selecting workers based on strategy and health
/// - Executing requests with fallback support
/// - Building raw contexts from ghost contexts
/// - Parsing responses into unified types
pub struct GhostRouter {
    workers: Arc<tokio::sync::RwLock<WorkerRegistry>>,
    health_engine: Arc<HealthEngine>,
    /// Fallback engine for tier escalation (reserved for future use)
    _fallback: Arc<FallbackEngine>,
    config: Arc<GhostConfig>,
    /// Round-robin counter for RR strategy
    rr_counter: Arc<tokio::sync::Mutex<usize>>,
}

impl GhostRouter {
    /// Creates a new router
    pub fn new(
        workers: Arc<tokio::sync::RwLock<WorkerRegistry>>,
        health_engine: Arc<HealthEngine>,
        fallback: Arc<FallbackEngine>,
        config: Arc<GhostConfig>,
    ) -> Self {
        Self {
            workers,
            health_engine,
            _fallback: fallback,
            config,
            rr_counter: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    /// Routes a get_post request
    pub async fn route_get_post(
        &self,
        platform: Platform,
        id: &str,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<GhostPost, GhostError> {
        let capability = Self::get_post_capability(platform)?;
        let selection = self.select_worker(capability, platform, ctx, strategy, &[]).await?;

        self.execute_with_fallback(
            || async {
                self.execute_get_post(&selection, platform, id, ctx).await
            },
            capability,
            platform,
            ctx,
            strategy,
        ).await
    }

    /// Routes a get_user request
    pub async fn route_get_user(
        &self,
        platform: Platform,
        id: &str,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<GhostUser, GhostError> {
        let capability = Self::get_user_capability(platform)?;
        let selection = self.select_worker(capability, platform, ctx, strategy, &[]).await?;

        self.execute_with_fallback(
            || async {
                self.execute_get_user(&selection, platform, id, ctx).await
            },
            capability,
            platform,
            ctx,
            strategy,
        ).await
    }

    /// Routes a search request
    pub async fn route_search(
        &self,
        platform: Platform,
        query: &str,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<Vec<GhostPost>, GhostError> {
        let capability = Self::get_search_capability(platform)?;
        let selection = self.select_worker(capability, platform, ctx, strategy, &[]).await?;

        self.execute_with_fallback(
            || async {
                self.execute_search(&selection, platform, query, ctx).await
            },
            capability,
            platform,
            ctx,
            strategy,
        ).await
    }

    /// Routes a trending request
    pub async fn route_trending(
        &self,
        platform: Platform,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<Vec<GhostPost>, GhostError> {
        let capability = Self::get_trending_capability(platform)?;
        let selection = self.select_worker(capability, platform, ctx, strategy, &[]).await?;

        self.execute_with_fallback(
            || async {
                self.execute_trending(&selection, platform, ctx).await
            },
            capability,
            platform,
            ctx,
            strategy,
        ).await
    }

    /// Routes a timeline request
    pub async fn route_timeline(
        &self,
        platform: Platform,
        user_id: &str,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<Vec<GhostPost>, GhostError> {
        let capability = Self::get_timeline_capability(platform)?;
        let selection = self.select_worker(capability, platform, ctx, strategy, &[]).await?;

        self.execute_with_fallback(
            || async {
                self.execute_timeline(&selection, platform, user_id, ctx).await
            },
            capability,
            platform,
            ctx,
            strategy,
        ).await
    }

    /// Maps platform to post capability
    fn get_post_capability(platform: Platform) -> Result<Capability, GhostError> {
        match platform {
            Platform::X => Ok(Capability::XRead),
            Platform::Threads => Ok(Capability::ThreadsRead),
            Platform::Unknown => Err(GhostError::ValidationError("Unknown platform".into())),
        }
    }

    /// Maps platform to user capability
    fn get_user_capability(platform: Platform) -> Result<Capability, GhostError> {
        match platform {
            Platform::X => Ok(Capability::XUserRead),
            Platform::Threads => Ok(Capability::ThreadsUserRead),
            Platform::Unknown => Err(GhostError::ValidationError("Unknown platform".into())),
        }
    }

    /// Maps platform to search capability
    fn get_search_capability(platform: Platform) -> Result<Capability, GhostError> {
        match platform {
            Platform::X => Ok(Capability::XSearch),
            Platform::Threads => Ok(Capability::ThreadsSearch),
            Platform::Unknown => Err(GhostError::ValidationError("Unknown platform".into())),
        }
    }

    /// Maps platform to trending capability
    fn get_trending_capability(platform: Platform) -> Result<Capability, GhostError> {
        match platform {
            Platform::X => Ok(Capability::XTrending),
            Platform::Threads => Ok(Capability::ThreadsRead), // Threads uses read for trending
            Platform::Unknown => Err(GhostError::ValidationError("Unknown platform".into())),
        }
    }

    /// Maps platform to timeline capability
    fn get_timeline_capability(platform: Platform) -> Result<Capability, GhostError> {
        match platform {
            Platform::X => Ok(Capability::XTimeline),
            Platform::Threads => Ok(Capability::ThreadsTimeline),
            Platform::Unknown => Err(GhostError::ValidationError("Unknown platform".into())),
        }
    }

    /// Selects the best worker for a request
    async fn select_worker(
        &self,
        capability: Capability,
        platform: Platform,
        _ctx: &GhostContext,
        strategy: Strategy,
        exclude: &[String],
    ) -> Result<WorkerSelection, GhostError> {
        let workers = self.workers.read().await;
        let candidates = workers.get_ids_by_capability_and_platform(capability, platform);

        // Filter out excluded workers
        let candidates: Vec<String> = candidates
            .into_iter()
            .filter(|id| !exclude.contains(id))
            .collect();

        if candidates.is_empty() {
            return Err(GhostError::WorkersExhausted(
                format!("No workers available for {:?} on {:?}", capability, platform)
            ));
        }

        // Apply strategy to select best worker
        match strategy {
            Strategy::HealthFirst => self.select_by_health(&workers, candidates).await,
            Strategy::Fastest => self.select_by_latency(&workers, candidates).await,
            Strategy::CostOptimized => self.select_by_cost(&workers, candidates).await,
            Strategy::RoundRobin => self.select_round_robin(&workers, candidates).await,
            Strategy::OfficialFirst => self.select_official_first(&workers, candidates).await,
            Strategy::OfficialOnly => self.select_official_only(&workers, candidates).await,
            Strategy::ScrapersOnly => self.select_scraper_only(&workers, candidates).await,
        }
    }

    /// Selects worker by health score
    async fn select_by_health(
        &self,
        _workers: &WorkerRegistry,
        candidates: Vec<String>,
    ) -> Result<WorkerSelection, GhostError> {
        let mut best: Option<(String, f64)> = None;

        for id in candidates {
            let health = self.health_engine.get_health(&id).await;
            let score = health.score;

            if let Some((_, best_score)) = &best {
                if score > *best_score {
                    best = Some((id, score));
                }
            } else {
                best = Some((id, score));
            }
        }

        let (worker_id, score) = best.ok_or_else(|| GhostError::WorkersExhausted("No candidates".into()))?;
        let tier = self.determine_tier_from_score(score);

        Ok(WorkerSelection::new(worker_id, tier))
    }

    /// Selects worker by latency
    async fn select_by_latency(
        &self,
        _workers: &WorkerRegistry,
        candidates: Vec<String>,
    ) -> Result<WorkerSelection, GhostError> {
        let mut best: Option<(String, u64)> = None;

        for id in candidates {
            let health = self.health_engine.get_health(&id).await;
            let latency = health.avg_latency_ms;

            if let Some((_, best_latency)) = &best {
                if latency < *best_latency {
                    best = Some((id, latency));
                }
            } else {
                best = Some((id, latency));
            }
        }

        let (worker_id, _) = best.ok_or_else(|| GhostError::WorkersExhausted("No candidates".into()))?;
        let health = self.health_engine.get_health(&worker_id).await;

        Ok(WorkerSelection::new(worker_id, self.determine_tier_from_score(health.score)))
    }

    /// Selects worker by cost
    async fn select_by_cost(
        &self,
        workers: &WorkerRegistry,
        candidates: Vec<String>,
    ) -> Result<WorkerSelection, GhostError> {
        let mut best: Option<(String, f64)> = None;

        for id in &candidates {
            if let Some(worker) = workers.get(id) {
                // Use priority as inverse cost (higher priority = lower effective cost)
                let cost = 1.0 / (worker.manifest().priority as f64 + 1.0);

                if let Some((_, best_cost)) = &best {
                    if cost < *best_cost {
                        best = Some((id.clone(), cost));
                    }
                } else {
                    best = Some((id.clone(), cost));
                }
            }
        }

        let (worker_id, _) = best.ok_or_else(|| GhostError::WorkersExhausted("No candidates".into()))?;
        let health = self.health_engine.get_health(&worker_id).await;

        Ok(WorkerSelection::new(worker_id, self.determine_tier_from_score(health.score)))
    }

    /// Selects worker using round-robin
    async fn select_round_robin(
        &self,
        _workers: &WorkerRegistry,
        candidates: Vec<String>,
    ) -> Result<WorkerSelection, GhostError> {
        if candidates.is_empty() {
            return Err(GhostError::WorkersExhausted("No candidates".into()));
        }

        let mut counter = self.rr_counter.lock().await;
        let index = *counter % candidates.len();
        *counter = (*counter + 1) % candidates.len();

        let worker_id = candidates[index].clone();
        let health = self.health_engine.get_health(&worker_id).await;

        Ok(WorkerSelection::new(worker_id, self.determine_tier_from_score(health.score)))
    }

    /// Selects official API worker first
    async fn select_official_first(
        &self,
        workers: &WorkerRegistry,
        candidates: Vec<String>,
    ) -> Result<WorkerSelection, GhostError> {
        // First, try to find an official API worker
        for id in &candidates {
            if let Some(worker) = workers.get(id) {
                if worker.manifest().worker_type == ghost_schema::WorkerType::Official {
                    return Ok(WorkerSelection::new(id.clone(), CapabilityTier::Official));
                }
            }
        }

        // Fall back to health-based selection
        self.select_by_health(workers, candidates).await
    }

    /// Selects only official API worker
    async fn select_official_only(
        &self,
        workers: &WorkerRegistry,
        candidates: Vec<String>,
    ) -> Result<WorkerSelection, GhostError> {
        for id in &candidates {
            if let Some(worker) = workers.get(id) {
                if worker.manifest().worker_type == ghost_schema::WorkerType::Official {
                    return Ok(WorkerSelection::new(id.clone(), CapabilityTier::Official));
                }
            }
        }

        Err(GhostError::WorkersExhausted("No official API workers available".into()))
    }

    /// Selects only scraper workers
    async fn select_scraper_only(
        &self,
        workers: &WorkerRegistry,
        candidates: Vec<String>,
    ) -> Result<WorkerSelection, GhostError> {
        let scraper_candidates: Vec<String> = candidates
            .iter()
            .filter(|id| {
                workers.get(*id)
                    .map(|w| w.manifest().worker_type != ghost_schema::WorkerType::Official)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        if scraper_candidates.is_empty() {
            return Err(GhostError::WorkersExhausted("No scraper workers available".into()));
        }

        self.select_by_health(workers, scraper_candidates).await
    }

    /// Determines tier from health score
    fn determine_tier_from_score(&self, score: f64) -> CapabilityTier {
        if score >= self.config.health.healthy_threshold {
            CapabilityTier::Fast
        } else if score >= self.config.health.degraded_threshold {
            CapabilityTier::Heavy
        } else {
            CapabilityTier::Official
        }
    }

    /// Executes a request with fallback support
    async fn execute_with_fallback<T, F, Fut>(
        &self,
        execute: F,
        capability: Capability,
        platform: Platform,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<T, GhostError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, GhostError>>,
    {
        let mut attempts = 0;
        let max_attempts = self.config.max_retries;
        let mut exclude = Vec::new();

        loop {
            attempts += 1;

            match execute().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempts >= max_attempts {
                        tracing::warn!(
                            attempts = attempts,
                            error = %e,
                            "Request failed after max attempts"
                        );
                        return Err(e);
                    }

                    // Get current worker selection for exclusion
                    let selection = self.select_worker(capability, platform, ctx, strategy, &exclude).await;
                    if let Ok(sel) = selection {
                        exclude.push(sel.worker_id.clone());
                    }

                    tracing::debug!(
                        attempt = attempts,
                        max_attempts = max_attempts,
                        error = %e,
                        "Request failed, will retry"
                    );

                    // Apply jitter before retry
                    self.apply_jitter(platform).await;
                }
            }
        }
    }

    /// Applies jitter delay for a platform
    async fn apply_jitter(&self, platform: Platform) {
        let (min_ms, max_ms) = platform.jitter_range_ms();
        if min_ms > 0 || max_ms > 0 {
            let delay_ms = min_ms + (rand_jitter() % (max_ms - min_ms + 1));
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }
    }

    /// Executes get_post on a worker
    async fn execute_get_post(
        &self,
        selection: &WorkerSelection,
        platform: Platform,
        id: &str,
        ctx: &GhostContext,
    ) -> Result<GhostPost, GhostError> {
        let workers = self.workers.read().await;
        let worker = workers.get(&selection.worker_id)
            .ok_or_else(|| GhostError::ScraperError {
                worker: selection.worker_id.clone(),
                message: "Worker not found".into(),
            })?;

        // Build raw context
        let url = format!("{}/i/status/{}", platform.base_url(), id);
        let raw_ctx = self.build_raw_context(platform, &url, ctx);

        // Execute and time
        let start = std::time::Instant::now();
        let result = worker.execute(&raw_ctx).await;
        let latency = start.elapsed().as_millis() as u64;

        // Record result
        match &result {
            Ok(_) => self.health_engine.record_success(&selection.worker_id, latency).await,
            Err(_) => self.health_engine.record_failure(&selection.worker_id).await,
        }

        let payload = result?;
        self.parse_post(payload, platform).await
    }

    /// Executes get_user on a worker
    async fn execute_get_user(
        &self,
        selection: &WorkerSelection,
        platform: Platform,
        id: &str,
        ctx: &GhostContext,
    ) -> Result<GhostUser, GhostError> {
        let workers = self.workers.read().await;
        let worker = workers.get(&selection.worker_id)
            .ok_or_else(|| GhostError::ScraperError {
                worker: selection.worker_id.clone(),
                message: "Worker not found".into(),
            })?;

        let url = format!("{}/{}", platform.base_url(), id);
        let raw_ctx = self.build_raw_context(platform, &url, ctx);

        let start = std::time::Instant::now();
        let result = worker.execute(&raw_ctx).await;
        let latency = start.elapsed().as_millis() as u64;

        match &result {
            Ok(_) => self.health_engine.record_success(&selection.worker_id, latency).await,
            Err(_) => self.health_engine.record_failure(&selection.worker_id).await,
        }

        let payload = result?;
        self.parse_user(payload, platform).await
    }

    /// Executes search on a worker
    async fn execute_search(
        &self,
        selection: &WorkerSelection,
        platform: Platform,
        query: &str,
        ctx: &GhostContext,
    ) -> Result<Vec<GhostPost>, GhostError> {
        let workers = self.workers.read().await;
        let worker = workers.get(&selection.worker_id)
            .ok_or_else(|| GhostError::ScraperError {
                worker: selection.worker_id.clone(),
                message: "Worker not found".into(),
            })?;

        let url = format!("{}/search?q={}", platform.base_url(), urlencoding::encode(query));
        let raw_ctx = self.build_raw_context(platform, &url, ctx);

        let start = std::time::Instant::now();
        let result = worker.execute(&raw_ctx).await;
        let latency = start.elapsed().as_millis() as u64;

        match &result {
            Ok(_) => self.health_engine.record_success(&selection.worker_id, latency).await,
            Err(_) => self.health_engine.record_failure(&selection.worker_id).await,
        }

        let payload = result?;
        self.parse_posts(payload, platform).await
    }

    /// Executes trending on a worker
    async fn execute_trending(
        &self,
        selection: &WorkerSelection,
        platform: Platform,
        ctx: &GhostContext,
    ) -> Result<Vec<GhostPost>, GhostError> {
        let workers = self.workers.read().await;
        let worker = workers.get(&selection.worker_id)
            .ok_or_else(|| GhostError::ScraperError {
                worker: selection.worker_id.clone(),
                message: "Worker not found".into(),
            })?;

        let url = format!("{}/explore", platform.base_url());
        let raw_ctx = self.build_raw_context(platform, &url, ctx);

        let start = std::time::Instant::now();
        let result = worker.execute(&raw_ctx).await;
        let latency = start.elapsed().as_millis() as u64;

        match &result {
            Ok(_) => self.health_engine.record_success(&selection.worker_id, latency).await,
            Err(_) => self.health_engine.record_failure(&selection.worker_id).await,
        }

        let payload = result?;
        self.parse_posts(payload, platform).await
    }

    /// Executes timeline on a worker
    async fn execute_timeline(
        &self,
        selection: &WorkerSelection,
        platform: Platform,
        user_id: &str,
        ctx: &GhostContext,
    ) -> Result<Vec<GhostPost>, GhostError> {
        let workers = self.workers.read().await;
        let worker = workers.get(&selection.worker_id)
            .ok_or_else(|| GhostError::ScraperError {
                worker: selection.worker_id.clone(),
                message: "Worker not found".into(),
            })?;

        let url = format!("{}/{}", platform.base_url(), user_id);
        let raw_ctx = self.build_raw_context(platform, &url, ctx);

        let start = std::time::Instant::now();
        let result = worker.execute(&raw_ctx).await;
        let latency = start.elapsed().as_millis() as u64;

        match &result {
            Ok(_) => self.health_engine.record_success(&selection.worker_id, latency).await,
            Err(_) => self.health_engine.record_failure(&selection.worker_id).await,
        }

        let payload = result?;
        self.parse_posts(payload, platform).await
    }

    /// Builds a RawContext from a GhostContext
    fn build_raw_context(&self, platform: Platform, url: &str, ctx: &GhostContext) -> RawContext {
        let mut raw_ctx = RawContext::get(url);

        // Add platform-specific headers
        if let Some(shield) = self.config.shields.get(&platform) {
            // Add headers based on shield config
            raw_ctx = raw_ctx
                .with_header("User-Agent", &shield.header_profile)
                .with_header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8");
        }

        // Add proxy if available
        if let Some(ref proxy) = ctx.proxy {
            raw_ctx = raw_ctx.with_proxy(proxy.clone());
        }

        // Add session if available
        if let Some(ref session) = ctx.session {
            raw_ctx = raw_ctx.with_session(session.clone());
        }

        raw_ctx
    }

    /// Parses a payload into a GhostPost
    async fn parse_post(
        &self,
        payload: PayloadBlob,
        platform: Platform,
    ) -> Result<GhostPost, GhostError> {
        // Parse JSON response
        let json: serde_json::Value = payload.as_json()
            .map_err(|e| GhostError::ParseError(format!("Failed to parse JSON: {}", e)))?;

        // Extract fields from the JSON
        let id = json.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let text = json.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Build the post
        let mut post = GhostPost::new(id, platform, text);

        // Extract optional fields
        if let Some(like_count) = json.get("like_count").and_then(|v| v.as_u64()) {
            post.like_count = Some(like_count);
        }
        if let Some(repost_count) = json.get("repost_count").and_then(|v| v.as_u64()) {
            post.repost_count = Some(repost_count);
        }
        if let Some(reply_count) = json.get("reply_count").and_then(|v| v.as_u64()) {
            post.reply_count = Some(reply_count);
        }
        if let Some(view_count) = json.get("view_count").and_then(|v| v.as_u64()) {
            post.view_count = Some(view_count);
        }

        Ok(post)
    }

    /// Parses a payload into a GhostUser
    async fn parse_user(
        &self,
        payload: PayloadBlob,
        platform: Platform,
    ) -> Result<GhostUser, GhostError> {
        let json: serde_json::Value = payload.as_json()
            .map_err(|e| GhostError::ParseError(format!("Failed to parse JSON: {}", e)))?;

        let id = json.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let username = json.get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut user = GhostUser::new(id, platform, username);

        // Extract optional fields
        if let Some(display_name) = json.get("name").and_then(|v| v.as_str()) {
            user.display_name = Some(display_name.to_string());
        }
        if let Some(bio) = json.get("description").and_then(|v| v.as_str()) {
            user.bio = Some(bio.to_string());
        }
        if let Some(avatar_url) = json.get("profile_image_url").and_then(|v| v.as_str()) {
            user.avatar_url = Some(avatar_url.to_string());
        }
        if let Some(followers_count) = json.get("followers_count").and_then(|v| v.as_u64()) {
            user.followers_count = Some(followers_count);
        }
        if let Some(following_count) = json.get("following_count").and_then(|v| v.as_u64()) {
            user.following_count = Some(following_count);
        }
        if let Some(posts_count) = json.get("tweets_count").or_else(|| json.get("posts_count")).and_then(|v| v.as_u64()) {
            user.posts_count = Some(posts_count);
        }
        if let Some(is_verified) = json.get("verified").and_then(|v| v.as_bool()) {
            user.is_verified = Some(is_verified);
        }

        Ok(user)
    }

    /// Parses a payload into multiple GhostPosts
    async fn parse_posts(
        &self,
        payload: PayloadBlob,
        platform: Platform,
    ) -> Result<Vec<GhostPost>, GhostError> {
        let json: serde_json::Value = payload.as_json()
            .map_err(|e| GhostError::ParseError(format!("Failed to parse JSON: {}", e)))?;

        let posts = match json {
            serde_json::Value::Array(items) => {
                let mut result = Vec::with_capacity(items.len());
                for item in items {
                    if let Ok(post) = self.parse_single_post(item, platform) {
                        result.push(post);
                    }
                }
                result
            }
            serde_json::Value::Object(ref map) if map.contains_key("data") => {
                // Handle wrapped responses
                if let Some(data) = map.get("data") {
                    if let serde_json::Value::Array(items) = data {
                        let mut result = Vec::with_capacity(items.len());
                        for item in items {
                            if let Ok(post) = self.parse_single_post(item.clone(), platform) {
                                result.push(post);
                            }
                        }
                        result
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            _ => vec![],
        };

        Ok(posts)
    }

    /// Parses a single post from JSON
    fn parse_single_post(&self, json: serde_json::Value, platform: Platform) -> Result<GhostPost, GhostError> {
        let id = json.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let text = json.get("text")
            .or_else(|| json.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut post = GhostPost::new(id, platform, text);

        if let Some(like_count) = json.get("like_count").and_then(|v| v.as_u64()) {
            post.like_count = Some(like_count);
        }
        if let Some(repost_count) = json.get("repost_count").or_else(|| json.get("reposts_count")).and_then(|v| v.as_u64()) {
            post.repost_count = Some(repost_count);
        }
        if let Some(reply_count) = json.get("reply_count").and_then(|v| v.as_u64()) {
            post.reply_count = Some(reply_count);
        }

        Ok(post)
    }
}

/// Simple random jitter for delays
fn rand_jitter() -> u64 {
    // Simple LCG for jitter
    use std::sync::atomic::{AtomicU64, Ordering};
    static STATE: AtomicU64 = AtomicU64::new(12345);

    let mut s = STATE.load(Ordering::Relaxed);
    s = s.wrapping_mul(1103515245).wrapping_add(12345);
    STATE.store(s, Ordering::Relaxed);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_post_capability() {
        assert_eq!(GhostRouter::get_post_capability(Platform::X).unwrap(), Capability::XRead);
        assert_eq!(GhostRouter::get_post_capability(Platform::Threads).unwrap(), Capability::ThreadsRead);
    }

    #[test]
    fn test_get_user_capability() {
        assert_eq!(GhostRouter::get_user_capability(Platform::X).unwrap(), Capability::XUserRead);
        assert_eq!(GhostRouter::get_user_capability(Platform::Threads).unwrap(), Capability::ThreadsUserRead);
    }

    #[test]
    fn test_get_search_capability() {
        assert_eq!(GhostRouter::get_search_capability(Platform::X).unwrap(), Capability::XSearch);
        assert_eq!(GhostRouter::get_search_capability(Platform::Threads).unwrap(), Capability::ThreadsSearch);
    }
}
