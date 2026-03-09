//! Ghost Bridge - Polyglot FFI Integration
//!
//! This crate provides FFI bridges for integrating Python, Node.js, and Go
//! scrapers with the Rust core. It enables calling foreign language scrapers
//! as if they were native Rust workers.
//!
//! # Architecture
//!
//! The bridge system consists of several components:
//!
//! - **Bridge trait**: Defines the interface for FFI bridges
//! - **BridgeWorker**: Wraps FFI communication as a GhostWorker
//! - **ProtocolHandler**: Manages serialization/deserialization of messages
//! - **WorkerPool/WorkerFactory**: Manage collections of bridge workers
//!
//! # Feature Flags
//!
//! - `pyo3`: Enable Python bridge via PyO3
//! - `napi`: Enable Node.js bridge via napi-rs
//! - `grpc`: Enable gRPC bridge (planned)
//!
//! # Example
//!
//! ```rust,ignore
//! use ghost_bridge::{BridgeManager, create_bridge, BridgeType, BridgeConfig};
//!
//! // Create a bridge manager
//! let mut manager = BridgeManager::new();
//!
//! // Create and add a bridge
//! let config = BridgeConfig::new(BridgeType::PyO3);
//! let bridge = create_bridge(config)?;
//! manager.add_bridge(bridge);
//!
//! // Initialize all bridges
//! manager.initialize_all().await?;
//! ```
//!
//! All types are imported from `ghost-schema` - the single source of truth.

pub mod bridge;
pub mod error;
pub mod protocol;
pub mod worker;

// Conditionally compile Python and Node.js modules
// These modules handle their own feature gating internally
#[cfg(feature = "pyo3")]
pub mod python;

#[cfg(feature = "napi")]
pub mod nodejs;

// Re-export main types
pub use bridge::{Bridge, BridgeManager, create_bridge, create_default_bridge_manager};
pub use error::BridgeError;
pub use protocol::{ProtocolBuilder, ProtocolHandler};
pub use worker::{BridgeWorker, WorkerFactory, WorkerPool};

// Re-export GhostWorker trait
pub use ghost_core::GhostWorker;

// Re-export bridge types from ghost-schema
pub use ghost_schema::{
    BridgeConfig, BridgeStats, BridgeType, Capability, CapabilityManifest, CapabilityTier,
    GhostError, HealthCheckMessage, HealthCheckResponse, MessageEnvelope, MessageType, Platform,
    PayloadBlob, RawContext, SerializationFormat, WorkerManifestMessage, WorkerProtocol,
    WorkerRequest, WorkerResponse,
};

/// Prelude module for common imports
pub mod prelude {
    pub use crate::bridge::{Bridge, BridgeManager};
    pub use crate::error::BridgeError;
    pub use crate::protocol::ProtocolHandler;
    pub use crate::worker::{BridgeWorker, WorkerFactory, WorkerPool};
    pub use ghost_core::GhostWorker;
    pub use ghost_schema::{
        BridgeConfig, BridgeStats, BridgeType, Capability, GhostError, PayloadBlob, Platform,
        RawContext,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_manager_creation() {
        let manager = BridgeManager::new();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_protocol_handler_defaults() {
        let handler = ProtocolHandler::default();
        assert_eq!(handler.protocol(), ghost_schema::WorkerProtocol::JsonStdio);
    }

    #[test]
    fn test_worker_pool_defaults() {
        let pool = WorkerPool::default();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_bridge_error_is_recoverable() {
        let err = BridgeError::Timeout("test".into());
        assert!(err.is_recoverable());
    }
}
