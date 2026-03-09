//! Credential management
//!
//! This module provides credential storage and management for multi-tenant
//! session/cookie data with platform-specific organization.

use std::collections::HashMap;

use ghost_schema::{CredentialEntry, CredentialStatus, GhostError, Platform};

/// Credential store for managing session credentials
///
/// The CredentialStore manages credentials indexed by ID and provides
/// efficient lookups by tenant and platform.
pub struct CredentialStore {
    /// Credentials by ID
    credentials: HashMap<String, CredentialEntry>,
    /// Credentials indexed by tenant and platform
    by_tenant_platform: HashMap<(String, Platform), Vec<String>>,
}

impl CredentialStore {
    /// Creates a new credential store
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
            by_tenant_platform: HashMap::new(),
        }
    }

    /// Creates a new credential store with capacity hint
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            credentials: HashMap::with_capacity(capacity),
            by_tenant_platform: HashMap::new(),
        }
    }

    /// Adds a credential to the store
    ///
    /// If a credential with the same ID already exists, it will be replaced.
    pub fn add_credential(&mut self, credential: CredentialEntry) {
        let id = credential.id.clone();
        let tenant_id = credential.tenant_id.clone();
        let platform = credential.platform;

        // Remove old indexes if updating
        if let Some(old_cred) = self.credentials.get(&id) {
            if let Some(ids) = self.by_tenant_platform.get_mut(&(old_cred.tenant_id.clone(), old_cred.platform)) {
                ids.retain(|i| i != &id);
            }
        }

        // Index by tenant and platform
        self.by_tenant_platform
            .entry((tenant_id, platform))
            .or_default()
            .push(id.clone());

        self.credentials.insert(id, credential);
    }

    /// Gets a credential by ID
    pub fn get(&self, id: &str) -> Option<&CredentialEntry> {
        self.credentials.get(id)
    }

    /// Gets a mutable reference to a credential by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut CredentialEntry> {
        self.credentials.get_mut(id)
    }

    /// Gets all credentials for a specific tenant and platform
    pub fn get_for_tenant(&self, tenant_id: &str, platform: Platform) -> Vec<&CredentialEntry> {
        self.by_tenant_platform
            .get(&(tenant_id.to_string(), platform))
            .map(|ids| ids.iter().filter_map(|id| self.credentials.get(id)).collect())
            .unwrap_or_default()
    }

    /// Gets all active credentials for a tenant and platform
    pub fn get_active_for_tenant(&self, tenant_id: &str, platform: Platform) -> Vec<&CredentialEntry> {
        self.get_for_tenant(tenant_id, platform)
            .into_iter()
            .filter(|c| c.is_active && c.validation_status.is_usable())
            .collect()
    }

    /// Gets the first active credential for a tenant and platform
    pub fn get_first_active(&self, tenant_id: &str, platform: Platform) -> Option<&CredentialEntry> {
        self.get_active_for_tenant(tenant_id, platform).into_iter().next()
    }

    /// Removes a credential from the store
    ///
    /// Returns the removed credential, or None if not found.
    pub fn remove(&mut self, id: &str) -> Option<CredentialEntry> {
        let credential = self.credentials.remove(id);

        if let Some(ref cred) = credential {
            if let Some(ids) = self.by_tenant_platform.get_mut(&(cred.tenant_id.clone(), cred.platform)) {
                ids.retain(|i| i != id);
                // Clean up empty index entries
                if ids.is_empty() {
                    self.by_tenant_platform.remove(&(cred.tenant_id.clone(), cred.platform));
                }
            }
        }

        credential
    }

    /// Lists all credential IDs
    pub fn list(&self) -> Vec<&String> {
        self.credentials.keys().collect()
    }

    /// Lists all credentials
    pub fn list_all(&self) -> impl Iterator<Item = &CredentialEntry> {
        self.credentials.values()
    }

    /// Lists credentials for a specific platform
    pub fn list_for_platform(&self, platform: Platform) -> Vec<&CredentialEntry> {
        self.credentials
            .values()
            .filter(|c| c.platform == platform)
            .collect()
    }

    /// Validates all credentials
    ///
    /// Returns a vector of (credential_id, is_valid) pairs.
    /// This performs a basic check on credential validity.
    pub async fn validate_all(&self) -> Result<Vec<(String, bool)>, GhostError> {
        let mut results = Vec::with_capacity(self.credentials.len());
        for (id, cred) in &self.credentials {
            // Basic validation: check if session has required data
            let valid = cred.validation_status.is_usable() && cred.session.cookies.is_some();
            results.push((id.clone(), valid));
        }
        Ok(results)
    }

    /// Marks a credential as invalid
    pub fn mark_invalid(&mut self, id: &str, reason: CredentialStatus) {
        if let Some(cred) = self.credentials.get_mut(id) {
            cred.set_status(reason);
        }
    }

    /// Marks a credential as active/inactive
    pub fn set_active(&mut self, id: &str, active: bool) {
        if let Some(cred) = self.credentials.get_mut(id) {
            cred.is_active = active;
        }
    }

    /// Records usage of a credential
    pub fn record_usage(&mut self, id: &str) {
        if let Some(cred) = self.credentials.get_mut(id) {
            cred.record_usage();
        }
    }

    /// Returns the count of credentials
    pub fn len(&self) -> usize {
        self.credentials.len()
    }

    /// Returns whether the store is empty
    pub fn is_empty(&self) -> bool {
        self.credentials.is_empty()
    }

    /// Returns the count of active credentials
    pub fn active_count(&self) -> usize {
        self.credentials.values().filter(|c| c.is_active).count()
    }

    /// Returns the count of valid credentials
    pub fn valid_count(&self) -> usize {
        self.credentials
            .values()
            .filter(|c| c.validation_status.is_usable())
            .count()
    }

    /// Clears all credentials from the store
    pub fn clear(&mut self) {
        self.credentials.clear();
        self.by_tenant_platform.clear();
    }

    /// Gets statistics about the credential store
    pub fn stats(&self) -> CredentialStoreStats {
        let mut stats = CredentialStoreStats {
            total: self.credentials.len(),
            ..CredentialStoreStats::default()
        };

        for cred in self.credentials.values() {
            if cred.is_active {
                stats.active += 1;
            }
            if cred.validation_status.is_usable() {
                stats.valid += 1;
            }
            stats.total_usage += cred.usage_count;
        }

        stats
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the credential store
#[derive(Debug, Clone, Default)]
pub struct CredentialStoreStats {
    /// Total number of credentials
    pub total: usize,
    /// Number of active credentials
    pub active: usize,
    /// Number of valid credentials
    pub valid: usize,
    /// Total usage count across all credentials
    pub total_usage: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghost_schema::SessionData;

    fn create_test_credential(id: &str, tenant_id: &str, platform: Platform) -> CredentialEntry {
        CredentialEntry::new(
            id,
            tenant_id,
            platform,
            SessionData::from_cookies("test_cookie=value"),
        )
    }

    #[test]
    fn test_credential_store_new() {
        let store = CredentialStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_credential_store_default() {
        let store = CredentialStore::default();
        assert!(store.is_empty());
    }

    #[test]
    fn test_credential_store_add() {
        let mut store = CredentialStore::new();
        let cred = create_test_credential("cred1", "tenant1", Platform::X);

        store.add_credential(cred);
        assert_eq!(store.len(), 1);
        assert!(store.get("cred1").is_some());
    }

    #[test]
    fn test_credential_store_remove() {
        let mut store = CredentialStore::new();
        store.add_credential(create_test_credential("cred1", "tenant1", Platform::X));

        let removed = store.remove("cred1");
        assert!(removed.is_some());
        assert!(store.is_empty());
    }

    #[test]
    fn test_credential_store_get_for_tenant() {
        let mut store = CredentialStore::new();
        store.add_credential(create_test_credential("cred1", "tenant1", Platform::X));
        store.add_credential(create_test_credential("cred2", "tenant1", Platform::X));
        store.add_credential(create_test_credential("cred3", "tenant2", Platform::X));

        let results = store.get_for_tenant("tenant1", Platform::X);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_credential_store_get_active_for_tenant() {
        let mut store = CredentialStore::new();
        let mut cred = create_test_credential("cred1", "tenant1", Platform::X);
        cred.is_active = false;
        store.add_credential(cred);
        store.add_credential(create_test_credential("cred2", "tenant1", Platform::X));

        let results = store.get_active_for_tenant("tenant1", Platform::X);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "cred2");
    }

    #[test]
    fn test_credential_store_mark_invalid() {
        let mut store = CredentialStore::new();
        store.add_credential(create_test_credential("cred1", "tenant1", Platform::X));

        store.mark_invalid("cred1", CredentialStatus::Invalid);
        let cred = store.get("cred1").unwrap();
        assert_eq!(cred.validation_status, CredentialStatus::Invalid);
    }

    #[test]
    fn test_credential_store_set_active() {
        let mut store = CredentialStore::new();
        store.add_credential(create_test_credential("cred1", "tenant1", Platform::X));

        store.set_active("cred1", false);
        assert!(!store.get("cred1").unwrap().is_active);

        store.set_active("cred1", true);
        assert!(store.get("cred1").unwrap().is_active);
    }

    #[test]
    fn test_credential_store_record_usage() {
        let mut store = CredentialStore::new();
        store.add_credential(create_test_credential("cred1", "tenant1", Platform::X));

        store.record_usage("cred1");
        store.record_usage("cred1");
        assert_eq!(store.get("cred1").unwrap().usage_count, 2);
    }

    #[test]
    fn test_credential_store_clear() {
        let mut store = CredentialStore::new();
        store.add_credential(create_test_credential("cred1", "tenant1", Platform::X));
        store.add_credential(create_test_credential("cred2", "tenant2", Platform::Threads));

        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn test_credential_store_stats() {
        let mut store = CredentialStore::new();
        store.add_credential(create_test_credential("cred1", "tenant1", Platform::X));
        store.add_credential(create_test_credential("cred2", "tenant1", Platform::Threads));

        store.record_usage("cred1");
        store.record_usage("cred1");
        store.record_usage("cred2");

        let stats = store.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.active, 2);
        assert_eq!(stats.total_usage, 3);
    }

    #[tokio::test]
    async fn test_credential_store_validate_all() {
        let mut store = CredentialStore::new();
        store.add_credential(create_test_credential("cred1", "tenant1", Platform::X));

        let results = store.validate_all().await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
