//! Bridge abstraction for FFI integration
//!
//! This module provides the core bridge abstraction for communicating with
//! foreign scrapers (Python, Node.js, Go) via various FFI mechanisms.

use ghost_schema::{BridgeConfig, BridgeStats, BridgeType, GhostError};

/// Bridge for communicating with foreign scrapers
///
/// This trait defines the interface for FFI bridges that allow Rust code
/// to communicate with scrapers written in other languages.
pub trait Bridge: Send + Sync {
    /// Returns the bridge type
    fn bridge_type(&self) -> BridgeType;

    /// Initializes the bridge
    ///
    /// This method should be called before any worker operations.
    /// It sets up the foreign runtime (Python, Node.js, etc.) if needed.
    fn initialize(&mut self) -> Result<(), GhostError>;

    /// Shuts down the bridge
    ///
    /// This method releases all resources and shuts down the foreign runtime.
    fn shutdown(&mut self) -> Result<(), GhostError>;

    /// Checks if the bridge is healthy
    ///
    /// Returns true if the bridge is initialized and ready for operations.
    fn is_healthy(&self) -> bool;

    /// Returns bridge statistics
    fn stats(&self) -> BridgeStats;
}

/// Bridge manager for handling multiple bridges
///
/// The BridgeManager coordinates multiple FFI bridges, providing a unified
/// interface for initialization, health checking, and statistics aggregation.
pub struct BridgeManager {
    /// Bridges stored with their mutable state
    bridges: Vec<Box<dyn Bridge>>,
    /// Aggregated statistics
    stats: BridgeStats,
}

impl BridgeManager {
    /// Creates a new bridge manager
    pub fn new() -> Self {
        Self {
            bridges: Vec::new(),
            stats: BridgeStats::new(),
        }
    }

    /// Adds a bridge to the manager
    ///
    /// The bridge will be initialized when `initialize_all` is called.
    pub fn add_bridge(&mut self, bridge: Box<dyn Bridge>) {
        self.bridges.push(bridge);
    }

    /// Initializes all bridges
    ///
    /// This method initializes all registered bridges in sequence.
    /// If a bridge fails to initialize, an error is returned but
    /// already initialized bridges remain initialized.
    pub async fn initialize_all(&mut self) -> Result<(), GhostError> {
        let mut errors = Vec::new();

        for (i, bridge) in self.bridges.iter_mut().enumerate() {
            if let Err(e) = bridge.initialize() {
                tracing::error!("Failed to initialize bridge {}: {}", i, e);
                errors.push((i, e.to_string()));
            }
        }

        self.stats.is_initialized = errors.is_empty();
        self.stats.active_workers = self.healthy_count();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(GhostError::ConfigError(format!(
                "Failed to initialize {} bridge(s): {:?}",
                errors.len(),
                errors
            )))
        }
    }

    /// Shuts down all bridges
    ///
    /// This method attempts to shut down all bridges, even if some fail.
    /// All errors are collected and returned.
    pub async fn shutdown_all(&mut self) -> Result<(), GhostError> {
        let mut errors = Vec::new();

        for (i, bridge) in self.bridges.iter_mut().enumerate() {
            if let Err(e) = bridge.shutdown() {
                tracing::error!("Failed to shutdown bridge {}: {}", i, e);
                errors.push((i, e.to_string()));
            }
        }

        self.stats.is_initialized = false;
        self.stats.active_workers = 0;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(GhostError::ConfigError(format!(
                "Failed to shutdown {} bridge(s): {:?}",
                errors.len(),
                errors
            )))
        }
    }

    /// Returns bridge health status for all bridges
    pub fn health_status(&self) -> Vec<(BridgeType, bool)> {
        self.bridges
            .iter()
            .map(|b| (b.bridge_type(), b.is_healthy()))
            .collect()
    }

    /// Returns the number of healthy bridges
    pub fn healthy_count(&self) -> usize {
        self.bridges.iter().filter(|b| b.is_healthy()).count()
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

    /// Gets a bridge by index
    pub fn get(&self, index: usize) -> Option<&dyn Bridge> {
        self.bridges.get(index).map(|b| b.as_ref())
    }

    /// Gets a bridge by type (returns first match)
    pub fn get_by_type(&self, bridge_type: BridgeType) -> Option<&dyn Bridge> {
        self.bridges
            .iter()
            .find(|b| b.bridge_type() == bridge_type)
            .map(|b| b.as_ref())
    }

    /// Removes a bridge by index
    pub fn remove(&mut self, index: usize) -> Option<Box<dyn Bridge>> {
        if index < self.bridges.len() {
            Some(self.bridges.remove(index))
        } else {
            None
        }
    }

    /// Clears all bridges
    pub fn clear(&mut self) {
        self.bridges.clear();
        self.stats = BridgeStats::new();
    }

    /// Aggregates statistics from all bridges
    pub fn aggregate_stats(&self) -> BridgeStats {
        let mut aggregated = BridgeStats::new();
        let mut total_requests = 0u64;
        let mut total_failed = 0u64;
        let mut total_latency = 0u64;
        let mut count = 0u64;

        for bridge in &self.bridges {
            let stats = bridge.stats();
            total_requests += stats.total_requests;
            total_failed += stats.failed_requests;
            total_latency += stats.avg_latency_ms;
            count += 1;
        }

        aggregated.active_workers = self.healthy_count();
        aggregated.total_requests = total_requests;
        aggregated.failed_requests = total_failed;
        aggregated.avg_latency_ms = if count > 0 {
            total_latency / count
        } else {
            0
        };
        aggregated.is_initialized = self.stats.is_initialized;

        aggregated
    }
}

impl Default for BridgeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new bridge from configuration
///
/// This factory function creates the appropriate bridge type based on
/// the configuration. The bridge must be initialized before use.
///
/// # Arguments
///
/// * `config` - Bridge configuration specifying type and parameters
///
/// # Returns
///
/// Returns a boxed bridge instance or an error if the bridge type
/// is not supported or not enabled.
pub fn create_bridge(config: BridgeConfig) -> Result<Box<dyn Bridge>, GhostError> {
    match config.bridge_type {
        BridgeType::PyO3 => {
            #[cfg(feature = "pyo3")]
            {
                let bridge = crate::python::PythonBridge::new()?;
                Ok(Box::new(bridge))
            }
            #[cfg(not(feature = "pyo3"))]
            {
                Err(GhostError::ConfigError(
                    "Python bridge not enabled (compile with 'pyo3' feature)".into(),
                ))
            }
        }
        BridgeType::Napi => {
            #[cfg(feature = "napi")]
            {
                let bridge = crate::nodejs::NodeBridge::new()?;
                Ok(Box::new(bridge))
            }
            #[cfg(not(feature = "napi"))]
            {
                Err(GhostError::ConfigError(
                    "NAPI bridge not enabled (compile with 'napi' feature)".into(),
                ))
            }
        }
        BridgeType::Grpc => Err(GhostError::NotImplemented(
            "gRPC bridge not yet implemented".into(),
        )),
        BridgeType::Uds => Err(GhostError::NotImplemented(
            "UDS bridge not yet implemented".into(),
        )),
        BridgeType::Native => Err(GhostError::ConfigError(
            "Native bridge should use direct GhostWorker implementation".into(),
        )),
    }
}

/// Creates a bridge manager with default bridges based on enabled features
///
/// This convenience function creates a BridgeManager and adds bridges
/// for all enabled FFI backends.
#[allow(unused_mut)]
pub fn create_default_bridge_manager() -> Result<BridgeManager, GhostError> {
    let mut manager = BridgeManager::new();

    #[cfg(feature = "pyo3")]
    {
        manager.add_bridge(Box::new(crate::python::PythonBridge::new()?));
    }

    #[cfg(feature = "napi")]
    {
        manager.add_bridge(Box::new(crate::nodejs::NodeBridge::new()?));
    }

    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_manager_new() {
        let manager = BridgeManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_bridge_manager_default() {
        let manager = BridgeManager::default();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_bridge_manager_stats() {
        let manager = BridgeManager::new();
        let stats = manager.stats();
        assert_eq!(stats.active_workers, 0);
        assert!(!stats.is_initialized);
    }

    #[test]
    fn test_bridge_manager_health_status() {
        let manager = BridgeManager::new();
        let status = manager.health_status();
        assert!(status.is_empty());
    }

    #[test]
    fn test_create_bridge_native_error() {
        let config = BridgeConfig::new(BridgeType::Native);
        let result = create_bridge(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_bridge_grpc_error() {
        let config = BridgeConfig::new(BridgeType::Grpc);
        let result = create_bridge(config);
        assert!(result.is_err());
        if let Err(GhostError::NotImplemented(_)) = result {
            // Expected
        } else {
            panic!("Expected NotImplemented error");
        }
    }

    #[test]
    fn test_create_bridge_uds_error() {
        let config = BridgeConfig::new(BridgeType::Uds);
        let result = create_bridge(config);
        assert!(result.is_err());
        if let Err(GhostError::NotImplemented(_)) = result {
            // Expected
        } else {
            panic!("Expected NotImplemented error");
        }
    }
}
