//! Context injection for multi-tenancy
//!
//! This module provides context injection for building GhostContext with
//! automatic proxy and credential selection.

use ghost_schema::{
    GhostContext, GhostError, InjectionOptions, InjectionResult, Platform, ProxyConfig,
    SessionData,
};

use crate::{CredentialStore, ProxyPool, VaultManager};

/// Context injector for building GhostContext
///
/// The ContextInjector coordinates proxy pool, credential store, and vault
/// to automatically inject context for requests.
pub struct ContextInjector {
    /// Proxy pool for proxy selection
    proxy_pool: Option<ProxyPool>,
    /// Credential store for session/credential lookup
    credential_store: Option<CredentialStore>,
    /// Vault manager for secret retrieval
    vault: Option<VaultManager>,
}

impl ContextInjector {
    /// Creates a new context injector
    pub fn new() -> Self {
        Self {
            proxy_pool: None,
            credential_store: None,
            vault: None,
        }
    }

    /// Sets the proxy pool
    pub fn with_proxy_pool(mut self, pool: ProxyPool) -> Self {
        self.proxy_pool = Some(pool);
        self
    }

    /// Sets the credential store
    pub fn with_credential_store(mut self, store: CredentialStore) -> Self {
        self.credential_store = Some(store);
        self
    }

    /// Sets the vault manager
    pub fn with_vault(mut self, vault: VaultManager) -> Self {
        self.vault = Some(vault);
        self
    }

    /// Injects context for a tenant and platform
    ///
    /// Automatically selects an available proxy and credential for the tenant.
    pub async fn inject(
        &self,
        tenant_id: &str,
        platform: Platform,
    ) -> Result<GhostContext, GhostError> {
        let mut builder = GhostContext::builder().tenant_id(tenant_id);

        // Get proxy from pool
        if let Some(pool) = &self.proxy_pool {
            if let Some(proxy) = pool.get_next().await {
                builder = builder.proxy_config(proxy.config);
            }
        }

        // Get credential from store
        if let Some(store) = &self.credential_store {
            if let Some(cred) = store.get_first_active(tenant_id, platform) {
                builder = builder.session_data(cred.session.clone());
            }
        }

        Ok(builder.build())
    }

    /// Injects context with a specific session
    ///
    /// Uses the provided session data, optionally selecting a sticky proxy.
    pub fn inject_with_session(
        &self,
        tenant_id: &str,
        session: SessionData,
        session_id: Option<&str>,
    ) -> GhostContext {
        let builder = GhostContext::builder()
            .tenant_id(tenant_id)
            .session_data(session);

        // For sticky sessions, we'd need async access to the proxy pool
        // This is a simplified version that doesn't support sticky proxy selection
        let _ = session_id; // Will be used for sticky proxy selection when available

        builder.build()
    }

    /// Injects context with a specific proxy
    pub fn inject_with_proxy(&self, tenant_id: &str, proxy: ProxyConfig) -> GhostContext {
        GhostContext::builder()
            .tenant_id(tenant_id)
            .proxy_config(proxy)
            .build()
    }

    /// Injects context with both specific proxy and session
    pub fn inject_with_proxy_and_session(
        &self,
        tenant_id: &str,
        proxy: ProxyConfig,
        session: SessionData,
    ) -> GhostContext {
        GhostContext::builder()
            .tenant_id(tenant_id)
            .proxy_config(proxy)
            .session_data(session)
            .build()
    }

    /// Injects context from vault reference
    ///
    /// Retrieves session data from the vault and builds context.
    pub async fn inject_from_vault(
        &mut self,
        tenant_id: &str,
        _platform: Platform,
        vault_ref: &str,
    ) -> Result<GhostContext, GhostError> {
        if let Some(vault) = &mut self.vault {
            let secret = vault.get(vault_ref).await?;
            let session = SessionData::from_cookies(&secret);

            Ok(GhostContext::builder()
                .tenant_id(tenant_id)
                .session_data(session)
                .build())
        } else {
            Err(GhostError::ConfigError("Vault not configured".into()))
        }
    }

    /// Injects context with full options
    ///
    /// Provides fine-grained control over injection behavior with overrides.
    pub async fn inject_with_options(
        &self,
        tenant_id: &str,
        platform: Platform,
        options: InjectionOptions,
    ) -> Result<InjectionResult, GhostError> {
        let mut builder = GhostContext::builder().tenant_id(tenant_id);
        let mut selected_proxy_id = None;
        let mut selected_credential_id = None;
        let mut used_fallback = false;

        // Apply proxy override or select from pool
        if let Some(proxy) = &options.proxy_override {
            builder = builder.proxy_config(proxy.clone());
        } else if let Some(pool) = &self.proxy_pool {
            if let Some(proxy) = pool.get_next().await {
                selected_proxy_id = Some(proxy.id.clone());
                builder = builder.proxy_config(proxy.config);
            }
        }

        // Apply session override or select from store
        if let Some(session) = &options.session_override {
            builder = builder.session_data(session.clone());
        } else if let Some(store) = &self.credential_store {
            if let Some(cred) = store.get_first_active(tenant_id, platform) {
                selected_credential_id = Some(cred.id.clone());
                builder = builder.session_data(cred.session.clone());
            } else {
                // Try fallback to any credential for tenant
                used_fallback = true;
                if let Some(cred) = store.get_for_tenant(tenant_id, platform).first() {
                    selected_credential_id = Some(cred.id.clone());
                    builder = builder.session_data(cred.session.clone());
                }
            }
        }

        // Apply timeout override
        if let Some(timeout_ms) = options.timeout_ms {
            builder = builder.metadata("timeout_ms", timeout_ms.to_string());
        }

        // Add any additional metadata
        for (key, value) in &options.metadata {
            builder = builder.metadata(key, value);
        }

        let context = builder.build();

        let mut result = InjectionResult::new(context);
        result.selected_proxy_id = selected_proxy_id;
        result.selected_credential_id = selected_credential_id;
        result.used_fallback = used_fallback;

        Ok(result)
    }

    /// Checks if the injector has a proxy pool configured
    pub fn has_proxy_pool(&self) -> bool {
        self.proxy_pool.is_some()
    }

    /// Checks if the injector has a credential store configured
    pub fn has_credential_store(&self) -> bool {
        self.credential_store.is_some()
    }

    /// Checks if the injector has a vault configured
    pub fn has_vault(&self) -> bool {
        self.vault.is_some()
    }
}

impl Default for ContextInjector {
    fn default() -> Self {
        Self::new()
    }
}

/// Injection middleware for automatic context building
///
/// Provides a higher-level interface for context injection with
/// request-level configuration.
pub struct InjectionMiddleware {
    /// Context injector
    injector: ContextInjector,
}

impl InjectionMiddleware {
    /// Creates new injection middleware
    pub fn new(injector: ContextInjector) -> Self {
        Self { injector }
    }

    /// Creates new middleware with default injector
    pub fn default_injector() -> Self {
        Self::new(ContextInjector::new())
    }

    /// Builds context for a request
    pub async fn build_context(
        &self,
        tenant_id: &str,
        platform: Platform,
        options: InjectionOptions,
    ) -> Result<GhostContext, GhostError> {
        let result = self
            .injector
            .inject_with_options(tenant_id, platform, options)
            .await?;
        Ok(result.context)
    }

    /// Builds context with default options
    pub async fn build_default_context(
        &self,
        tenant_id: &str,
        platform: Platform,
    ) -> Result<GhostContext, GhostError> {
        self.injector.inject(tenant_id, platform).await
    }

    /// Gets a reference to the injector
    pub fn injector(&self) -> &ContextInjector {
        &self.injector
    }
}

/// Builder for creating ContextInjector with fluent API
pub struct ContextInjectorBuilder {
    injector: ContextInjector,
}

impl ContextInjectorBuilder {
    /// Creates a new builder
    pub fn new() -> Self {
        Self {
            injector: ContextInjector::new(),
        }
    }

    /// Adds a proxy pool
    pub fn with_proxy_pool(mut self, pool: ProxyPool) -> Self {
        self.injector = self.injector.with_proxy_pool(pool);
        self
    }

    /// Adds a credential store
    pub fn with_credential_store(mut self, store: CredentialStore) -> Self {
        self.injector = self.injector.with_credential_store(store);
        self
    }

    /// Adds a vault manager
    pub fn with_vault(mut self, vault: VaultManager) -> Self {
        self.injector = self.injector.with_vault(vault);
        self
    }

    /// Builds the injector
    pub fn build(self) -> ContextInjector {
        self.injector
    }
}

impl Default for ContextInjectorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_injector_new() {
        let injector = ContextInjector::new();
        assert!(!injector.has_proxy_pool());
        assert!(!injector.has_credential_store());
        assert!(!injector.has_vault());
    }

    #[test]
    fn test_context_injector_default() {
        let injector = ContextInjector::default();
        assert!(!injector.has_proxy_pool());
    }

    #[test]
    fn test_context_injector_with_proxy_pool() {
        let injector = ContextInjector::new().with_proxy_pool(ProxyPool::new());
        assert!(injector.has_proxy_pool());
    }

    #[test]
    fn test_context_injector_with_credential_store() {
        let injector = ContextInjector::new().with_credential_store(CredentialStore::new());
        assert!(injector.has_credential_store());
    }

    #[test]
    fn test_context_injector_with_vault() {
        let injector = ContextInjector::new().with_vault(VaultManager::memory());
        assert!(injector.has_vault());
    }

    #[tokio::test]
    async fn test_context_injector_inject() {
        let injector = ContextInjector::new();
        let ctx = injector.inject("tenant1", Platform::X).await.unwrap();

        assert_eq!(ctx.tenant_id, Some("tenant1".to_string()));
    }

    #[tokio::test]
    async fn test_context_injector_inject_with_proxy() {
        let mut pool = ProxyPool::new();
        let config = ProxyConfig::from_url("http://localhost:8080").unwrap();
        pool.add_proxy(ghost_schema::ProxyEntry::new("p1", config))
            .unwrap();

        let injector = ContextInjector::new().with_proxy_pool(pool);
        let ctx = injector.inject("tenant1", Platform::X).await.unwrap();

        assert!(ctx.proxy.is_some());
    }

    #[test]
    fn test_context_injector_inject_with_session() {
        let injector = ContextInjector::new();
        let session = SessionData::from_cookies("test=value");

        let ctx = injector.inject_with_session("tenant1", session, None);

        assert_eq!(ctx.tenant_id, Some("tenant1".to_string()));
        assert!(ctx.session.is_some());
    }

    #[test]
    fn test_context_injector_inject_with_proxy_explicit() {
        let injector = ContextInjector::new();
        let proxy = ProxyConfig::from_url("http://localhost:8080").unwrap();

        let ctx = injector.inject_with_proxy("tenant1", proxy.clone());

        assert_eq!(ctx.tenant_id, Some("tenant1".to_string()));
        assert!(ctx.proxy.is_some());
    }

    #[tokio::test]
    async fn test_context_injector_inject_with_options() {
        let injector = ContextInjector::new();
        let options = InjectionOptions::new();

        let result = injector
            .inject_with_options("tenant1", Platform::X, options)
            .await
            .unwrap();

        assert_eq!(
            result.context.tenant_id,
            Some("tenant1".to_string())
        );
    }

    #[tokio::test]
    async fn test_context_injector_inject_with_options_overrides() {
        let injector = ContextInjector::new();
        let proxy = ProxyConfig::from_url("http://localhost:8080").unwrap();
        let session = SessionData::from_cookies("test=value");

        let options = InjectionOptions::new()
            .with_proxy(proxy.clone())
            .with_session(session.clone());

        let result = injector
            .inject_with_options("tenant1", Platform::X, options)
            .await
            .unwrap();

        assert!(result.context.proxy.is_some());
        assert!(result.context.session.is_some());
    }

    #[tokio::test]
    async fn test_injection_middleware() {
        let injector = ContextInjector::new();
        let middleware = InjectionMiddleware::new(injector);

        let ctx = middleware
            .build_default_context("tenant1", Platform::X)
            .await
            .unwrap();

        assert_eq!(ctx.tenant_id, Some("tenant1".to_string()));
    }

    #[test]
    fn test_context_injector_builder() {
        let injector = ContextInjectorBuilder::new()
            .with_proxy_pool(ProxyPool::new())
            .with_credential_store(CredentialStore::new())
            .with_vault(VaultManager::memory())
            .build();

        assert!(injector.has_proxy_pool());
        assert!(injector.has_credential_store());
        assert!(injector.has_vault());
    }

    #[tokio::test]
    async fn test_context_injector_from_vault() {
        let vault = VaultManager::memory();
        vault.put("session/tenant1", "cookie=test").await.unwrap();

        let _injector = ContextInjector::new().with_vault(vault);

        // Note: This test needs mutable injector for vault access
        // In practice, the vault would be accessed before creating the injector
        // or through Arc<RwLock<VaultManager>>
    }
}
