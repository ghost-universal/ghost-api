//! Bridge worker implementations
//!
//! Types imported from ghost-schema - the single source of truth.

use async_trait::async_trait;
use ghost_schema::{
    Capability, CapabilityManifest, CapabilityTier, GhostError,
    PayloadBlob, RawContext, Platform, BridgeType,
};
use ghost_core::GhostWorker;

/// A worker that communicates through a bridge
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
    pub fn new(id: impl Into<String>, bridge_type: BridgeType) -> Self {
        // TODO: Implement bridge worker construction
        Self {
            id: id.into(),
            bridge_type,
            capabilities: Vec::new(),
            platforms: Vec::new(),
            manifest: CapabilityManifest::new("", Vec::new()),
        }
    }

    /// Sets the capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        // TODO: Implement capabilities setter
        self.capabilities = capabilities.clone();
        self.manifest = CapabilityManifest::new(&self.id, capabilities);
        self
    }

    /// Sets the platforms
    pub fn with_platforms(mut self, platforms: Vec<Platform>) -> Self {
        // TODO: Implement platforms setter
        self.platforms = platforms;
        self
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
        // TODO: Implement bridge worker execution
        match self.bridge_type {
            BridgeType::PyO3 => {
                #[cfg(feature = "pyo3")]
                {
                    crate::python::execute_python_worker(&self.id, ctx).await
                }
                #[cfg(not(feature = "pyo3"))]
                {
                    Err(GhostError::NotImplemented("Python bridge not enabled".into()))
                }
            }
            BridgeType::Napi => {
                #[cfg(feature = "napi")]
                {
                    crate::nodejs::execute_nodejs_worker(&self.id, ctx).await
                }
                #[cfg(not(feature = "napi"))]
                {
                    Err(GhostError::NotImplemented("NAPI bridge not enabled".into()))
                }
            }
            BridgeType::Grpc => {
                Err(GhostError::NotImplemented("gRPC bridge not implemented".into()))
            }
            BridgeType::Uds => {
                Err(GhostError::NotImplemented("UDS bridge not implemented".into()))
            }
            BridgeType::Native => {
                Err(GhostError::NotImplemented("Native bridge should use direct worker".into()))
            }
        }
    }

    fn manifest(&self) -> CapabilityManifest {
        self.manifest.clone()
    }
}

/// Worker factory for creating bridge workers
pub struct WorkerFactory {
    /// Bridge type
    bridge_type: BridgeType,
    /// Worker counter
    counter: u32,
}

impl WorkerFactory {
    /// Creates a new worker factory
    pub fn new(bridge_type: BridgeType) -> Self {
        // TODO: Implement worker factory construction
        Self {
            bridge_type,
            counter: 0,
        }
    }

    /// Creates a new worker
    pub fn create_worker(&mut self, capabilities: Vec<Capability>, platforms: Vec<Platform>) -> Box<dyn GhostWorker> {
        // TODO: Implement worker creation
        self.counter += 1;
        let id = format!("{}_{}", self.bridge_type.runtime_name(), self.counter);

        Box::new(
            BridgeWorker::new(id, self.bridge_type)
                .with_capabilities(capabilities)
                .with_platforms(platforms),
        )
    }

    /// Returns the number of workers created
    pub fn worker_count(&self) -> u32 {
        self.counter
    }
}

/// Worker pool for managing bridge workers
pub struct WorkerPool {
    /// Workers in the pool
    workers: Vec<Box<dyn GhostWorker>>,
    /// Bridge type
    bridge_type: BridgeType,
    /// Maximum pool size
    max_size: usize,
}

impl WorkerPool {
    /// Creates a new worker pool
    pub fn new(bridge_type: BridgeType, max_size: usize) -> Self {
        // TODO: Implement worker pool construction
        Self {
            workers: Vec::new(),
            bridge_type,
            max_size,
        }
    }

    /// Adds a worker to the pool
    pub fn add(&mut self, worker: Box<dyn GhostWorker>) -> Result<(), GhostError> {
        // TODO: Implement worker addition
        if self.workers.len() >= self.max_size {
            return Err(GhostError::ConfigError("Worker pool full".into()));
        }
        self.workers.push(worker);
        Ok(())
    }

    /// Removes a worker from the pool
    pub fn remove(&mut self, worker_id: &str) -> Option<Box<dyn GhostWorker>> {
        // TODO: Implement worker removal
        let pos = self.workers.iter().position(|w| w.id() == worker_id)?;
        Some(self.workers.remove(pos))
    }

    /// Returns a worker by ID
    pub fn get(&self, worker_id: &str) -> Option<&Box<dyn GhostWorker>> {
        // TODO: Implement worker lookup
        self.workers.iter().find(|w| w.id() == worker_id)
    }

    /// Returns the pool size
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// Returns whether the pool is empty
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }
}
