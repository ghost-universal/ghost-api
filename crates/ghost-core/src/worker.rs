//! GhostWorker trait and worker management

use async_trait::async_trait;
use ghost_schema::{Capability, GhostError, PayloadBlob, RawContext, CapabilityManifest, Platform};

/// The trait that scrapers implement to integrate with the engine
#[async_trait]
pub trait GhostWorker: Send + Sync {
    /// Unique identifier for this worker
    fn id(&self) -> &str;

    /// Capabilities this worker supports
    fn capabilities(&self) -> Vec<Capability>;

    /// Platform(s) this worker supports
    fn platforms(&self) -> Vec<Platform>;

    /// Execute a request and return a PayloadBlob
    async fn execute(&self, ctx: &RawContext) -> Result<PayloadBlob, GhostError>;

    /// Returns the capability manifest
    fn manifest(&self) -> CapabilityManifest;

    /// Performs a health check
    async fn health_check(&self) -> Result<WorkerHealth, GhostError> {
        // TODO: Implement default health check logic
        Ok(WorkerHealth::default())
    }

    /// Returns current worker status
    fn status(&self) -> WorkerStatus {
        // TODO: Implement default status determination
        WorkerStatus::Idle
    }

    /// Returns the current load (0.0 - 1.0)
    fn load(&self) -> f64 {
        // TODO: Implement load calculation
        0.0
    }

    /// Shuts down the worker
    async fn shutdown(&self) -> Result<(), GhostError> {
        // TODO: Implement graceful shutdown
        Ok(())
    }
}

/// Worker health status
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkerHealth {
    /// Health score (0.0 - 1.0)
    pub score: f64,
    /// Success rate over recent requests
    pub success_rate: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// Whether the worker is cooling down
    pub is_cooling_down: bool,
}

impl WorkerHealth {
    /// Creates a new WorkerHealth with defaults
    pub fn new() -> Self {
        // TODO: Implement WorkerHealth construction
        Self {
            score: 1.0,
            success_rate: 1.0,
            avg_latency_ms: 0,
            consecutive_failures: 0,
            is_cooling_down: false,
        }
    }

    /// Updates health based on a successful request
    pub fn record_success(&mut self, latency_ms: u64) {
        // TODO: Implement success recording with rolling window
        self.consecutive_failures = 0;
        self.avg_latency_ms = latency_ms;
    }

    /// Updates health based on a failed request
    pub fn record_failure(&mut self) {
        // TODO: Implement failure recording with rolling window
        self.consecutive_failures += 1;
        self.success_rate = (self.success_rate * 0.9).max(0.0);
    }

    /// Calculates the health score
    pub fn calculate_score(&self, success_rate: f64, latency_ms: u64, max_latency_ms: u64) -> f64 {
        // TODO: Implement health score calculation
        // Health = (S_rate × 0.6) + (L_norm × 0.4)
        let latency_norm = 1.0 - (latency_ms as f64 / max_latency_ms as f64).min(1.0);
        (success_rate * 0.6) + (latency_norm * 0.4)
    }

    /// Returns the health tier
    pub fn tier(&self) -> HealthTier {
        // TODO: Implement tier determination
        if self.score > 0.8 {
            HealthTier::Healthy
        } else if self.score > 0.5 {
            HealthTier::Degraded
        } else if self.score > 0.0 {
            HealthTier::Unhealthy
        } else {
            HealthTier::Dead
        }
    }

    /// Returns whether this worker should be used
    pub fn is_usable(&self, threshold: f64) -> bool {
        // TODO: Implement usability check
        self.score >= threshold && !self.is_cooling_down
    }
}

impl Default for WorkerHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// Health tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthTier {
    /// Health > 0.8 - Preferred for routing
    Healthy,
    /// Health 0.5 - 0.8 - Used when healthy workers unavailable
    Degraded,
    /// Health < 0.5 - Only used as last resort
    Unhealthy,
    /// Health = 0.0 - Circuit breaker engaged
    Dead,
}

impl HealthTier {
    /// Returns the priority for this tier (higher = better)
    pub fn priority(&self) -> u8 {
        // TODO: Implement tier priority
        match self {
            HealthTier::Healthy => 3,
            HealthTier::Degraded => 2,
            HealthTier::Unhealthy => 1,
            HealthTier::Dead => 0,
        }
    }
}

/// Worker operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    /// Worker is idle and ready
    Idle,
    /// Worker is processing requests
    Busy,
    /// Worker is in cooldown mode
    CoolingDown,
    /// Worker is offline
    Offline,
    /// Worker is initializing
    Initializing,
    /// Worker encountered an error
    Error,
}

/// Registry for managing workers
pub struct WorkerRegistry {
    workers: std::collections::HashMap<String, Box<dyn GhostWorker>>,
    workers_by_capability: std::collections::HashMap<Capability, Vec<String>>,
    workers_by_platform: std::collections::HashMap<Platform, Vec<String>>,
}

impl WorkerRegistry {
    /// Creates a new empty registry
    pub fn new() -> Self {
        // TODO: Implement registry construction
        Self {
            workers: std::collections::HashMap::new(),
            workers_by_capability: std::collections::HashMap::new(),
            workers_by_platform: std::collections::HashMap::new(),
        }
    }

    /// Registers a new worker
    pub fn register(&mut self, worker: Box<dyn GhostWorker>) {
        // TODO: Implement worker registration with indexing
        let id = worker.id().to_string();
        let capabilities = worker.capabilities();
        let platforms = worker.platforms();

        // Index by capabilities
        for cap in capabilities {
            self.workers_by_capability
                .entry(cap)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        // Index by platforms
        for platform in platforms {
            self.workers_by_platform
                .entry(platform)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        self.workers.insert(id, worker);
    }

    /// Unregisters a worker
    pub fn unregister(&mut self, worker_id: &str) {
        // TODO: Implement worker unregistration with index cleanup
        self.workers.remove(worker_id);
    }

    /// Gets a worker by ID
    pub fn get(&self, worker_id: &str) -> Option<&Box<dyn GhostWorker>> {
        // TODO: Implement worker lookup
        self.workers.get(worker_id)
    }

    /// Gets all workers with a specific capability
    pub fn get_by_capability(&self, capability: Capability) -> Vec<&Box<dyn GhostWorker>> {
        // TODO: Implement capability-based worker lookup
        self.workers_by_capability
            .get(&capability)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.workers.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets all workers for a platform
    pub fn get_by_platform(&self, platform: Platform) -> Vec<&Box<dyn GhostWorker>> {
        // TODO: Implement platform-based worker lookup
        self.workers_by_platform
            .get(&platform)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.workers.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns capabilities available for a platform
    pub fn capabilities_for_platform(&self, platform: &str) -> Vec<Capability> {
        // TODO: Implement capability lookup by platform name
        Vec::new()
    }

    /// Returns the number of registered workers
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// Returns whether the registry is empty
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    /// Returns all worker IDs
    pub fn worker_ids(&self) -> impl Iterator<Item = &String> {
        self.workers.keys()
    }

    /// Returns all workers
    pub fn all_workers(&self) -> impl Iterator<Item = &Box<dyn GhostWorker>> {
        self.workers.values()
    }
}

impl Default for WorkerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Worker statistics
#[derive(Debug, Clone, Default)]
pub struct WorkerStats {
    /// Total requests handled
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average latency
    pub avg_latency_ms: u64,
    /// P95 latency
    pub p95_latency_ms: u64,
    /// Last request timestamp
    pub last_request_at: Option<std::time::Instant>,
    /// Current load (0.0 - 1.0)
    pub current_load: f64,
}

impl WorkerStats {
    /// Creates new empty stats
    pub fn new() -> Self {
        // TODO: Implement stats construction
        Self::default()
    }

    /// Records a request result
    pub fn record(&mut self, success: bool, latency_ms: u64) {
        // TODO: Implement stats recording
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
        self.last_request_at = Some(std::time::Instant::now());
    }

    /// Returns the success rate
    pub fn success_rate(&self) -> f64 {
        // TODO: Implement success rate calculation
        if self.total_requests == 0 {
            1.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }
}
