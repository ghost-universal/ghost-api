//! Ghost Core - Routing and Health Logic
//!
//! This crate provides the core orchestration engine for ghost-api,
//! including health-based routing, fallback logic, and worker management.
//!
//! All types are imported from `ghost-schema` - the single source of truth.

mod ghost;
mod worker;
mod router;
mod health;
mod fallback;
mod events;
pub mod config;

#[cfg(test)]
mod mock;

pub use ghost::*;
pub use worker::*;
pub use router::*;
pub use health::*;
pub use fallback::*;
pub use events::*;

#[cfg(test)]
pub use mock::*;

// Re-export commonly used types from ghost-schema
pub use ghost_schema::{
    // Core types
    GhostContext, GhostError, GhostPost, GhostUser, Platform, Capability,
    Strategy, PayloadBlob, RawContext, SessionData, ProxyConfig,
    // Worker types
    WorkerHealth, HealthTier, WorkerStatus, WorkerStats,
    WorkerSelection, HealthStatus, PlatformHealthStatus,
    CircuitBreaker, HealthCheckResult,
    // Event types
    GhostEvent, SessionUnhealthyReason, SessionAction, AutoscaleEventType,
    // Config types
    GhostConfig, HealthConfig, ScraperConfig, PlatformShieldConfig,
    AutoscalingConfig, ConfigBuilder,
    // Fallback types
    FallbackContext, FailureReason, FallbackAction, FallbackStep,
    FallbackEvent, FallbackTracker,
    // Capability types
    CapabilityTier, CapabilityManifest, WorkerType, BridgeType,
};
