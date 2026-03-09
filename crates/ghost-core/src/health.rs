//! Health scoring and circuit breaking logic
//!
//! This module manages worker health scores and circuit breakers
//! for resilient request routing.

use std::collections::VecDeque;
use std::collections::HashMap;
use std::sync::Arc;

use ghost_schema::{
    Capability, GhostError, Platform, WorkerHealth, WorkerStats,
    CircuitBreaker, HealthConfig, HealthStatus,
    PlatformHealthStatus,
};

use crate::WorkerRegistry;

/// Rolling window for tracking request results
#[derive(Debug, Clone)]
struct RollingWindow {
    /// Results in the window (true = success)
    results: VecDeque<bool>,
    /// Latencies in the window (ms)
    latencies: VecDeque<u64>,
    /// Maximum window size
    max_size: usize,
}

impl RollingWindow {
    fn new(max_size: usize) -> Self {
        Self {
            results: VecDeque::with_capacity(max_size),
            latencies: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn record(&mut self, success: bool, latency_ms: u64) {
        if self.results.len() >= self.max_size {
            self.results.pop_front();
            self.latencies.pop_front();
        }
        self.results.push_back(success);
        self.latencies.push_back(latency_ms);
    }

    fn success_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 1.0;
        }
        let successes = self.results.iter().filter(|&&s| s).count();
        successes as f64 / self.results.len() as f64
    }

    fn avg_latency(&self) -> u64 {
        if self.latencies.is_empty() {
            return 0;
        }
        self.latencies.iter().sum::<u64>() / self.latencies.len() as u64
    }

    fn p95_latency(&self) -> u64 {
        if self.latencies.is_empty() {
            return 0;
        }
        let mut sorted: Vec<u64> = self.latencies.iter().copied().collect();
        sorted.sort();
        let index = (sorted.len() as f64 * 0.95) as usize;
        sorted.get(index.min(sorted.len() - 1)).copied().unwrap_or(0)
    }
}

/// Health engine for managing worker health scores
pub struct HealthEngine {
    config: HealthConfig,
    health_scores: Arc<tokio::sync::RwLock<HashMap<String, WorkerHealth>>>,
    worker_stats: Arc<tokio::sync::RwLock<HashMap<String, WorkerStats>>>,
    circuit_breakers: Arc<tokio::sync::RwLock<HashMap<String, CircuitBreaker>>>,
    rolling_windows: Arc<tokio::sync::RwLock<HashMap<String, RollingWindow>>>,
}

impl HealthEngine {
    /// Creates a new health engine
    pub fn new(config: &HealthConfig) -> Self {
        Self {
            config: config.clone(),
            health_scores: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            worker_stats: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            rolling_windows: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Initializes a new worker in the health engine
    pub async fn initialize_worker(&self, worker_id: &str) {
        let mut scores = self.health_scores.write().await;
        scores.insert(worker_id.to_string(), WorkerHealth::new());

        let mut stats = self.worker_stats.write().await;
        stats.insert(worker_id.to_string(), WorkerStats::new());

        let mut windows = self.rolling_windows.write().await;
        windows.insert(worker_id.to_string(), RollingWindow::new(self.config.stats_window_size));

        tracing::debug!(worker_id = %worker_id, "Worker health initialized");
    }

    /// Removes a worker from health tracking
    pub async fn remove_worker(&self, worker_id: &str) {
        self.health_scores.write().await.remove(worker_id);
        self.worker_stats.write().await.remove(worker_id);
        self.circuit_breakers.write().await.remove(worker_id);
        self.rolling_windows.write().await.remove(worker_id);

        tracing::debug!(worker_id = %worker_id, "Worker health removed");
    }

    /// Gets the health score for a worker
    pub async fn get_health(&self, worker_id: &str) -> WorkerHealth {
        self.health_scores
            .read()
            .await
            .get(worker_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Updates health score for a worker
    pub async fn update_health(&self, worker_id: &str, health: WorkerHealth) {
        self.health_scores.write().await.insert(worker_id.to_string(), health);
    }

    /// Records a successful request
    pub async fn record_success(&self, worker_id: &str, latency_ms: u64) {
        // Update rolling window
        {
            let mut windows = self.rolling_windows.write().await;
            if let Some(window) = windows.get_mut(worker_id) {
                window.record(true, latency_ms);
            }
        }

        // Update health score
        {
            let mut scores = self.health_scores.write().await;
            if let Some(health) = scores.get_mut(worker_id) {
                health.record_success(latency_ms);
                health.consecutive_failures = 0;
                health.is_cooling_down = false;
            }
        }

        // Update stats
        {
            let mut stats = self.worker_stats.write().await;
            if let Some(stat) = stats.get_mut(worker_id) {
                stat.record(true, latency_ms);
            }
        }

        // Record probe success if in half-open state
        {
            let mut breakers = self.circuit_breakers.write().await;
            if let Some(breaker) = breakers.get_mut(worker_id) {
                breaker.record_probe_success();
            }
        }

        // Recalculate health score
        self.recalculate_score(worker_id).await;
    }

    /// Records a failed request
    pub async fn record_failure(&self, worker_id: &str) {
        // Update rolling window
        {
            let mut windows = self.rolling_windows.write().await;
            if let Some(window) = windows.get_mut(worker_id) {
                window.record(false, 0);
            }
        }

        // Update health score
        let should_trip = {
            let mut scores = self.health_scores.write().await;
            if let Some(health) = scores.get_mut(worker_id) {
                health.record_failure();
                health.consecutive_failures >= self.config.consecutive_failure_threshold
            } else {
                false
            }
        };

        // Update stats
        {
            let mut stats = self.worker_stats.write().await;
            if let Some(stat) = stats.get_mut(worker_id) {
                stat.record(false, 0);
            }
        }

        // Record probe failure if in half-open state
        {
            let mut breakers = self.circuit_breakers.write().await;
            if let Some(breaker) = breakers.get_mut(worker_id) {
                breaker.record_probe_failure();
            }
        }

        // Trip circuit breaker if needed
        if should_trip {
            self.trip_circuit_breaker(worker_id).await;
        }

        // Recalculate health score
        self.recalculate_score(worker_id).await;
    }

    /// Recalculates health score from rolling window
    async fn recalculate_score(&self, worker_id: &str) {
        let windows = self.rolling_windows.read().await;
        if let Some(window) = windows.get(worker_id) {
            let success_rate = window.success_rate();
            let avg_latency = window.avg_latency();

            let score = self.calculate_score(success_rate, avg_latency);

            let mut scores = self.health_scores.write().await;
            if let Some(health) = scores.get_mut(worker_id) {
                health.score = score;
                health.success_rate = success_rate;
                health.avg_latency_ms = avg_latency;
            }
        }
    }

    /// Trips the circuit breaker for a worker
    pub async fn trip_circuit_breaker(&self, worker_id: &str) {
        let mut breakers = self.circuit_breakers.write().await;
        let breaker = breakers
            .entry(worker_id.to_string())
            .or_insert_with(|| CircuitBreaker::new(self.config.circuit_breaker_timeout_secs));
        breaker.trip();

        // Mark worker as cooling down
        drop(breakers);
        let mut scores = self.health_scores.write().await;
        if let Some(health) = scores.get_mut(worker_id) {
            health.is_cooling_down = true;
        }

        tracing::warn!(worker_id = %worker_id, "Circuit breaker tripped");
    }

    /// Resets the circuit breaker for a worker
    pub async fn reset_circuit_breaker(&self, worker_id: &str) {
        {
            let mut breakers = self.circuit_breakers.write().await;
            if let Some(breaker) = breakers.get_mut(worker_id) {
                breaker.reset();
            }
        }

        let mut scores = self.health_scores.write().await;
        if let Some(health) = scores.get_mut(worker_id) {
            health.is_cooling_down = false;
        }

        tracing::info!(worker_id = %worker_id, "Circuit breaker reset");
    }

    /// Checks if circuit breaker is open for a worker
    pub async fn is_circuit_open(&self, worker_id: &str) -> bool {
        let breakers = self.circuit_breakers.read().await;
        breakers
            .get(worker_id)
            .map(|b| b.is_open())
            .unwrap_or(false)
    }

    /// Returns all healthy workers
    pub async fn healthy_workers(&self) -> Vec<String> {
        let scores = self.health_scores.read().await;
        scores
            .iter()
            .filter(|(_, h)| h.score >= self.config.healthy_threshold && !h.is_cooling_down)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Returns all degraded workers
    pub async fn degraded_workers(&self) -> Vec<String> {
        let scores = self.health_scores.read().await;
        scores
            .iter()
            .filter(|(_, h)| {
                h.score < self.config.healthy_threshold
                    && h.score >= self.config.degraded_threshold
                    && !h.is_cooling_down
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Returns all unhealthy workers
    pub async fn unhealthy_workers(&self) -> Vec<String> {
        let scores = self.health_scores.read().await;
        scores
            .iter()
            .filter(|(_, h)| h.score < self.config.degraded_threshold || h.is_cooling_down)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Performs health checks on all workers
    pub async fn check_all(&self, registry: &WorkerRegistry) -> Result<(), GhostError> {
        let worker_ids: Vec<String> = registry.worker_ids().map(|s| s.clone()).collect();

        for worker_id in worker_ids {
            if let Some(worker) = registry.get(&worker_id) {
                match worker.health_check().await {
                    Ok(health) => {
                        self.update_health(&worker_id, health).await;
                        tracing::debug!(worker_id = %worker_id, "Health check passed");
                    }
                    Err(e) => {
                        tracing::warn!(worker_id = %worker_id, error = %e, "Health check failed");
                        self.record_failure(&worker_id).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Returns aggregated health status
    pub fn status(&self) -> HealthStatus {
        HealthStatus::new()
    }

    /// Returns aggregated health status from registry
    pub async fn aggregate_status(&self, registry: &WorkerRegistry) -> HealthStatus {
        let scores = self.health_scores.read().await;

        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        let mut total_score = 0.0;

        for worker_id in registry.worker_ids() {
            if let Some(health) = scores.get(worker_id) {
                total_score += health.score;

                if health.is_cooling_down {
                    unhealthy_count += 1;
                } else if health.score >= self.config.healthy_threshold {
                    healthy_count += 1;
                } else if health.score >= self.config.degraded_threshold {
                    degraded_count += 1;
                } else {
                    unhealthy_count += 1;
                }
            }
        }

        let total_count = registry.len();
        let avg_score = if total_count > 0 {
            total_score / total_count as f64
        } else {
            0.0
        };

        // Build platform status
        let mut platform_status = HashMap::new();
        for platform in [Platform::X, Platform::Threads] {
            let workers = registry.get_by_platform(platform);
            if !workers.is_empty() {
                let mut platform_score = 0.0;
                let mut platform_latency = 0;

                for worker in &workers {
                    if let Some(health) = scores.get(worker.id()) {
                        platform_score += health.score;
                        platform_latency += health.avg_latency_ms as usize;
                    }
                }

                let count = workers.len();
                platform_status.insert(platform, PlatformHealthStatus {
                    platform,
                    available_workers: count,
                    avg_latency_ms: (platform_latency / count) as u64,
                    health_score: platform_score / count as f64,
                });
            }
        }

        HealthStatus {
            healthy_count,
            degraded_count,
            unhealthy_count,
            total_count,
            avg_score,
            platform_status,
        }
    }

    /// Gets the top N healthy workers for a capability
    pub async fn get_top_workers(&self, capability: Capability, registry: &WorkerRegistry, n: usize) -> Vec<String> {
        let candidates = registry.get_ids_by_capability(capability);
        let scores = self.health_scores.read().await;

        let mut scored: Vec<(String, f64)> = candidates
            .into_iter()
            .filter_map(|id| {
                scores.get(&id).map(|h| (id, h.score))
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().take(n).map(|(id, _)| id).collect()
    }

    /// Calculates the health score using the algorithm
    ///
    /// Formula: Health = (S_rate × 0.6) + (L_norm × 0.4)
    ///
    /// Where:
    /// - S_rate: Success rate over recent requests (0.0 - 1.0)
    /// - L_norm: Normalized latency (fast = 1.0, slow = 0.0)
    pub fn calculate_score(&self, success_rate: f64, latency_ms: u64) -> f64 {
        // Normalize latency (0ms = 1.0, max_latency = 0.0)
        let latency_norm = 1.0 - (latency_ms as f64 / self.config.max_latency_ms as f64).min(1.0);

        // Apply weighted formula
        (success_rate * 0.6) + (latency_norm * 0.4)
    }

    /// Gets worker stats
    pub async fn get_stats(&self, worker_id: &str) -> Option<WorkerStats> {
        self.worker_stats.read().await.get(worker_id).cloned()
    }

    /// Gets detailed stats including rolling window data
    pub async fn get_detailed_stats(&self, worker_id: &str) -> Option<DetailedStats> {
        let health = self.get_health(worker_id).await;
        let stats = self.get_stats(worker_id).await;

        let (p95_latency, success_rate, avg_latency) = {
            let windows = self.rolling_windows.read().await;
            if let Some(window) = windows.get(worker_id) {
                (window.p95_latency(), window.success_rate(), window.avg_latency())
            } else {
                (0, 1.0, 0)
            }
        };

        Some(DetailedStats {
            worker_id: worker_id.to_string(),
            health,
            stats: stats.unwrap_or_default(),
            p95_latency,
            window_success_rate: success_rate,
            window_avg_latency: avg_latency,
            is_circuit_open: self.is_circuit_open(worker_id).await,
        })
    }

    /// Returns the health configuration
    pub fn config(&self) -> &HealthConfig {
        &self.config
    }
}

/// Detailed statistics for a worker
#[derive(Debug, Clone)]
pub struct DetailedStats {
    /// Worker ID
    pub worker_id: String,
    /// Health information
    pub health: WorkerHealth,
    /// Aggregate statistics
    pub stats: WorkerStats,
    /// P95 latency from rolling window
    pub p95_latency: u64,
    /// Success rate from rolling window
    pub window_success_rate: f64,
    /// Average latency from rolling window
    pub window_avg_latency: u64,
    /// Whether circuit breaker is open
    pub is_circuit_open: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_window() {
        let mut window = RollingWindow::new(5);

        window.record(true, 100);
        window.record(true, 150);
        window.record(false, 0);
        window.record(true, 200);

        assert_eq!(window.success_rate(), 0.75);
        // Avg latency includes all entries including the failed one with 0 latency
        // (100 + 150 + 0 + 200) / 4 = 112
        assert_eq!(window.avg_latency(), 112);
    }

    #[test]
    fn test_rolling_window_overflow() {
        let mut window = RollingWindow::new(3);

        window.record(true, 100);
        window.record(true, 150);
        window.record(true, 200);
        window.record(true, 250);

        // Should have dropped the first entry
        assert_eq!(window.results.len(), 3);
    }

    #[tokio::test]
    async fn test_health_engine_creation() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);
        assert!(engine.healthy_workers().await.is_empty());
    }

    #[test]
    fn test_calculate_score() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        // Perfect health
        let score = engine.calculate_score(1.0, 0);
        assert!((score - 1.0).abs() < 0.01);

        // 80% success rate, 500ms latency (normalized to 0.75 for 2000ms max)
        let score = engine.calculate_score(0.8, 500);
        // = 0.8 * 0.6 + 0.75 * 0.4 = 0.48 + 0.3 = 0.78
        assert!((score - 0.78).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_initialize_worker() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("test-worker").await;
        let health = engine.get_health("test-worker").await;
        assert_eq!(health.score, 1.0);
    }

    #[tokio::test]
    async fn test_record_success() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("test-worker").await;
        engine.record_success("test-worker", 100).await;

        let health = engine.get_health("test-worker").await;
        assert!(health.score > 0.0);
    }

    #[tokio::test]
    async fn test_record_failure() {
        let config = HealthConfig::new();
        let engine = HealthEngine::new(&config);

        engine.initialize_worker("test-worker").await;
        engine.record_failure("test-worker").await;

        let health = engine.get_health("test-worker").await;
        assert!(health.consecutive_failures > 0);
    }
}
