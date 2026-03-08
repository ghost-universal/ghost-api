//! Bridge abstraction for FFI integration
//!
//! Types imported from ghost-schema - the single source of truth.

use std::sync::Arc;

use ghost_schema::{
    GhostError, BridgeType, BridgeStats, BridgeConfig,
};

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

/// Bridge manager for handling multiple bridges
pub struct BridgeManager {
    bridges: Vec<Arc<dyn Bridge>>,
    stats: BridgeStats,
}

impl BridgeManager {
    /// Creates a new bridge manager
    pub fn new() -> Self {
        // TODO: Implement bridge manager construction
        Self {
            bridges: Vec::new(),
            stats: BridgeStats::new(),
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
        for bridge in &mut self.bridges.iter_mut() {
            // Note: Arc doesn't allow mutable access, need to refactor
        }
        self.stats.is_initialized = true;
        Ok(())
    }

    /// Shuts down all bridges
    pub async fn shutdown_all(&mut self) -> Result<(), GhostError> {
        // TODO: Implement bridge shutdown
        self.stats.is_initialized = false;
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

    /// Returns the number of bridges
    pub fn len(&self) -> usize {
        self.bridges.len()
    }

    /// Returns whether the manager is empty
    pub fn is_empty(&self) -> bool {
        self.bridges.is_empty()
    }

    /// Returns aggregated stats
    pub fn stats(&self) -> &BridgeStats {
        &self.stats
    }
}

impl Default for BridgeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new bridge from configuration
pub fn create_bridge(config: BridgeConfig) -> Result<Box<dyn Bridge>, GhostError> {
    // TODO: Implement bridge factory
    match config.bridge_type {
        BridgeType::PyO3 => {
            #[cfg(feature = "pyo3")]
            {
                Ok(Box::new(crate::python::PythonBridge::new()?))
            }
            #[cfg(not(feature = "pyo3"))]
            {
                Err(GhostError::ConfigError("Python bridge not enabled".into()))
            }
        }
        BridgeType::Napi => {
            #[cfg(feature = "napi")]
            {
                Ok(Box::new(crate::nodejs::NodeBridge::new()?))
            }
            #[cfg(not(feature = "napi"))]
            {
                Err(GhostError::ConfigError("NAPI bridge not enabled".into()))
            }
        }
        BridgeType::Grpc => {
            Err(GhostError::NotImplemented("gRPC bridge not implemented".into()))
        }
        BridgeType::Uds => {
            Err(GhostError::NotImplemented("UDS bridge not implemented".into()))
        }
        BridgeType::Native => {
            Err(GhostError::ConfigError("Native bridge should use direct worker".into()))
        }
    }
}
