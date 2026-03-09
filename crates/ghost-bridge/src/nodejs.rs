//! Node.js bridge via NAPI
//!
//! Provides FFI integration with Node.js polyglot workers via napi-rs.

use ghost_schema::{BridgeStats, BridgeType, GhostError, PayloadBlob, RawContext};

use crate::bridge::Bridge;

/// Executes a Node.js worker
pub async fn execute_nodejs_worker(
    _worker_id: &str,
    _ctx: &RawContext,
) -> Result<PayloadBlob, GhostError> {
    #[cfg(feature = "napi")]
    {
        // NAPI implementation would use napi-rs to call into Node.js code
        // The actual implementation requires a native Node.js addon
        // For now, return NotImplemented until a proper NAPI addon is created
        Err(GhostError::NotImplemented(
            "NAPI worker execution requires native addon".into(),
        ))
    }

    #[cfg(not(feature = "napi"))]
    {
        Err(GhostError::ConfigError("NAPI bridge not enabled".into()))
    }
}

/// Node.js bridge implementation
///
/// Manages communication with Node.js workers via NAPI.
/// This bridge allows calling Node.js TypeScript/JavaScript scrapers
/// from Rust as if they were native functions.
pub struct NodeBridge {
    /// Whether the bridge has been initialized
    initialized: bool,
    /// Loaded worker script paths
    scripts: Vec<String>,
    /// Bridge statistics
    stats: BridgeStats,
    /// Configuration for the bridge
    config: NodeBridgeConfig,
}

/// Configuration for NodeBridge
#[derive(Debug, Clone)]
pub struct NodeBridgeConfig {
    /// Maximum number of concurrent workers
    pub max_concurrent: usize,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Memory limit per worker in MB
    pub memory_limit_mb: u64,
}

impl Default for NodeBridgeConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            timeout_ms: 30000,
            memory_limit_mb: 256,
        }
    }
}

impl NodeBridge {
    /// Creates a new Node.js bridge with default configuration
    pub fn new() -> Result<Self, GhostError> {
        Ok(Self {
            initialized: false,
            scripts: Vec::new(),
            stats: BridgeStats::new(),
            config: NodeBridgeConfig::default(),
        })
    }

    /// Creates a new Node.js bridge with custom configuration
    pub fn with_config(config: NodeBridgeConfig) -> Result<Self, GhostError> {
        Ok(Self {
            initialized: false,
            scripts: Vec::new(),
            stats: BridgeStats::new(),
            config,
        })
    }

    /// Loads a worker script
    pub fn load_script(&mut self, script_path: &str) -> Result<(), GhostError> {
        if self.scripts.len() >= self.config.max_concurrent {
            return Err(GhostError::ConfigError(
                "Maximum number of scripts already loaded".into(),
            ));
        }
        self.scripts.push(script_path.to_string());
        self.stats.active_workers = self.scripts.len();
        Ok(())
    }

    /// Returns whether the bridge is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns the loaded scripts
    pub fn scripts(&self) -> &[String] {
        &self.scripts
    }

    /// Returns the bridge configuration
    pub fn config(&self) -> &NodeBridgeConfig {
        &self.config
    }

    /// Returns mutable access to statistics
    pub fn stats_mut(&mut self) -> &mut BridgeStats {
        &mut self.stats
    }
}

impl Bridge for NodeBridge {
    fn bridge_type(&self) -> BridgeType {
        BridgeType::Napi
    }

    fn initialize(&mut self) -> Result<(), GhostError> {
        #[cfg(feature = "napi")]
        {
            // NAPI initialization happens lazily when first called
            // The napi-rs crate handles the actual Node.js runtime setup
            tracing::info!("Initializing Node.js NAPI bridge");
        }

        self.initialized = true;
        self.stats.is_initialized = true;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), GhostError> {
        tracing::info!("Shutting down Node.js NAPI bridge");
        self.initialized = false;
        self.stats.is_initialized = false;
        self.scripts.clear();
        self.stats.active_workers = 0;
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        self.initialized
    }

    fn stats(&self) -> BridgeStats {
        self.stats.clone()
    }
}

impl Default for NodeBridge {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            initialized: false,
            scripts: Vec::new(),
            stats: BridgeStats::new(),
            config: NodeBridgeConfig::default(),
        })
    }
}

/// NAPI function registration helper
#[cfg(feature = "napi")]
pub mod napi_helpers {
    use ghost_schema::{GhostError, PayloadBlob, RawContext};

    /// Registers a worker function with NAPI
    pub fn register_worker(_name: &str) -> Result<(), GhostError> {
        // NAPI function registration would be implemented here
        // when building a native Node.js addon
        Ok(())
    }

    /// Calls a registered worker
    pub async fn call_worker(
        _name: &str,
        _context: &RawContext,
    ) -> Result<PayloadBlob, GhostError> {
        // Actual NAPI call implementation would go here
        // This requires the napi-rs generated bindings
        Err(GhostError::NotImplemented(
            "NAPI call_worker requires native addon".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_bridge_new() {
        let bridge = NodeBridge::new().unwrap();
        assert!(!bridge.is_initialized());
        assert!(bridge.scripts().is_empty());
    }

    #[test]
    fn test_node_bridge_initialize() {
        let mut bridge = NodeBridge::new().unwrap();
        assert!(bridge.initialize().is_ok());
        assert!(bridge.is_initialized());
    }

    #[test]
    fn test_node_bridge_load_script() {
        let mut bridge = NodeBridge::new().unwrap();
        bridge.initialize().unwrap();
        assert!(bridge.load_script("test.js").is_ok());
        assert_eq!(bridge.scripts().len(), 1);
    }

    #[test]
    fn test_node_bridge_shutdown() {
        let mut bridge = NodeBridge::new().unwrap();
        bridge.initialize().unwrap();
        assert!(bridge.shutdown().is_ok());
        assert!(!bridge.is_initialized());
    }

    #[test]
    fn test_node_bridge_stats() {
        let bridge = NodeBridge::new().unwrap();
        let stats = bridge.stats();
        assert_eq!(stats.active_workers, 0);
        assert!(!stats.is_initialized);
    }
}
