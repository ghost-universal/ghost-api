//! Ghost Bridge - Polyglot FFI Integration
//!
//! This crate provides FFI bridges for integrating Python, Node.js, and Go
//! scrapers with the Rust core.
//!
//! All types are imported from `ghost-schema` - the single source of truth.

mod bridge;
mod worker;
mod protocol;

#[cfg(feature = "pyo3")]
mod python;

#[cfg(feature = "napi")]
mod nodejs;

pub use bridge::*;
pub use worker::*;
pub use protocol::*;

// Re-export GhostWorker trait
pub use ghost_core::GhostWorker;

// Re-export bridge types from ghost-schema
pub use ghost_schema::{
    BridgeType, BridgeStats, BridgeConfig,
    WorkerProtocol, SerializationFormat, MessageType,
    WorkerRequest, WorkerResponse, MessageEnvelope,
    WorkerManifestMessage, HealthCheckMessage, HealthCheckResponse,
    GhostError, PayloadBlob, RawContext, CapabilityManifest, Platform,
    Capability, CapabilityTier,
};
