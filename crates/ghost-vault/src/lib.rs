//! Ghost Vault - Proxy & Credential Injection
//!
//! This crate provides multi-tenant proxy and credential management,
//! including integration with secret managers.

mod proxy;
mod credential;
mod session;
mod vault;
mod injection;

pub use proxy::*;
pub use credential::*;
pub use session::*;
pub use vault::*;
pub use injection::*;

// Re-export commonly used types
pub use ghost_schema::{ProxyConfig, SessionData, GhostContext};
