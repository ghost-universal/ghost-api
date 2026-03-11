//! Tests for Ghost Core
//!
//! This module contains tests for the core Ghost API functionality.

use ghost_core::{Ghost, GhostConfig, GhostWorker, WorkerRegistry, HealthEngine, FallbackEngine};
use ghost_core::config::GhostConfigExt;
use ghost_schema::{
    Capability, CapabilityManifest, GhostError, PayloadBlob, PayloadContentType,
    Platform, RawContext, WorkerStatus, WorkerType, Strategy,
    CapabilityTier, FailureReason, HealthConfig,
};
use async_trait::async_trait;

// ============================================================================
// Mock Worker for Testing
// ============================================================================

struct TestWorker {
    id: String,
    capabilities: Vec<Capability>,
    platforms: Vec<Platform>,
    priority: u32,
}

impl TestWorker {
    fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            capabilities: vec![Capability::XRead, Capability::ThreadsRead],
            platforms: vec![Platform::X, Platform::Threads],
            priority: 50,
        }
    }

    fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    fn with_platforms(mut self, platforms: Vec<Platform>) -> Self {
        self.platforms = platforms;
        self
    }

    fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

#[async_trait]
impl GhostWorker for TestWorker {
    fn id(&self) -> &str {
        &self.id
    }

    fn capabilities(&self) -> Vec<Capability> {
        self.capabilities.clone()
    }

    fn platforms(&self) -> Vec<Platform> {
        self.platforms.clone()
    }

    async fn execute(&self, _ctx: &RawContext) -> Result<PayloadBlob, GhostError> {
        Ok(PayloadBlob::new(
            br#"{"id":"test","text":"Test response"}"#.to_vec(),
            PayloadContentType::Json,
        ))
    }

    fn manifest(&self) -> CapabilityManifest {
        CapabilityManifest::new(&self.id, self.capabilities.clone())
    }

    fn status(&self) -> WorkerStatus {
        WorkerStatus::Idle
    }

    fn load(&self) -> f64 {
        0.0
    }

    fn worker_type(&self) -> WorkerType {
        WorkerType::Mock
    }

    fn priority(&self) -> u32 {
        self.priority
    }
}

// ============================================================================
// Ghost Tests
// ============================================================================

mod ghost_tests {
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
    async fn test_worker_count_empty() {
        let ghost = Ghost::init().await.unwrap();
        assert_eq!(ghost.worker_count().await, 0);
    }

    #[tokio::test]
    async fn test_worker_registration() {
        let ghost = Ghost::init().await.unwrap();
        let worker = TestWorker::new("test-worker");

        let result = ghost.register_worker(Box::new(worker)).await;
        assert!(result.is_ok());
        assert_eq!(ghost.worker_count().await, 1);
    }

    #[tokio::test]
    async fn test_worker_unregistration() {
        let ghost = Ghost::init().await.unwrap();
        let worker = TestWorker::new("test-worker");

        ghost.register_worker(Box::new(worker)).await.unwrap();
        assert_eq!(ghost.worker_count().await, 1);

        let result = ghost.unregister_worker("test-worker").await;
        assert!(result.is_ok());
        assert_eq!(ghost.worker_count().await, 0);
    }

    #[tokio::test]
    async fn test_worker_unregistration_nonexistent() {
        let ghost = Ghost::init().await.unwrap();
        let result = ghost.unregister_worker("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_platform_client() {
        let ghost = Ghost::init().await.unwrap();

        let x_client = ghost.x();
        assert_eq!(x_client.platform(), Platform::X);

        let threads_client = ghost.threads();
        assert_eq!(threads_client.platform(), Platform::Threads);
    }

    #[tokio::test]
    async fn test_ghost_shutdown() {
        let ghost = Ghost::init().await.unwrap();
        let result = ghost.shutdown().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ghost_health_status() {
        let ghost = Ghost::init().await.unwrap();
        let status = ghost.health_status().await;
        assert_eq!(status.total_count, 0);
    }

    #[tokio::test]
    async fn test_ghost_capabilities_for_platform() {
        let ghost = Ghost::init().await.unwrap();
        let caps = ghost.capabilities_for("x").await;
        assert!(caps.is_empty());
    }

    #[tokio::test]
    async fn test_ghost_config_access() {
        let config = GhostConfig::new();
        let ghost = Ghost::init_with_config(config).await.unwrap();

        let config = ghost.config();
        assert_eq!(config.max_retries, 3);
    }
}

// ============================================================================
// Worker Registry Tests
// ============================================================================

mod worker_tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = WorkerRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_worker_registration_basic() {
        let mut registry = WorkerRegistry::new();
        let worker = TestWorker::new("test-worker");

        registry.register(Box::new(worker));
        assert_eq!(registry.len(), 1);
        assert!(registry.get("test-worker").is_some());
    }

    #[test]
    fn test_worker_capability_indexing() {
        let mut registry = WorkerRegistry::new();
        let worker = TestWorker::new("test-worker")
            .with_capabilities(vec![Capability::XRead, Capability::XSearch]);

        registry.register(Box::new(worker));

        let x_read_workers = registry.get_by_capability(Capability::XRead);
        assert_eq!(x_read_workers.len(), 1);

        let x_search_workers = registry.get_by_capability(Capability::XSearch);
        assert_eq!(x_search_workers.len(), 1);
    }

    #[test]
    fn test_worker_platform_filtering() {
        let mut registry = WorkerRegistry::new();
        let worker = TestWorker::new("x-only-worker")
            .with_platforms(vec![Platform::X]);

        registry.register(Box::new(worker));

        let x_workers = registry.get_by_platform(Platform::X);
        assert_eq!(x_workers.len(), 1);

        let threads_workers = registry.get_by_platform(Platform::Threads);
        assert_eq!(threads_workers.len(), 0);
    }

    #[test]
    fn test_worker_capability_and_platform() {
        let mut registry = WorkerRegistry::new();

        let worker1 = TestWorker::new("worker1")
            .with_capabilities(vec![Capability::XRead])
            .with_platforms(vec![Platform::X]);

        let worker2 = TestWorker::new("worker2")
            .with_capabilities(vec![Capability::XRead])
            .with_platforms(vec![Platform::Threads]);

        registry.register(Box::new(worker1));
        registry.register(Box::new(worker2));

        let x_platform_x_read = registry.get_by_capability_and_platform(Capability::XRead, Platform::X);
        assert_eq!(x_platform_x_read.len(), 1);
        assert_eq!(x_platform_x_read[0].id(), "worker1");

        let threads_platform_x_read = registry.get_by_capability_and_platform(Capability::XRead, Platform::Threads);
        assert_eq!(threads_platform_x_read.len(), 1);
        assert_eq!(threads_platform_x_read[0].id(), "worker2");
    }

    #[test]
    fn test_worker_unregistration() {
        let mut registry = WorkerRegistry::new();
        let worker = TestWorker::new("test-worker");

        registry.register(Box::new(worker));
        assert_eq!(registry.len(), 1);

        let removed = registry.unregister("test-worker");
        assert!(removed);
        assert_eq!(registry.len(), 0);
        assert!(registry.get("test-worker").is_none());
    }

    #[test]
    fn test_worker_unregistration_nonexistent() {
        let mut registry = WorkerRegistry::new();
        let removed = registry.unregister("nonexistent");
        assert!(!removed);
    }

    #[test]
    fn test_worker_round_robin() {
        let mut registry = WorkerRegistry::new();

        for i in 0..3 {
            let worker = TestWorker::new(format!("worker-{}", i));
            registry.register(Box::new(worker));
        }

        let id1 = registry.get_round_robin(Capability::XRead).unwrap().id().to_string();
        let id2 = registry.get_round_robin(Capability::XRead).unwrap().id().to_string();
        let id3 = registry.get_round_robin(Capability::XRead).unwrap().id().to_string();
        let id4 = registry.get_round_robin(Capability::XRead).unwrap().id().to_string();

        // Should cycle through workers
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_eq!(id1, id4); // Fourth should equal first
    }

    #[test]
    fn test_worker_priority_sorting() {
        let mut registry = WorkerRegistry::new();

        let low = TestWorker::new("low").with_priority(10);
        let high = TestWorker::new("high").with_priority(90);
        let medium = TestWorker::new("medium").with_priority(50);

        registry.register(Box::new(low));
        registry.register(Box::new(high));
        registry.register(Box::new(medium));

        let sorted = registry.get_by_capability_sorted(Capability::XRead);
        assert_eq!(sorted[0].id(), "high");
        assert_eq!(sorted[1].id(), "medium");
        assert_eq!(sorted[2].id(), "low");
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = WorkerRegistry::new();

        registry.register(Box::new(TestWorker::new("worker1")));
        registry.register(Box::new(TestWorker::new("worker2")));
        assert_eq!(registry.len(), 2);

        registry.clear();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }
}

// ============================================================================
// Health Tests
// ============================================================================

mod health_tests {
    use super::*;

    #[test]
    fn test_health_score_calculation() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        // Perfect score: 100% success, 0ms latency
        let score = engine.calculate_score(1.0, 0);
        assert!((score - 1.0).abs() < 0.01);

        // 80% success rate, 500ms latency
        // Health = (0.8 × 0.6) + (0.75 × 0.4) = 0.48 + 0.30 = 0.78
        let score = engine.calculate_score(0.8, 500);
        assert!((score - 0.78).abs() < 0.01);

        // 50% success rate, max latency
        // Health = (0.5 × 0.6) + (0.0 × 0.4) = 0.30
        let score = engine.calculate_score(0.5, 2000);
        assert!((score - 0.30).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_health_engine_initialize_worker() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("test-worker").await;

        let health = engine.get_health("test-worker").await;
        assert_eq!(health.score, 1.0);
    }

    #[tokio::test]
    async fn test_health_engine_record_success() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("test-worker").await;
        engine.record_success("test-worker", 100).await;

        let health = engine.get_health("test-worker").await;
        assert!(health.score > 0.0);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[tokio::test]
    async fn test_health_engine_record_failure() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("test-worker").await;
        engine.record_failure("test-worker").await;

        let health = engine.get_health("test-worker").await;
        assert_eq!(health.consecutive_failures, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_trip() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("test-worker").await;

        // Record failures to trip the circuit breaker
        for _ in 0..config.consecutive_failure_threshold {
            engine.record_failure("test-worker").await;
        }

        let is_open = engine.is_circuit_open("test-worker").await;
        assert!(is_open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("test-worker").await;

        engine.trip_circuit_breaker("test-worker").await;
        assert!(engine.is_circuit_open("test-worker").await);

        engine.reset_circuit_breaker("test-worker").await;
        assert!(!engine.is_circuit_open("test-worker").await);
    }

    #[tokio::test]
    async fn test_health_tier_classification() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("healthy-worker").await;
        engine.initialize_worker("unhealthy-worker").await;

        // Record successes for healthy worker
        for _ in 0..10 {
            engine.record_success("healthy-worker", 100).await;
        }

        // Record failures for unhealthy worker
        for _ in 0..5 {
            engine.record_failure("unhealthy-worker").await;
        }

        let healthy = engine.healthy_workers().await;
        let unhealthy = engine.unhealthy_workers().await;

        assert!(healthy.contains(&"healthy-worker".to_string()));
        assert!(unhealthy.contains(&"unhealthy-worker".to_string()));
    }

    #[tokio::test]
    async fn test_health_engine_stats() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("test-worker").await;

        // Record some requests
        engine.record_success("test-worker", 100).await;
        engine.record_success("test-worker", 150).await;
        engine.record_failure("test-worker").await;

        let stats = engine.get_stats("test-worker").await;
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.failed_requests, 1);
    }

    #[tokio::test]
    async fn test_health_engine_remove_worker() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("test-worker").await;
        engine.remove_worker("test-worker").await;

        let stats = engine.get_stats("test-worker").await;
        assert!(stats.is_none());
    }
}

// ============================================================================
// Fallback Tests
// ============================================================================

mod fallback_tests {
    use super::*;

    #[test]
    fn test_fallback_engine_creation() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);
        assert_eq!(engine.total_fallbacks(), 0);
    }

    #[test]
    fn test_next_tier_escalation() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        assert_eq!(engine.next_tier(CapabilityTier::Fast), Some(CapabilityTier::Heavy));
        assert_eq!(engine.next_tier(CapabilityTier::Heavy), Some(CapabilityTier::Official));
        assert_eq!(engine.next_tier(CapabilityTier::Official), None);
    }

    #[test]
    fn test_failure_reason_retryable() {
        assert!(FailureReason::RateLimited.is_retryable());
        assert!(FailureReason::WafChallenge.is_retryable());
        assert!(FailureReason::ProxyBlocked.is_retryable());
        assert!(FailureReason::Timeout.is_retryable());
        assert!(!FailureReason::SessionExpired.is_retryable());
    }

    #[test]
    fn test_failure_reason_escalation() {
        assert!(FailureReason::WafChallenge.requires_escalation());
        assert!(FailureReason::AllWorkersExhausted.requires_escalation());
        assert!(!FailureReason::RateLimited.requires_escalation());
    }

    #[test]
    fn test_fallback_engine_config() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        let _config = engine.config();
    }

    #[test]
    fn test_fallback_engine_tracker() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        let _tracker = engine.tracker();
    }
}

// ============================================================================
// Configuration Tests
// ============================================================================

mod config_tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = GhostConfig::new();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.default_strategy, Strategy::HealthFirst);
        assert_eq!(config.request_timeout_secs, 30);
    }

    #[test]
    fn test_config_validation() {
        let config = GhostConfig::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_all() {
        let config = GhostConfig::new();
        assert!(config.validate_all().is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = ghost_schema::ConfigBuilder::new()
            .strategy(Strategy::Fastest)
            .max_retries(5)
            .timeout(60)
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.default_strategy, Strategy::Fastest);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.request_timeout_secs, 60);
    }

    #[test]
    fn test_config_invalid_retries() {
        let mut config = GhostConfig::new();
        config.max_retries = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_health_config_defaults() {
        let config = HealthConfig::new();
        assert_eq!(config.healthy_threshold, 0.7);
        assert_eq!(config.degraded_threshold, 0.5);
        assert_eq!(config.max_latency_ms, 2000);
        assert_eq!(config.consecutive_failure_threshold, 5);
    }

    #[test]
    fn test_health_config_validation() {
        let config = HealthConfig::new();
        assert!(config.validate().is_ok());

        let mut invalid_config = HealthConfig::new();
        invalid_config.healthy_threshold = 1.5;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_health_config_invalid_degraded_threshold() {
        let mut config = HealthConfig::new();
        config.degraded_threshold = 1.5;
        assert!(config.validate().is_err());
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_ghost_full_workflow() {
        // Initialize Ghost
        let ghost = Ghost::init().await.unwrap();

        // Register a worker
        let worker = TestWorker::new("integration-worker")
            .with_capabilities(vec![Capability::XRead])
            .with_platforms(vec![Platform::X]);

        ghost.register_worker(Box::new(worker)).await.unwrap();

        // Check health status
        let status = ghost.health_status().await;
        assert_eq!(status.total_count, 1);

        // Check capabilities
        let caps = ghost.capabilities_for("x").await;
        assert!(!caps.is_empty());

        // Cleanup
        ghost.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_ghost_multiple_workers() {
        let ghost = Ghost::init().await.unwrap();

        // Register multiple workers
        for i in 0..3 {
            let worker = TestWorker::new(format!("worker-{}", i));
            ghost.register_worker(Box::new(worker)).await.unwrap();
        }

        assert_eq!(ghost.worker_count().await, 3);

        // Unregister one
        ghost.unregister_worker("worker-1").await.unwrap();
        assert_eq!(ghost.worker_count().await, 2);
    }

    #[tokio::test]
    async fn test_ghost_platform_support_check() {
        let ghost = Ghost::init().await.unwrap();

        // No workers, so no platforms supported
        let supported = ghost.is_platform_supported(Platform::X).await;
        assert!(!supported);

        // Register a worker for X
        let worker = TestWorker::new("x-worker")
            .with_platforms(vec![Platform::X]);
        ghost.register_worker(Box::new(worker)).await.unwrap();

        let supported = ghost.is_platform_supported(Platform::X).await;
        assert!(supported);

        let supported = ghost.is_platform_supported(Platform::Threads).await;
        assert!(!supported);
    }
}
