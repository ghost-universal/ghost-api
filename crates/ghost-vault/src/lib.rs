//! Ghost Vault - Proxy & Credential Injection
//!
//! This crate provides multi-tenant proxy and credential management,
//! including memory and file-based secret storage.
//!
//! # Architecture
//!
//! The vault system consists of several components:
//!
//! - **VaultProvider**: Trait for secret storage backends (memory, file)
//! - **VaultManager**: High-level secret access with caching
//! - **ProxyPool**: Proxy rotation and health management
//! - **CredentialStore**: Session/credential storage with indexing
//! - **SessionManager**: Session health monitoring
//! - **ContextInjector**: Automatic context building for requests
//!
//! # Example
//!
//! ```rust,ignore
//! use ghost_vault::{ProxyPool, CredentialStore, ContextInjector, VaultManager};
//!
//! // Create a proxy pool
//! let proxy_pool = ProxyPool::from_urls(&["http://proxy1:8080", "http://proxy2:8080"])?;
//!
//! // Create a credential store
//! let mut cred_store = CredentialStore::new();
//! cred_store.add_credential(my_credential);
//!
//! // Create an injector
//! let injector = ContextInjector::new()
//!     .with_proxy_pool(proxy_pool)
//!     .with_credential_store(cred_store);
//!
//! // Build context for a tenant
//! let ctx = injector.inject("tenant_123", Platform::X).await?;
//! ```

pub mod credential;
pub mod injection;
pub mod proxy;
pub mod session;
pub mod vault;

pub use credential::{CredentialStore, CredentialStoreStats};
pub use injection::{ContextInjector, ContextInjectorBuilder, InjectionMiddleware};
pub use proxy::ProxyPool;
pub use session::{SessionHealthChecker, SessionManager, SessionManagerStats};
pub use vault::{
    create_vault_manager, create_vault_provider, AsyncMemoryVault, FileVault, MemoryVault,
    VaultManager, VaultProvider,
};

// Re-export commonly used types from ghost-schema
pub use ghost_schema::{
    // Core types
    GhostContext, GhostError, Platform,
    // Proxy types
    ProxyConfig, ProxyEntry, ProxyProtocol, ProxyRotation,
    // Session types
    SessionData, SessionEntry, SessionHealthResult, SessionStatus, SessionType,
    // Credential types
    CredentialEntry, CredentialStatus,
    // Vault types
    CachedSecret, VaultConfig, VaultProviderType,
    // Injection types
    InjectionOptions, InjectionResult,
};
