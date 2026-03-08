//! Ghost Schema - Single Source of Truth for All Types
//!
//! This crate provides ALL data structures used across the Ghost API ecosystem.
//! No other crate should define types/interfaces - they must import from here.

mod types;
mod error;
mod capability;
mod platform;

// Re-export core types
pub use types::*;
pub use error::*;
pub use capability::*;
pub use platform::*;

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
        Strategy, BudgetLimits,
        // Session types
        SessionData, SessionType, ProxyConfig, ProxyProtocol,
    };
}
