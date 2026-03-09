//! Bridge worker implementations
//!
//! This module provides worker implementations that communicate through
//! FFI bridges to execute scraping tasks in foreign runtimes.

use async_trait::async_trait;
use ghost_core::GhostWorker;
use ghost_schema::{BridgeType, Capability, CapabilityManifest, GhostError, PayloadBlob, Platform, RawContext};

/// A worker that communicates through a bridge
///
/// BridgeWorker wraps communication with foreign scrapers (Python, Node.js)
/// and implements the GhostWorker trait for seamless integration with the
/// routing engine.
pub struct BridgeWorker {
    /// Worker ID
    id: String,
    /// Bridge type
    bridge_type: BridgeType,
    /// Capabilities
    capabilities: Vec<Capability>,
    /// Platforms
    platforms: Vec<Platform>,
    /// Manifest
    manifest: CapabilityManifest,
}

impl BridgeWorker {
    /// Creates a new bridge worker
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this worker
    /// * `bridge_type` - Type of FFI bridge to use
    pub fn new(id: impl Into<String>, bridge_type: BridgeType) -> Self {
        Self {
            id: id.into(),
            bridge_type,
            capabilities: Vec::new(),
            platforms: Vec::new(),
            manifest: CapabilityManifest::new("", Vec::new()),
        }
    }

    /// Sets the capabilities for this worker
    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.manifest = CapabilityManifest::new(&self.id, capabilities.clone());
        self.capabilities = capabilities;
        self
    }

    /// Sets the platforms for this worker
    pub fn with_platforms(mut self, platforms: Vec<Platform>) -> Self {
        self.platforms = platforms;
        self
    }

    /// Returns the bridge type for this worker
    pub fn bridge_type(&self) -> BridgeType {
        self.bridge_type
    }
}

#[async_trait]
impl GhostWorker for BridgeWorker {
    fn id(&self) -> &str {
        &self.id
    }

    fn capabilities(&self) -> Vec<Capability> {
        self.capabilities.clone()
    }

    fn platforms(&self) -> Vec<Platform> {
        self.platforms.clone()
    }

    async fn execute(&self, ctx: &RawContext) -> Result<PayloadBlob, GhostError> {
        match self.bridge_type {
            BridgeType::PyO3 => {
                #[cfg(feature = "pyo3")]
                {
                    crate::python::execute_python_worker(&self.id, ctx).await
                }
                #[cfg(not(feature = "pyo3"))]
                {
                    let _ = ctx;
                    Err(GhostError::ConfigError(
                        "Python bridge not enabled (compile with 'pyo3' feature)".into(),
                    ))
                }
            }
            BridgeType::Napi => {
                #[cfg(feature = "napi")]
                {
                    crate::nodejs::execute_nodejs_worker(&self.id, ctx).await
                }
                #[cfg(not(feature = "napi"))]
                {
                    let _ = ctx;
                    Err(GhostError::ConfigError(
                        "NAPI bridge not enabled (compile with 'napi' feature)".into(),
                    ))
                }
            }
            BridgeType::Grpc => {
                let _ = ctx;
                Err(GhostError::NotImplemented(
                    "gRPC bridge not yet implemented".into(),
                ))
            }
            BridgeType::Uds => {
                let _ = ctx;
                Err(GhostError::NotImplemented(
                    "UDS bridge not yet implemented".into(),
                ))
            }
            BridgeType::Native => {
                let _ = ctx;
                Err(GhostError::ConfigError(
                    "Native bridge should use direct GhostWorker implementation".into(),
                ))
            }
        }
    }

    fn manifest(&self) -> CapabilityManifest {
        self.manifest.clone()
    }
}

/// Worker factory for creating bridge workers
///
/// The factory generates workers with automatically incrementing IDs
/// based on the bridge type.
pub struct WorkerFactory {
    /// Bridge type for created workers
    bridge_type: BridgeType,
    /// Worker counter for ID generation
    counter: u32,
}

impl WorkerFactory {
    /// Creates a new worker factory for the given bridge type
    pub fn new(bridge_type: BridgeType) -> Self {
        Self {
            bridge_type,
            counter: 0,
        }
    }

    /// Creates a new worker with the given capabilities and platforms
    ///
    /// The worker ID is automatically generated as `{bridge_type}_{counter}`.
    pub fn create_worker(
        &mut self,
        capabilities: Vec<Capability>,
        platforms: Vec<Platform>,
    ) -> Box<dyn GhostWorker> {
        self.counter += 1;
        let id = format!("{}_{}", self.bridge_type.runtime_name(), self.counter);

        Box::new(
            BridgeWorker::new(id, self.bridge_type)
                .with_capabilities(capabilities)
                .with_platforms(platforms),
        )
    }

    /// Returns the number of workers created by this factory
    pub fn worker_count(&self) -> u32 {
        self.counter
    }

    /// Returns the bridge type for this factory
    pub fn bridge_type(&self) -> BridgeType {
        self.bridge_type
    }

    /// Resets the worker counter
    pub fn reset(&mut self) {
        self.counter = 0;
    }
}

/// Worker pool for managing bridge workers
///
/// The pool manages a collection of workers and provides operations
/// for adding, removing, and querying workers.
pub struct WorkerPool {
    /// Workers in the pool
    workers: Vec<Box<dyn GhostWorker>>,
    /// Maximum pool size
    max_size: usize,
}

impl WorkerPool {
    /// Creates a new worker pool with the given maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            workers: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Creates a new worker pool with default capacity
    pub fn with_default_capacity() -> Self {
        Self::new(10)
    }

    /// Adds a worker to the pool
    ///
    /// Returns an error if the pool is full.
    pub fn add(&mut self, worker: Box<dyn GhostWorker>) -> Result<(), GhostError> {
        if self.workers.len() >= self.max_size {
            return Err(GhostError::ConfigError(format!(
                "Worker pool full (max: {})",
                self.max_size
            )));
        }
        self.workers.push(worker);
        Ok(())
    }

    /// Removes a worker from the pool by ID
    ///
    /// Returns the removed worker, or None if not found.
    pub fn remove(&mut self, worker_id: &str) -> Option<Box<dyn GhostWorker>> {
        let pos = self.workers.iter().position(|w| w.id() == worker_id)?;
        Some(self.workers.remove(pos))
    }

    /// Returns a reference to a worker by ID
    pub fn get(&self, worker_id: &str) -> Option<&dyn GhostWorker> {
        self.workers.iter().find(|w| w.id() == worker_id).map(|w| w.as_ref())
    }

    /// Returns the current pool size
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// Returns whether the pool is empty
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    /// Returns whether the pool is full
    pub fn is_full(&self) -> bool {
        self.workers.len() >= self.max_size
    }

    /// Returns the maximum pool size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Returns an iterator over all workers
    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn GhostWorker>> {
        self.workers.iter()
    }

    /// Returns all worker IDs
    pub fn worker_ids(&self) -> Vec<&str> {
        self.workers.iter().map(|w| w.id()).collect()
    }

    /// Clears all workers from the pool
    pub fn clear(&mut self) {
        self.workers.clear();
    }

    /// Finds workers with a specific capability
    pub fn find_by_capability(&self, capability: Capability) -> Vec<&dyn GhostWorker> {
        self.workers
            .iter()
            .filter(|w| w.capabilities().contains(&capability))
            .map(|w| w.as_ref())
            .collect()
    }

    /// Finds workers for a specific platform
    pub fn find_by_platform(&self, platform: Platform) -> Vec<&dyn GhostWorker> {
        self.workers
            .iter()
            .filter(|w| w.platforms().contains(&platform))
            .map(|w| w.as_ref())
            .collect()
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_worker_new() {
        let worker = BridgeWorker::new("test_worker", BridgeType::PyO3);
        assert_eq!(worker.id(), "test_worker");
        assert_eq!(worker.bridge_type(), BridgeType::PyO3);
    }

    #[test]
    fn test_bridge_worker_with_capabilities() {
        let worker = BridgeWorker::new("test", BridgeType::Napi)
            .with_capabilities(vec![Capability::XRead]);
        assert_eq!(worker.capabilities().len(), 1);
    }

    #[test]
    fn test_bridge_worker_with_platforms() {
        let worker = BridgeWorker::new("test", BridgeType::PyO3)
            .with_platforms(vec![Platform::X]);
        assert_eq!(worker.platforms().len(), 1);
    }

    #[test]
    fn test_worker_factory_new() {
        let factory = WorkerFactory::new(BridgeType::PyO3);
        assert_eq!(factory.worker_count(), 0);
        assert_eq!(factory.bridge_type(), BridgeType::PyO3);
    }

    #[test]
    fn test_worker_factory_create() {
        let mut factory = WorkerFactory::new(BridgeType::PyO3);
        let worker = factory.create_worker(vec![Capability::XRead], vec![Platform::X]);
        assert_eq!(factory.worker_count(), 1);
        assert!(worker.id().starts_with("python_"));
    }

    #[test]
    fn test_worker_factory_reset() {
        let mut factory = WorkerFactory::new(BridgeType::PyO3);
        factory.create_worker(vec![], vec![]);
        factory.create_worker(vec![], vec![]);
        assert_eq!(factory.worker_count(), 2);
        factory.reset();
        assert_eq!(factory.worker_count(), 0);
    }

    #[test]
    fn test_worker_pool_new() {
        let pool = WorkerPool::new(5);
        assert!(pool.is_empty());
        assert_eq!(pool.max_size(), 5);
    }

    #[test]
    fn test_worker_pool_default() {
        let pool = WorkerPool::default();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_worker_pool_add() {
        let mut pool = WorkerPool::new(2);
        let worker = WorkerFactory::new(BridgeType::PyO3).create_worker(vec![], vec![]);
        assert!(pool.add(worker).is_ok());
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_worker_pool_full() {
        let mut pool = WorkerPool::new(1);
        let mut factory = WorkerFactory::new(BridgeType::PyO3);

        let w1 = factory.create_worker(vec![], vec![]);
        assert!(pool.add(w1).is_ok());

        let w2 = factory.create_worker(vec![], vec![]);
        assert!(pool.add(w2).is_err());
        assert!(pool.is_full());
    }

    #[test]
    fn test_worker_pool_remove() {
        let mut pool = WorkerPool::new(5);
        let mut factory = WorkerFactory::new(BridgeType::PyO3);

        let worker = factory.create_worker(vec![], vec![]);
        let id = worker.id().to_string();
        pool.add(worker).unwrap();

        let removed = pool.remove(&id);
        assert!(removed.is_some());
        assert!(pool.is_empty());
    }

    #[test]
    fn test_worker_pool_get() {
        let mut pool = WorkerPool::new(5);
        let mut factory = WorkerFactory::new(BridgeType::PyO3);

        let worker = factory.create_worker(vec![], vec![]);
        let id = worker.id().to_string();
        pool.add(worker).unwrap();

        assert!(pool.get(&id).is_some());
        assert!(pool.get("nonexistent").is_none());
    }

    #[test]
    fn test_worker_pool_clear() {
        let mut pool = WorkerPool::new(5);
        let mut factory = WorkerFactory::new(BridgeType::PyO3);

        pool.add(factory.create_worker(vec![], vec![])).unwrap();
        pool.add(factory.create_worker(vec![], vec![])).unwrap();
        assert_eq!(pool.len(), 2);

        pool.clear();
        assert!(pool.is_empty());
    }
}
