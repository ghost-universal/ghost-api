//! GhostWorker trait and worker management
//!
//! This module defines the worker trait and registry for managing
//! scraper workers of different types.

use async_trait::async_trait;
use ghost_schema::{
    Capability, GhostError, PayloadBlob, RawContext, CapabilityManifest,
    Platform, WorkerHealth, WorkerStatus, WorkerStats, HealthTier,
};

/// The trait that scrapers implement to integrate with the engine.
///
/// Workers are the core abstraction for fetching data from platforms.
/// Each worker declares its capabilities and platforms, and the engine
/// routes requests to appropriate workers based on health and strategy.
#[async_trait]
pub trait GhostWorker: Send + Sync {
    /// Unique identifier for this worker
    ///
    /// The ID must be unique across all registered workers.
    fn id(&self) -> &str;

    /// Capabilities this worker supports
    ///
    /// Returns the list of operations this worker can perform,
    /// such as reading posts, searching, or accessing user profiles.
    fn capabilities(&self) -> Vec<Capability>;

    /// Platform(s) this worker supports
    ///
    /// Returns the list of platforms this worker can interact with,
    /// such as X (Twitter) or Threads.
    fn platforms(&self) -> Vec<Platform>;

    /// Execute a request and return a PayloadBlob
    ///
    /// This is the main method for performing work. The worker receives
    /// a RawContext with all necessary information to make a request
    /// and returns the raw data for further processing.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context with URL, headers, proxy, and session info
    ///
    /// # Errors
    ///
    /// Returns `GhostError` if the request fails for any reason.
    async fn execute(&self, ctx: &RawContext) -> Result<PayloadBlob, GhostError>;

    /// Returns the capability manifest
    ///
    /// The manifest provides detailed information about the worker's
    /// capabilities, type, and configuration.
    fn manifest(&self) -> CapabilityManifest;

    /// Performs a health check
    ///
    /// This method is called periodically to verify the worker is
    /// functioning correctly. The default implementation returns a
    /// healthy status.
    async fn health_check(&self) -> Result<WorkerHealth, GhostError> {
        Ok(WorkerHealth::new())
    }

    /// Returns current worker status
    ///
    /// Indicates whether the worker is idle, busy, cooling down, or offline.
    fn status(&self) -> WorkerStatus {
        WorkerStatus::Idle
    }

    /// Returns the current load (0.0 - 1.0)
    ///
    /// A value of 0.0 means the worker is completely idle,
    /// while 1.0 means it's at maximum capacity.
    fn load(&self) -> f64 {
        0.0
    }

    /// Shuts down the worker
    ///
    /// Called when the worker should release all resources.
    async fn shutdown(&self) -> Result<(), GhostError> {
        Ok(())
    }

    /// Returns the worker type
    fn worker_type(&self) -> ghost_schema::WorkerType {
        ghost_schema::WorkerType::Unknown
    }

    /// Returns the priority for this worker (higher = preferred)
    fn priority(&self) -> u32 {
        50
    }
}

/// Registry for managing workers
///
/// The registry maintains workers indexed by ID, capability, and platform
/// for efficient lookup during request routing.
pub struct WorkerRegistry {
    /// Workers indexed by ID
    workers: std::collections::HashMap<String, Box<dyn GhostWorker>>,
    /// Worker IDs indexed by capability
    workers_by_capability: std::collections::HashMap<Capability, Vec<String>>,
    /// Worker IDs indexed by platform
    workers_by_platform: std::collections::HashMap<Platform, Vec<String>>,
    /// Round-robin counters per capability
    rr_counters: std::collections::HashMap<Capability, usize>,
}

impl WorkerRegistry {
    /// Creates a new empty registry
    pub fn new() -> Self {
        Self {
            workers: std::collections::HashMap::new(),
            workers_by_capability: std::collections::HashMap::new(),
            workers_by_platform: std::collections::HashMap::new(),
            rr_counters: std::collections::HashMap::new(),
        }
    }

    /// Registers a new worker
    ///
    /// If a worker with the same ID already exists, it will be replaced.
    /// The worker is indexed by all its capabilities and platforms.
    pub fn register(&mut self, worker: Box<dyn GhostWorker>) {
        let id = worker.id().to_string();
        let capabilities = worker.capabilities();
        let platforms = worker.platforms();

        // Remove old indexes if updating
        if self.workers.contains_key(&id) {
            self.remove_indexes(&id);
        }

        // Index by capabilities
        for cap in &capabilities {
            self.workers_by_capability
                .entry(*cap)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        // Index by platforms
        for platform in &platforms {
            self.workers_by_platform
                .entry(*platform)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        // Store the worker
        self.workers.insert(id, worker);

        tracing::debug!(
            capabilities = ?capabilities,
            platforms = ?platforms,
            "Worker registered"
        );
    }

    /// Removes indexes for a worker ID
    fn remove_indexes(&mut self, worker_id: &str) {
        for ids in self.workers_by_capability.values_mut() {
            ids.retain(|id| id != worker_id);
        }
        for ids in self.workers_by_platform.values_mut() {
            ids.retain(|id| id != worker_id);
        }
    }

    /// Unregisters a worker
    ///
    /// Returns true if the worker was found and removed.
    pub fn unregister(&mut self, worker_id: &str) -> bool {
        if let Some(_worker) = self.workers.remove(worker_id) {
            self.remove_indexes(worker_id);
            tracing::debug!(worker_id = %worker_id, "Worker unregistered");
            true
        } else {
            false
        }
    }

    /// Gets a worker by ID
    pub fn get(&self, worker_id: &str) -> Option<&Box<dyn GhostWorker>> {
        self.workers.get(worker_id)
    }

    /// Gets all workers with a specific capability
    pub fn get_by_capability(&self, capability: Capability) -> Vec<&Box<dyn GhostWorker>> {
        self.workers_by_capability
            .get(&capability)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.workers.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets worker IDs with a specific capability
    pub fn get_ids_by_capability(&self, capability: Capability) -> Vec<String> {
        self.workers_by_capability
            .get(&capability)
            .cloned()
            .unwrap_or_default()
    }

    /// Gets all workers for a platform
    pub fn get_by_platform(&self, platform: Platform) -> Vec<&Box<dyn GhostWorker>> {
        self.workers_by_platform
            .get(&platform)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.workers.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets workers with capability and platform
    pub fn get_by_capability_and_platform(
        &self,
        capability: Capability,
        platform: Platform,
    ) -> Vec<&Box<dyn GhostWorker>> {
        self.workers_by_capability
            .get(&capability)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.workers.get(id))
                    .filter(|w| w.platforms().contains(&platform))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets worker IDs with capability and platform
    pub fn get_ids_by_capability_and_platform(
        &self,
        capability: Capability,
        platform: Platform,
    ) -> Vec<String> {
        self.workers_by_capability
            .get(&capability)
            .map(|ids| {
                ids.iter()
                    .filter(|id| {
                        self.workers
                            .get(*id)
                            .map(|w| w.platforms().contains(&platform))
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns capabilities available for a platform name
    pub fn capabilities_for_platform(&self, platform: &str) -> Vec<Capability> {
        let platform = Platform::from_str(platform);
        if platform == Platform::Unknown {
            return Vec::new();
        }

        Capability::for_platform(platform)
            .into_iter()
            .filter(|cap| !self.get_by_capability_and_platform(*cap, platform).is_empty())
            .collect()
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

    /// Gets the next worker in round-robin order for a capability
    pub fn get_round_robin(&mut self, capability: Capability) -> Option<&Box<dyn GhostWorker>> {
        let ids = self.workers_by_capability.get(&capability)?;
        if ids.is_empty() {
            return None;
        }

        let counter = self.rr_counters.entry(capability).or_insert(0);
        let index = *counter % ids.len();
        *counter = (*counter + 1) % ids.len();

        self.workers.get(&ids[index])
    }

    /// Filters workers by excluding specific IDs
    pub fn filter_excluding(&self, capability: Capability, exclude: &[String]) -> Vec<&Box<dyn GhostWorker>> {
        self.get_by_capability(capability)
            .into_iter()
            .filter(|w| !exclude.contains(&w.id().to_string()))
            .collect()
    }

    /// Returns workers sorted by priority (highest first)
    pub fn get_by_capability_sorted(&self, capability: Capability) -> Vec<&Box<dyn GhostWorker>> {
        let mut workers = self.get_by_capability(capability);
        workers.sort_by(|a, b| b.priority().cmp(&a.priority()));
        workers
    }

    /// Clears all workers from the registry
    pub fn clear(&mut self) {
        self.workers.clear();
        self.workers_by_capability.clear();
        self.workers_by_platform.clear();
        self.rr_counters.clear();
    }
}

impl Default for WorkerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = WorkerRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut registry = WorkerRegistry::new();
        // Registry should start empty
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = WorkerRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_capabilities_for_platform() {
        let registry = WorkerRegistry::new();
        let caps = registry.capabilities_for_platform("x");
        assert!(caps.is_empty());
    }
}
