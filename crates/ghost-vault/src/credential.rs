//! Credential management
//!
//! Types imported from ghost-schema - the single source of truth.

use std::collections::HashMap;

use ghost_schema::{
    GhostError, Platform, SessionData, CredentialEntry,
};

/// Credential store for managing session credentials
pub struct CredentialStore {
    /// Credentials by ID
    credentials: HashMap<String, CredentialEntry>,
    /// Credentials indexed by tenant and platform
    by_tenant_platform: HashMap<(String, Platform), Vec<String>>,
    /// Vault reference for secret retrieval
    vault: Option<String>,
}

impl CredentialStore {
    /// Creates a new credential store
    pub fn new() -> Self {
        // TODO: Implement credential store construction
        Self {
            credentials: HashMap::new(),
            by_tenant_platform: HashMap::new(),
            vault: None,
        }
    }

    /// Adds a credential
    pub fn add_credential(&mut self, credential: CredentialEntry) {
        // TODO: Implement credential addition with indexing
        let id = credential.id.clone();
        let tenant = credential.tenant_id.clone();
        let platform = credential.platform;

        // Index by tenant and platform
        if let Some(ref tenant_id) = tenant {
            self.by_tenant_platform
                .entry((tenant_id.clone(), platform))
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        self.credentials.insert(id, credential);
    }

    /// Gets a credential by ID
    pub fn get(&self, id: &str) -> Option<&CredentialEntry> {
        // TODO: Implement credential retrieval
        self.credentials.get(id)
    }

    /// Gets credentials for a tenant and platform
    pub fn get_for_tenant(&self, tenant_id: &str, platform: Platform) -> Vec<&CredentialEntry> {
        // TODO: Implement tenant-specific credential lookup
        self.by_tenant_platform
            .get(&(tenant_id.to_string(), platform))
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.credentials.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Removes a credential
    pub fn remove(&mut self, id: &str) -> Option<CredentialEntry> {
        // TODO: Implement credential removal with index cleanup
        let credential = self.credentials.remove(id);

        if let Some(ref cred) = credential {
            if let Some(ref tenant_id) = cred.tenant_id {
                if let Some(ids) = self.by_tenant_platform.get_mut(&(tenant_id.clone(), cred.platform)) {
                    ids.retain(|i| i != id);
                }
            }
        }

        credential
    }

    /// Lists all credential IDs
    pub fn list(&self) -> Vec<&String> {
        // TODO: Implement credential listing
        self.credentials.keys().collect()
    }

    /// Validates all credentials
    pub async fn validate_all(&self) -> Result<Vec<(String, bool)>, GhostError> {
        // TODO: Implement batch validation
        let mut results = Vec::new();
        for (id, cred) in &self.credentials {
            let valid = cred.validate().is_ok();
            results.push((id.clone(), valid));
        }
        Ok(results)
    }

    /// Marks a credential as invalid
    pub fn mark_invalid(&mut self, id: &str) {
        // TODO: Implement invalid marking
        if let Some(cred) = self.credentials.get_mut(id) {
            cred.mark_invalid();
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
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}
