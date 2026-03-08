//! Ghost Schema - Single Source of Truth for All Types
//!
//! This crate provides ALL data structures used across the Ghost API ecosystem.
//! No other crate should define types/interfaces - they must import from here.

mod types;
mod error;
mod capability;
mod platform;
mod worker;
mod config;
mod event;
mod fallback;
mod vault;
mod bridge;
mod adapter;
mod server;
mod mock;
mod mapping;
mod adapter_types;
mod manifest;

// Re-export core types
pub use types::*;

// Re-export error types
pub use error::*;

// Re-export capability types
pub use capability::*;

// Re-export platform types
pub use platform::*;

// Re-export worker types
pub use worker::*;

// Re-export config types
pub use config::*;

// Re-export event types
pub use event::*;

// Re-export fallback types
pub use fallback::*;

// Re-export vault types
pub use vault::*;

// Re-export bridge types
pub use bridge::*;

// Re-export adapter types
pub use adapter::*;

// Re-export server types
pub use server::*;

// Re-export mock/testing types
pub use mock::*;

// Re-export mapping functions
pub use mapping::*;

// Re-export adapter-specific types
pub use adapter_types::*;

// Re-export manifest types
pub use manifest::*;

/// Prelude module with commonly used types
pub mod prelude {
    pub use crate::{
        // Core types
        GhostPost, GhostUser, GhostMedia, GhostContext, Platform, Capability,
        // Error types
        GhostError, GhostResult,
        // Capability types
        CapabilityTier, CapabilityManifest, WorkerType,
        // Platform types
        PlatformConfig, PlatformShield,
        // Config types
        Strategy, BudgetLimits, GhostConfig,
        // Session types
        SessionData, SessionType, ProxyConfig, ProxyProtocol,
        // Worker types
        WorkerHealth, WorkerStatus, HealthTier,
        // Vault types
        VaultProviderType, VaultConfig, ProxyEntry, CredentialEntry,
        // Bridge types
        BridgeType, WorkerProtocol,
        // Media types
        MediaType,
        // Mapping utilities
        normalize_username, build_profile_url, parse_iso8601,
    };
}
