//! Error types for bridge operations

use thiserror::Error;

/// Bridge-specific errors
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
        used: u64,
        limit: u64,
    },

    /// Invalid response
    #[error("Invalid response from worker: {0}")]
    InvalidResponse(String),

    /// Bridge shutdown
    #[error("Bridge is shut down")]
    Shutdown,
}

impl BridgeError {
    /// Returns whether this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        // TODO: Implement recoverability check
        matches!(
            self,
            BridgeError::Timeout(_)
                | BridgeError::CommunicationError(_)
                | BridgeError::WorkerCrashed(_)
        )
    }

    /// Returns whether the worker should be restarted
    pub fn should_restart_worker(&self) -> bool {
        // TODO: Implement restart decision
        matches!(
            self,
            BridgeError::WorkerCrashed(_)
                | BridgeError::RuntimeError(_)
                | BridgeError::MemoryLimitExceeded { .. }
        )
    }
}

impl From<BridgeError> for ghost_schema::GhostError {
    fn from(err: BridgeError) -> Self {
        ghost_schema::GhostError::Other(err.to_string())
    }
}

#[cfg(feature = "pyo3")]
impl From<pyo3::PyErr> for BridgeError {
    fn from(err: pyo3::PyErr) -> Self {
        BridgeError::RuntimeError(err.to_string())
    }
}
