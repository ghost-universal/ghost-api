//! Error types for bridge operations
//!
//! This module provides bridge-specific error types for handling
//! failures in FFI bridge communication.

use thiserror::Error;

/// Bridge-specific errors
///
/// These errors represent failures specific to FFI bridge operations,
/// such as initialization failures, worker crashes, and communication errors.
#[derive(Debug, Error)]
pub enum BridgeError {
    /// Failed to initialize bridge
    #[error("Bridge initialization failed: {0}")]
    InitializationFailed(String),

    /// Bridge not initialized
    #[error("Bridge not initialized")]
    NotInitialized,

    /// Worker not found
    #[error("Worker not found: {0}")]
    WorkerNotFound(String),

    /// Worker crashed
    #[error("Worker crashed: {0}")]
    WorkerCrashed(String),

    /// Communication error
    #[error("Communication error: {0}")]
    CommunicationError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Runtime error
    #[error("Runtime error: {0}")]
    RuntimeError(String),

    /// Memory limit exceeded
    #[error("Memory limit exceeded: used {used}MB, limit {limit}MB")]
    MemoryLimitExceeded {
        /// Memory used in MB
        used: u64,
        /// Memory limit in MB
        limit: u64,
    },

    /// Invalid response
    #[error("Invalid response from worker: {0}")]
    InvalidResponse(String),

    /// Bridge shutdown
    #[error("Bridge is shut down")]
    Shutdown,

    /// Feature not enabled
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl BridgeError {
    /// Returns whether this error is recoverable
    ///
    /// Recoverable errors are those where retrying the operation
    /// might succeed (e.g., timeouts, temporary communication failures).
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            BridgeError::Timeout(_)
                | BridgeError::CommunicationError(_)
                | BridgeError::WorkerCrashed(_)
        )
    }

    /// Returns whether the worker should be restarted
    ///
    /// Some errors indicate that the worker process is in a bad state
    /// and should be restarted before continuing.
    pub fn should_restart_worker(&self) -> bool {
        matches!(
            self,
            BridgeError::WorkerCrashed(_)
                | BridgeError::RuntimeError(_)
                | BridgeError::MemoryLimitExceeded { .. }
        )
    }

    /// Returns whether this error indicates a resource limit was hit
    pub fn is_resource_limit(&self) -> bool {
        matches!(
            self,
            BridgeError::MemoryLimitExceeded { .. } | BridgeError::Timeout(_)
        )
    }

    /// Returns whether this error indicates a communication failure
    pub fn is_communication_failure(&self) -> bool {
        matches!(
            self,
            BridgeError::CommunicationError(_)
                | BridgeError::InvalidResponse(_)
                | BridgeError::WorkerCrashed(_)
        )
    }

    /// Creates an initialization failed error
    pub fn init_failed(message: impl Into<String>) -> Self {
        BridgeError::InitializationFailed(message.into())
    }

    /// Creates a worker not found error
    pub fn worker_not_found(worker_id: impl Into<String>) -> Self {
        BridgeError::WorkerNotFound(worker_id.into())
    }

    /// Creates a timeout error
    pub fn timeout(operation: impl Into<String>) -> Self {
        BridgeError::Timeout(operation.into())
    }

    /// Creates a communication error
    pub fn communication(message: impl Into<String>) -> Self {
        BridgeError::CommunicationError(message.into())
    }

    /// Creates a feature not enabled error
    pub fn feature_not_enabled(feature: impl Into<String>) -> Self {
        BridgeError::FeatureNotEnabled(feature.into())
    }
}

impl From<BridgeError> for ghost_schema::GhostError {
    fn from(err: BridgeError) -> Self {
        match err {
            BridgeError::Timeout(msg) => ghost_schema::GhostError::Timeout(msg),
            BridgeError::ConfigError(msg) => ghost_schema::GhostError::ConfigError(msg),
            BridgeError::WorkerNotFound(id) => ghost_schema::GhostError::ScraperError {
                worker: id,
                message: "Worker not found".into(),
            },
            _ => ghost_schema::GhostError::Other(err.to_string()),
        }
    }
}

#[cfg(feature = "pyo3")]
impl From<pyo3::PyErr> for BridgeError {
    fn from(err: pyo3::PyErr) -> Self {
        BridgeError::RuntimeError(err.to_string())
    }
}

impl From<std::io::Error> for BridgeError {
    fn from(err: std::io::Error) -> Self {
        BridgeError::CommunicationError(err.to_string())
    }
}

impl From<serde_json::Error> for BridgeError {
    fn from(err: serde_json::Error) -> Self {
        BridgeError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_error_is_recoverable() {
        assert!(BridgeError::Timeout("test".into()).is_recoverable());
        assert!(BridgeError::CommunicationError("test".into()).is_recoverable());
        assert!(BridgeError::WorkerCrashed("test".into()).is_recoverable());
        assert!(!BridgeError::NotInitialized.is_recoverable());
    }

    #[test]
    fn test_bridge_error_should_restart() {
        assert!(BridgeError::WorkerCrashed("test".into()).should_restart_worker());
        assert!(BridgeError::RuntimeError("test".into()).should_restart_worker());
        assert!(BridgeError::MemoryLimitExceeded { used: 100, limit: 50 }.should_restart_worker());
        assert!(!BridgeError::Timeout("test".into()).should_restart_worker());
    }

    #[test]
    fn test_bridge_error_is_resource_limit() {
        assert!(BridgeError::MemoryLimitExceeded { used: 100, limit: 50 }.is_resource_limit());
        assert!(BridgeError::Timeout("test".into()).is_resource_limit());
        assert!(!BridgeError::WorkerCrashed("test".into()).is_resource_limit());
    }

    #[test]
    fn test_bridge_error_is_communication_failure() {
        assert!(BridgeError::CommunicationError("test".into()).is_communication_failure());
        assert!(BridgeError::InvalidResponse("test".into()).is_communication_failure());
        assert!(BridgeError::WorkerCrashed("test".into()).is_communication_failure());
        assert!(!BridgeError::Timeout("test".into()).is_communication_failure());
    }

    #[test]
    fn test_bridge_error_constructors() {
        let err = BridgeError::init_failed("test");
        assert!(matches!(err, BridgeError::InitializationFailed(_)));

        let err = BridgeError::worker_not_found("worker1");
        assert!(matches!(err, BridgeError::WorkerNotFound(_)));

        let err = BridgeError::timeout("operation");
        assert!(matches!(err, BridgeError::Timeout(_)));

        let err = BridgeError::communication("failed");
        assert!(matches!(err, BridgeError::CommunicationError(_)));

        let err = BridgeError::feature_not_enabled("pyo3");
        assert!(matches!(err, BridgeError::FeatureNotEnabled(_)));
    }

    #[test]
    fn test_bridge_error_to_ghost_error() {
        let err: ghost_schema::GhostError = BridgeError::Timeout("test".into()).into();
        assert!(matches!(err, ghost_schema::GhostError::Timeout(_)));

        let err: ghost_schema::GhostError = BridgeError::ConfigError("bad config".into()).into();
        assert!(matches!(err, ghost_schema::GhostError::ConfigError(_)));
    }
}
