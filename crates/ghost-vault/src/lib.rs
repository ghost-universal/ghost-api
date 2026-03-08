//! Ghost Vault - Proxy & Credential Injection
//!
//! This crate provides multi-tenant proxy and credential management,
//! including integration with secret managers.
//!
//! All types are imported from `ghost-schema` - the single source of truth.

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

// Re-export commonly used types from ghost-schema
pub use ghost_schema::{
    ProxyConfig, ProxyProtocol, SessionData, SessionType, GhostContext, GhostError, Platform,
    // Vault types
    VaultProviderType, VaultConfig, CachedSecret, ProxyEntry, ProxyRotation,
    CredentialEntry, SessionEntry, SessionStatus, SessionHealthResult,
    InjectionOptions, InjectionResult,
};
