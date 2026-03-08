//! GhostWorker trait and worker management
//!
//! This module defines the worker trait and registry for managing
//! scraper workers of different types.

use async_trait::async_trait;
use ghost_schema::{
    Capability, GhostError, PayloadBlob, RawContext, CapabilityManifest,
    Platform, WorkerHealth, WorkerStatus, WorkerStats, HealthTier,
};

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
        Ok(WorkerHealth::new())
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
