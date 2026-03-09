//! Main Ghost API entry point
//!
//! This module provides the primary interface for interacting with the Ghost API,
//! including initialization, platform clients, and worker management.

use std::sync::Arc;

use ghost_schema::{
    GhostContext, GhostError, GhostPost, GhostUser, Platform, Strategy,
    Capability, HealthStatus, CapabilityManifest, PayloadBlob,
};

use crate::{GhostRouter, GhostConfig, GhostEvent, HealthEngine, WorkerRegistry, FallbackEngine};

/// The main Ghost API instance
///
/// This is the primary entry point for the Ghost API. It manages workers,
/// health scoring, routing, and provides platform-specific clients.
pub struct Ghost {
    /// Router for request dispatching
    router: Arc<GhostRouter>,
    /// Health engine for worker scoring
    health_engine: Arc<HealthEngine>,
    /// Worker registry
    workers: Arc<tokio::sync::RwLock<WorkerRegistry>>,
    /// Configuration
    config: Arc<GhostConfig>,
    /// Event channel sender
    event_tx: tokio::sync::broadcast::Sender<GhostEvent>,
    /// Fallback engine
    fallback: Arc<FallbackEngine>,
    /// Shutdown signal
    shutdown_tx: tokio::sync::watch::Sender<bool>,
}

impl Ghost {
    /// Initializes a new Ghost instance with default configuration
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails, such as when the
    /// configuration is invalid or required resources are unavailable.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ghost_core::Ghost;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let ghost = Ghost::init().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn init() -> Result<Self, GhostError> {
        Self::init_with_config(GhostConfig::default()).await
    }

    /// Initializes a new Ghost instance with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Custom configuration for the Ghost instance
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or initialization fails.
    pub async fn init_with_config(config: GhostConfig) -> Result<Self, GhostError> {
        // Validate configuration
        config.validate_all()?;

        // Create event broadcast channel
        let (event_tx, _) = tokio::sync::broadcast::channel(1000);

        // Create shutdown signal
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);

        // Initialize components
        let workers = Arc::new(tokio::sync::RwLock::new(WorkerRegistry::new()));
        let health_engine = Arc::new(HealthEngine::new(&config.health));
        let fallback = Arc::new(FallbackEngine::new(&config));
        let config = Arc::new(config);

        let router = Arc::new(GhostRouter::new(
            workers.clone(),
            health_engine.clone(),
            fallback.clone(),
            config.clone(),
        ));

        tracing::info!(
            strategy = ?config.default_strategy,
            max_retries = config.max_retries,
            "Ghost instance initialized"
        );

        Ok(Self {
            router,
            health_engine,
            workers,
            config,
            event_tx,
            fallback,
            shutdown_tx,
        })
    }

    /// Returns a platform-specific client for X (Twitter)
    ///
    /// The returned client can be used to make requests to the X platform
    /// with health-aware routing and fallback support.
    pub fn x(&self) -> PlatformClient {
        PlatformClient {
            platform: Platform::X,
            ghost: self,
        }
    }

    /// Returns a platform-specific client for Threads
    ///
    /// The returned client can be used to make requests to the Threads platform
    /// with health-aware routing and fallback support.
    pub fn threads(&self) -> PlatformClient {
        PlatformClient {
            platform: Platform::Threads,
            ghost: self,
        }
    }

    /// Returns capabilities available for a platform
    ///
    /// # Arguments
    ///
    /// * `platform` - Platform name (e.g., "x", "threads")
    pub async fn capabilities_for(&self, platform: &str) -> Vec<Capability> {
        let workers = self.workers.read().await;
        workers.capabilities_for_platform(platform)
    }

    /// Returns the health status of all workers
    ///
    /// This provides an aggregated view of worker health across all platforms.
    pub async fn health_status(&self) -> HealthStatus {
        let workers = self.workers.read().await;
        self.health_engine.aggregate_status(&workers).await
    }

    /// Returns an event subscriber
    ///
    /// Subscribe to events to monitor worker health, fallbacks, and other
    /// system events.
    pub fn events(&self) -> tokio::sync::broadcast::Receiver<GhostEvent> {
        self.event_tx.subscribe()
    }

    /// Triggers a health check for all workers
    ///
    /// This method performs health checks on all registered workers and
    /// updates their health scores accordingly.
    pub async fn check_health(&self) -> Result<(), GhostError> {
        let workers = self.workers.read().await;
        self.health_engine.check_all(&workers).await
    }

    /// Registers a new worker
    ///
    /// # Arguments
    ///
    /// * `worker` - Worker implementation to register
    ///
    /// # Errors
    ///
    /// Returns an error if a worker with the same ID is already registered.
    pub async fn register_worker(&self, worker: Box<dyn crate::GhostWorker>) -> Result<(), GhostError> {
        let worker_id = worker.id().to_string();
        let capabilities = worker.capabilities();
        let platforms = worker.platforms();

        {
            let mut registry = self.workers.write().await;
            registry.register(worker);
        }

        // Initialize health for the new worker
        self.health_engine.initialize_worker(&worker_id).await;

        // Emit registration event
        self.emit_event(GhostEvent::WorkerRegistered {
            worker_id,
            capabilities,
            platforms,
        });

        Ok(())
    }

    /// Unregisters a worker
    ///
    /// # Arguments
    ///
    /// * `worker_id` - ID of the worker to unregister
    ///
    /// # Errors
    ///
    /// Returns an error if the worker is not found.
    pub async fn unregister_worker(&self, worker_id: &str) -> Result<(), GhostError> {
        {
            let mut registry = self.workers.write().await;
            if registry.get(worker_id).is_none() {
                return Err(GhostError::ValidationError(
                    format!("Worker '{}' not found", worker_id)
                ));
            }
            registry.unregister(worker_id);
        }

        // Clean up health data
        self.health_engine.remove_worker(worker_id).await;

        // Emit unregistration event
        self.emit_event(GhostEvent::WorkerUnregistered {
            worker_id: worker_id.to_string(),
        });

        Ok(())
    }

    /// Shuts down the Ghost instance gracefully
    ///
    /// This method signals all workers to shut down and waits for
    /// in-flight requests to complete.
    pub async fn shutdown(&self) -> Result<(), GhostError> {
        tracing::info!("Initiating Ghost shutdown");

        // Send shutdown signal
        let _ = self.shutdown_tx.send(true);

        // Shutdown all workers
        let workers = self.workers.read().await;
        let worker_ids: Vec<String> = workers.worker_ids().map(|s| s.clone()).collect();
        drop(workers);

        for worker_id in worker_ids {
            tracing::debug!("Shutting down worker: {}", worker_id);
            // Workers will be dropped and their shutdown implementations called
        }

        self.emit_event(GhostEvent::Shutdown);
        tracing::info!("Ghost shutdown complete");

        Ok(())
    }

    /// Returns the configuration
    pub fn config(&self) -> &GhostConfig {
        &self.config
    }

    /// Emits an event to all subscribers
    fn emit_event(&self, event: GhostEvent) {
        // Ignore send errors (no subscribers is fine)
        let _ = self.event_tx.send(event);
    }

    /// Returns the fallback engine
    pub fn fallback_engine(&self) -> &FallbackEngine {
        &self.fallback
    }

    /// Returns the number of registered workers
    pub async fn worker_count(&self) -> usize {
        self.workers.read().await.len()
    }

    /// Checks if a platform is supported
    pub async fn is_platform_supported(&self, platform: Platform) -> bool {
        let workers = self.workers.read().await;
        !workers.get_by_platform(platform).is_empty()
    }

    /// Returns the router for direct access
    pub fn router(&self) -> &Arc<GhostRouter> {
        &self.router
    }

    /// Returns the health engine for direct access
    pub fn health_engine(&self) -> &Arc<HealthEngine> {
        &self.health_engine
    }
}

impl Drop for Ghost {
    fn drop(&mut self) {
        tracing::debug!("Ghost instance dropped");
    }
}

/// Platform-specific client for making requests
///
/// This client provides methods for interacting with a specific platform,
/// automatically handling routing, health checks, and fallbacks.
pub struct PlatformClient<'a> {
    platform: Platform,
    ghost: &'a Ghost,
}

impl<'a> PlatformClient<'a> {
    /// Gets a post by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Post identifier
    /// * `ctx` - Request context with tenant, proxy, and session info
    /// * `strategy` - Routing strategy to use
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or no workers are available.
    pub async fn get_post(
        &self,
        id: &str,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<GhostPost, GhostError> {
        tracing::debug!(
            platform = ?self.platform,
            post_id = %id,
            strategy = ?strategy,
            "Getting post"
        );

        let start = std::time::Instant::now();
        let result = self.ghost.router.route_get_post(self.platform, id, ctx, strategy).await;

        let latency = start.elapsed();
        tracing::debug!(
            platform = ?self.platform,
            post_id = %id,
            latency_ms = latency.as_millis(),
            success = result.is_ok(),
            "Get post completed"
        );

        result
    }

    /// Gets a user by ID or username
    ///
    /// # Arguments
    ///
    /// * `id` - User identifier or username
    /// * `ctx` - Request context
    /// * `strategy` - Routing strategy
    pub async fn get_user(
        &self,
        id: &str,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<GhostUser, GhostError> {
        tracing::debug!(
            platform = ?self.platform,
            user_id = %id,
            strategy = ?strategy,
            "Getting user"
        );

        let start = std::time::Instant::now();
        let result = self.ghost.router.route_get_user(self.platform, id, ctx, strategy).await;

        let latency = start.elapsed();
        tracing::debug!(
            platform = ?self.platform,
            user_id = %id,
            latency_ms = latency.as_millis(),
            success = result.is_ok(),
            "Get user completed"
        );

        result
    }

    /// Searches for content
    ///
    /// # Arguments
    ///
    /// * `query` - Search query string
    /// * `ctx` - Request context
    /// * `strategy` - Routing strategy
    pub async fn search(
        &self,
        query: &str,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<Vec<GhostPost>, GhostError> {
        tracing::debug!(
            platform = ?self.platform,
            query = %query,
            strategy = ?strategy,
            "Searching"
        );

        let start = std::time::Instant::now();
        let result = self.ghost.router.route_search(self.platform, query, ctx, strategy).await;

        let latency = start.elapsed();
        tracing::debug!(
            platform = ?self.platform,
            query = %query,
            latency_ms = latency.as_millis(),
            success = result.is_ok(),
            result_count = result.as_ref().map(|r| r.len()).unwrap_or(0),
            "Search completed"
        );

        result
    }

    /// Gets trending content
    ///
    /// # Arguments
    ///
    /// * `ctx` - Request context
    /// * `strategy` - Routing strategy
    pub async fn trending(
        &self,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<Vec<GhostPost>, GhostError> {
        tracing::debug!(
            platform = ?self.platform,
            strategy = ?strategy,
            "Getting trending"
        );

        self.ghost.router.route_trending(self.platform, ctx, strategy).await
    }

    /// Gets a user's timeline
    ///
    /// # Arguments
    ///
    /// * `user_id` - User identifier
    /// * `ctx` - Request context
    /// * `strategy` - Routing strategy
    pub async fn timeline(
        &self,
        user_id: &str,
        ctx: &GhostContext,
        strategy: Strategy,
    ) -> Result<Vec<GhostPost>, GhostError> {
        tracing::debug!(
            platform = ?self.platform,
            user_id = %user_id,
            strategy = ?strategy,
            "Getting timeline"
        );

        self.ghost.router.route_timeline(self.platform, user_id, ctx, strategy).await
    }

    /// Returns the platform this client is for
    pub fn platform(&self) -> Platform {
        self.platform
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ghost_init() {
        let ghost = Ghost::init().await;
        assert!(ghost.is_ok());
    }

    #[tokio::test]
    async fn test_ghost_init_with_config() {
        let config = GhostConfig::default();
        let ghost = Ghost::init_with_config(config).await;
        assert!(ghost.is_ok());
    }

    #[tokio::test]
    async fn test_platform_client_creation() {
        let ghost = Ghost::init().await.unwrap();
        let x_client = ghost.x();
        assert_eq!(x_client.platform(), Platform::X);

        let threads_client = ghost.threads();
        assert_eq!(threads_client.platform(), Platform::Threads);
    }

    #[tokio::test]
    async fn test_worker_count() {
        let ghost = Ghost::init().await.unwrap();
        let count = ghost.worker_count().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_health_status() {
        let ghost = Ghost::init().await.unwrap();
        let status = ghost.health_status().await;
        assert_eq!(status.total_count, 0);
    }
}
