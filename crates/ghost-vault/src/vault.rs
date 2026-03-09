//! Secret vault integration
//!
//! This module provides vault providers for secret management.
//! Only memory and file-based storage are supported.

use async_trait::async_trait;
use ghost_schema::{CachedSecret, GhostError, VaultConfig, VaultProviderType};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Secret vault provider trait
///
/// Defines the interface for secret storage backends.
#[async_trait]
pub trait VaultProvider: Send + Sync {
    /// Gets a secret by path
    async fn get_secret(&self, path: &str) -> Result<String, GhostError>;

    /// Puts a secret at a path
    async fn put_secret(&self, path: &str, value: &str) -> Result<(), GhostError>;

    /// Deletes a secret
    async fn delete_secret(&self, path: &str) -> Result<(), GhostError>;

    /// Lists secrets at a path prefix
    async fn list_secrets(&self, prefix: &str) -> Result<Vec<String>, GhostError>;

    /// Returns the provider name
    fn provider_name(&self) -> &'static str;

    /// Returns the provider type
    fn provider_type(&self) -> VaultProviderType;
}

/// In-memory vault provider (for testing and ephemeral storage)
///
/// Stores secrets in memory with no persistence. Ideal for testing
/// or temporary secret storage.
pub struct MemoryVault {
    secrets: HashMap<String, String>,
}

impl MemoryVault {
    /// Creates a new memory vault
    pub fn new() -> Self {
        Self {
            secrets: HashMap::new(),
        }
    }

    /// Creates a new memory vault with capacity hint
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            secrets: HashMap::with_capacity(capacity),
        }
    }

    /// Pre-populates the vault with secrets
    pub fn with_secrets(mut self, secrets: HashMap<String, String>) -> Self {
        self.secrets = secrets;
        self
    }

    /// Adds a secret
    pub fn add_secret(&mut self, path: impl Into<String>, value: impl Into<String>) {
        self.secrets.insert(path.into(), value.into());
    }

    /// Returns the number of stored secrets
    pub fn len(&self) -> usize {
        self.secrets.len()
    }

    /// Returns whether the vault is empty
    pub fn is_empty(&self) -> bool {
        self.secrets.is_empty()
    }
}

#[async_trait]
impl VaultProvider for MemoryVault {
    async fn get_secret(&self, path: &str) -> Result<String, GhostError> {
        self.secrets
            .get(path)
            .cloned()
            .ok_or_else(|| GhostError::ConfigError(format!("Secret not found: {}", path)))
    }

    async fn put_secret(&self, path: &str, value: &str) -> Result<(), GhostError> {
        // MemoryVault needs interior mutability for put operations
        // For now, return an error suggesting use of the sync method
        let _ = (path, value);
        Err(GhostError::NotImplemented(
            "Use MemoryVault::add_secret for synchronous writes".into(),
        ))
    }

    async fn delete_secret(&self, path: &str) -> Result<(), GhostError> {
        let _ = path;
        Err(GhostError::NotImplemented(
            "Use mutable MemoryVault for deletes".into(),
        ))
    }

    async fn list_secrets(&self, prefix: &str) -> Result<Vec<String>, GhostError> {
        Ok(self
            .secrets
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }

    fn provider_name(&self) -> &'static str {
        "memory"
    }

    fn provider_type(&self) -> VaultProviderType {
        VaultProviderType::Memory
    }
}

impl Default for MemoryVault {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe in-memory vault with async write support
pub struct AsyncMemoryVault {
    secrets: Arc<RwLock<HashMap<String, String>>>,
}

impl AsyncMemoryVault {
    /// Creates a new async memory vault
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates with pre-populated secrets
    pub fn with_secrets(secrets: HashMap<String, String>) -> Self {
        Self {
            secrets: Arc::new(RwLock::new(secrets)),
        }
    }
}

impl Default for AsyncMemoryVault {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VaultProvider for AsyncMemoryVault {
    async fn get_secret(&self, path: &str) -> Result<String, GhostError> {
        let secrets = self.secrets.read().await;
        secrets
            .get(path)
            .cloned()
            .ok_or_else(|| GhostError::ConfigError(format!("Secret not found: {}", path)))
    }

    async fn put_secret(&self, path: &str, value: &str) -> Result<(), GhostError> {
        let mut secrets = self.secrets.write().await;
        secrets.insert(path.to_string(), value.to_string());
        Ok(())
    }

    async fn delete_secret(&self, path: &str) -> Result<(), GhostError> {
        let mut secrets = self.secrets.write().await;
        secrets
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| GhostError::ConfigError(format!("Secret not found: {}", path)))
    }

    async fn list_secrets(&self, prefix: &str) -> Result<Vec<String>, GhostError> {
        let secrets = self.secrets.read().await;
        Ok(secrets
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }

    fn provider_name(&self) -> &'static str {
        "async_memory"
    }

    fn provider_type(&self) -> VaultProviderType {
        VaultProviderType::Memory
    }
}

/// File-based vault provider (persistent)
///
/// Stores secrets in a JSON file with optional encryption support.
pub struct FileVault {
    /// Path to the vault file
    file_path: String,
    /// In-memory cache of secrets
    secrets: HashMap<String, String>,
    /// Whether there are unsaved changes
    dirty: bool,
}

impl FileVault {
    /// Creates a new file vault
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            secrets: HashMap::new(),
            dirty: false,
        }
    }

    /// Creates a new file vault and loads existing data
    pub fn open(file_path: impl Into<String>) -> Result<Self, GhostError> {
        let mut vault = Self::new(file_path);
        vault.load()?;
        Ok(vault)
    }

    /// Loads secrets from the file
    pub fn load(&mut self) -> Result<(), GhostError> {
        let path = Path::new(&self.file_path);

        if !path.exists() {
            // File doesn't exist yet, start with empty vault
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        self.secrets = serde_json::from_str(&content).unwrap_or_default();
        self.dirty = false;

        Ok(())
    }

    /// Saves secrets to the file
    pub fn save(&self) -> Result<(), GhostError> {
        let json = serde_json::to_string_pretty(&self.secrets)?;
        std::fs::write(&self.file_path, json)?;
        Ok(())
    }

    /// Saves if there are unsaved changes
    pub fn save_if_dirty(&mut self) -> Result<(), GhostError> {
        if self.dirty {
            self.save()?;
            self.dirty = false;
        }
        Ok(())
    }

    /// Creates the vault file if it doesn't exist
    pub fn ensure_exists(&self) -> Result<(), GhostError> {
        let path = Path::new(&self.file_path);

        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&self.file_path, "{}")?;
        }

        Ok(())
    }

    /// Adds a secret (marks as dirty)
    pub fn add_secret(&mut self, path: impl Into<String>, value: impl Into<String>) {
        self.secrets.insert(path.into(), value.into());
        self.dirty = true;
    }

    /// Removes a secret (marks as dirty)
    pub fn remove_secret(&mut self, path: &str) -> Option<String> {
        let value = self.secrets.remove(path);
        if value.is_some() {
            self.dirty = true;
        }
        value
    }

    /// Returns whether there are unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Returns the file path
    pub fn file_path(&self) -> &str {
        &self.file_path
    }
}

#[async_trait]
impl VaultProvider for FileVault {
    async fn get_secret(&self, path: &str) -> Result<String, GhostError> {
        self.secrets
            .get(path)
            .cloned()
            .ok_or_else(|| GhostError::ConfigError(format!("Secret not found: {}", path)))
    }

    async fn put_secret(&self, path: &str, value: &str) -> Result<(), GhostError> {
        let _ = (path, value);
        // Note: This requires mutable access. Use the sync methods or AsyncFileVault.
        Err(GhostError::NotImplemented(
            "Use mutable FileVault methods for writes".into(),
        ))
    }

    async fn delete_secret(&self, path: &str) -> Result<(), GhostError> {
        let _ = path;
        Err(GhostError::NotImplemented(
            "Use mutable FileVault methods for deletes".into(),
        ))
    }

    async fn list_secrets(&self, prefix: &str) -> Result<Vec<String>, GhostError> {
        Ok(self
            .secrets
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }

    fn provider_name(&self) -> &'static str {
        "file"
    }

    fn provider_type(&self) -> VaultProviderType {
        VaultProviderType::File
    }
}

/// Vault manager for coordinating secret access
///
/// The VaultManager provides a unified interface for secret access with
/// caching, TTL support, and provider abstraction.
pub struct VaultManager {
    /// Active provider
    provider: Box<dyn VaultProvider>,
    /// Secret cache
    cache: HashMap<String, CachedSecret>,
    /// Configuration
    config: VaultConfig,
}

impl VaultManager {
    /// Creates a new vault manager
    pub fn new(provider: Box<dyn VaultProvider>, config: VaultConfig) -> Self {
        Self {
            provider,
            cache: HashMap::new(),
            config,
        }
    }

    /// Creates a memory vault manager
    pub fn memory() -> Self {
        Self::new(
            Box::new(AsyncMemoryVault::new()),
            VaultConfig::memory(),
        )
    }

    /// Creates a file vault manager
    pub fn file(path: impl Into<String>) -> Self {
        let path_str = path.into();
        Self::new(
            Box::new(FileVault::new(&path_str)),
            VaultConfig::file(&path_str),
        )
    }

    /// Creates an async memory vault manager
    pub fn async_memory() -> Self {
        Self::new(
            Box::new(AsyncMemoryVault::new()),
            VaultConfig::memory(),
        )
    }

    /// Gets a secret (with caching)
    ///
    /// Returns cached value if not expired, otherwise fetches from provider.
    pub async fn get(&mut self, path: &str) -> Result<String, GhostError> {
        // Check cache first
        if let Some(cached) = self.cache.get(path) {
            if !cached.is_expired() {
                return Ok(cached.value.clone());
            }
        }

        // Fetch from provider
        let value = self.provider.get_secret(path).await?;

        // Cache the result
        self.cache.insert(
            path.to_string(),
            CachedSecret::new(&value, self.config.cache_ttl_secs),
        );

        Ok(value)
    }

    /// Gets a secret without caching
    pub async fn get_uncached(&self, path: &str) -> Result<String, GhostError> {
        self.provider.get_secret(path).await
    }

    /// Puts a secret
    pub async fn put(&self, path: &str, value: &str) -> Result<(), GhostError> {
        self.provider.put_secret(path, value).await
    }

    /// Deletes a secret
    pub async fn delete(&self, path: &str) -> Result<(), GhostError> {
        self.provider.delete_secret(path).await
    }

    /// Lists secrets with a prefix
    pub async fn list(&self, prefix: &str) -> Result<Vec<String>, GhostError> {
        self.provider.list_secrets(prefix).await
    }

    /// Lists all secrets
    pub async fn list_all(&self) -> Result<Vec<String>, GhostError> {
        self.list("").await
    }

    /// Invalidates the cache for a path
    pub fn invalidate(&mut self, path: &str) {
        self.cache.remove(path);
    }

    /// Clears all cached secrets
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Returns the cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Returns the provider name
    pub fn provider_name(&self) -> &'static str {
        self.provider.provider_name()
    }

    /// Returns the provider type
    pub fn provider_type(&self) -> VaultProviderType {
        self.provider.provider_type()
    }

    /// Returns the configuration
    pub fn config(&self) -> &VaultConfig {
        &self.config
    }

    /// Preloads secrets into cache
    pub async fn preload(&mut self, prefix: &str) -> Result<usize, GhostError> {
        let paths = self.list(prefix).await?;
        let mut loaded = 0;

        for path in &paths {
            if self.get(path).await.is_ok() {
                loaded += 1;
            }
        }

        Ok(loaded)
    }
}

/// Creates a vault provider from configuration
pub fn create_vault_provider(config: &VaultConfig) -> Result<Box<dyn VaultProvider>, GhostError> {
    match config.provider {
        VaultProviderType::Memory => Ok(Box::new(AsyncMemoryVault::new())),
        VaultProviderType::File => {
            let path = config
                .file_path
                .as_ref()
                .ok_or_else(|| GhostError::ConfigError("file_path required for File provider".into()))?;
            Ok(Box::new(FileVault::new(path)))
        }
    }
}

/// Creates a vault manager from configuration
pub fn create_vault_manager(config: &VaultConfig) -> Result<VaultManager, GhostError> {
    let provider = create_vault_provider(config)?;
    Ok(VaultManager::new(provider, config.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_vault_new() {
        let vault = MemoryVault::new();
        assert!(vault.is_empty());
    }

    #[test]
    fn test_memory_vault_add_secret() {
        let mut vault = MemoryVault::new();
        vault.add_secret("test/key", "value");

        assert_eq!(vault.len(), 1);
    }

    #[test]
    fn test_memory_vault_with_secrets() {
        let mut secrets = HashMap::new();
        secrets.insert("key1".to_string(), "value1".to_string());
        secrets.insert("key2".to_string(), "value2".to_string());

        let vault = MemoryVault::new().with_secrets(secrets);
        assert_eq!(vault.len(), 2);
    }

    #[tokio::test]
    async fn test_memory_vault_get_secret() {
        let mut vault = MemoryVault::new();
        vault.add_secret("test/key", "value");

        let result = vault.get_secret("test/key").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "value");
    }

    #[tokio::test]
    async fn test_memory_vault_get_not_found() {
        let vault = MemoryVault::new();
        let result = vault.get_secret("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_memory_vault_list_secrets() {
        let mut vault = MemoryVault::new();
        vault.add_secret("prefix/key1", "value1");
        vault.add_secret("prefix/key2", "value2");
        vault.add_secret("other/key3", "value3");

        let result = vault.list_secrets("prefix").await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_async_memory_vault() {
        let vault = AsyncMemoryVault::new();

        // Put
        assert!(vault.put_secret("key", "value").await.is_ok());

        // Get
        let result = vault.get_secret("key").await.unwrap();
        assert_eq!(result, "value");

        // List
        let list = vault.list_secrets("").await.unwrap();
        assert_eq!(list.len(), 1);

        // Delete
        assert!(vault.delete_secret("key").await.is_ok());
        assert!(vault.get_secret("key").await.is_err());
    }

    #[tokio::test]
    async fn test_vault_manager_memory() {
        let mut manager = VaultManager::memory();

        // Put a secret
        assert!(manager.put("test/key", "value").await.is_ok());

        // Get with caching
        let result = manager.get("test/key").await.unwrap();
        assert_eq!(result, "value");

        // Should be cached
        assert_eq!(manager.cache_size(), 1);

        // Invalidate
        manager.invalidate("test/key");
        assert_eq!(manager.cache_size(), 0);
    }

    #[tokio::test]
    async fn test_vault_manager_list() {
        let manager = VaultManager::async_memory();
        manager.put("prefix/a", "1").await.unwrap();
        manager.put("prefix/b", "2").await.unwrap();
        manager.put("other/c", "3").await.unwrap();

        let list = manager.list("prefix").await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_create_vault_provider_memory() {
        let config = VaultConfig::memory();
        let provider = create_vault_provider(&config).unwrap();
        assert_eq!(provider.provider_type(), VaultProviderType::Memory);
    }

    #[test]
    fn test_create_vault_provider_file() {
        let config = VaultConfig::file("/tmp/test_vault.json");
        let provider = create_vault_provider(&config).unwrap();
        assert_eq!(provider.provider_type(), VaultProviderType::File);
    }

    #[test]
    fn test_create_vault_provider_file_missing_path() {
        let config = VaultConfig {
            provider: VaultProviderType::File,
            file_path: None,
            ..VaultConfig::default()
        };
        let result = create_vault_provider(&config);
        assert!(result.is_err());
    }
}
