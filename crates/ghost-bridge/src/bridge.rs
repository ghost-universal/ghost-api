//! Bridge abstraction for FFI integration

use std::sync::Arc;

use ghost_schema::{GhostError, PayloadBlob, RawContext};

/// Bridge for communicating with foreign scrapers
pub trait Bridge: Send + Sync {
    /// Returns the bridge type
    fn bridge_type(&self) -> BridgeType;

    /// Initializes the bridge
    fn initialize(&mut self) -> Result<(), GhostError>;

    /// Shuts down the bridge
    fn shutdown(&mut self) -> Result<(), GhostError>;

    /// Checks if the bridge is healthy
    fn is_healthy(&self) -> bool;

    /// Returns bridge statistics
    fn stats(&self) -> BridgeStats;
}

/// Type of FFI bridge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeType {
    /// Python via PyO3
    PyO3,
    /// Node.js via NAPI
    Napi,
    /// Go via gRPC
    Grpc,
    /// Generic Unix Domain Socket
    Uds,
    /// In-process (native Rust)
    Native,
}

impl BridgeType {
    /// Returns whether this bridge requires external runtime
    pub fn requires_runtime(&self) -> bool {
        // TODO: Implement runtime requirement check
        matches!(
            self,
            BridgeType::PyO3 | BridgeType::Napi | BridgeType::Grpc
        )
    }

    /// Returns the runtime name
    pub fn runtime_name(&self) -> &'static str {
        // TODO: Implement runtime name
        match self {
            BridgeType::PyO3 => "python",
            BridgeType::Napi => "node",
            BridgeType::Grpc => "go",
            BridgeType::Uds => "uds",
            BridgeType::Native => "native",
        }
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Default)]
pub struct BridgeStats {
    /// Number of active workers
    pub active_workers: usize,
    /// Total requests handled
    pub total_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average latency in ms
    pub avg_latency_ms: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Whether the bridge is initialized
    pub is_initialized: bool,
}

impl BridgeStats {
    /// Creates new stats
    pub fn new() -> Self {
        // TODO: Implement stats construction
        Self::default()
    }

    /// Returns the success rate
    pub fn success_rate(&self) -> f64 {
        // TODO: Implement success rate calculation
        if self.total_requests == 0 {
            1.0
        } else {
            (self.total_requests - self.failed_requests) as f64 / self.total_requests as f64
        }
    }
}

/// Bridge manager for handling multiple bridges
pub struct BridgeManager {
    bridges: Vec<Arc<dyn Bridge>>,
}

impl BridgeManager {
    /// Creates a new bridge manager
    pub fn new() -> Self {
        // TODO: Implement bridge manager construction
        Self {
            bridges: Vec::new(),
        }
    }

    /// Adds a bridge
    pub fn add_bridge(&mut self, bridge: Arc<dyn Bridge>) {
        // TODO: Implement bridge addition
        self.bridges.push(bridge);
    }

    /// Initializes all bridges
    pub async fn initialize_all(&mut self) -> Result<(), GhostError> {
        // TODO: Implement bridge initialization
        Ok(())
    }

    /// Shuts down all bridges
    pub async fn shutdown_all(&mut self) -> Result<(), GhostError> {
        // TODO: Implement bridge shutdown
        Ok(())
    }

    /// Returns bridge health status
    pub fn health_status(&self) -> Vec<(BridgeType, bool)> {
        // TODO: Implement health status aggregation
        self.bridges
            .iter()
            .map(|b| (b.bridge_type(), b.is_healthy()))
            .collect()
    }
}

impl Default for BridgeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for bridge creation
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Bridge type
    pub bridge_type: BridgeType,
    /// Maximum workers
    pub max_workers: usize,
    /// Request timeout in ms
    pub timeout_ms: u64,
    /// Memory limit in MB
    pub memory_limit_mb: u64,
    /// Path to worker script/binary
    pub worker_path: Option<String>,
}

impl BridgeConfig {
    /// Creates a new bridge config
    pub fn new(bridge_type: BridgeType) -> Self {
        // TODO: Implement bridge config construction
        Self {
            bridge_type,
            max_workers: 5,
            timeout_ms: 30000,
            memory_limit_mb: 512,
            worker_path: None,
        }
    }

    /// Sets the worker path
    pub fn with_worker_path(mut self, path: impl Into<String>) -> Self {
        // TODO: Implement worker path setter
        self.worker_path = Some(path.into());
        self
    }
}
