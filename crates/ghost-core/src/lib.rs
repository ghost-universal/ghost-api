//! Ghost Core - Routing and Health Logic
//!
//! This crate provides the core orchestration engine for ghost-api,
//! including health-based routing, fallback logic, and worker management.

mod ghost;
mod worker;
mod router;
mod health;
mod fallback;
mod events;
mod config;

pub use ghost::*;
pub use worker::*;
pub use router::*;
pub use health::*;
pub use fallback::*;
pub use events::*;
pub use config::*;

// Re-export commonly used types from ghost-schema
pub use ghost_schema::{
    GhostContext, GhostError, GhostPost, GhostUser, Platform, Capability,
    Strategy, PayloadBlob, RawContext, SessionData, ProxyConfig,
};
