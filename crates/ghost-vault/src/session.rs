//! Session management and health monitoring
//!
//! This module provides session tracking and health monitoring for
//! multi-tenant session state management.

use std::collections::HashMap;

use ghost_schema::{
    GhostError, Platform, SessionEntry, SessionHealthResult, SessionStatus, SessionData,
};

/// Session manager for tracking session health
///
/// The SessionManager tracks active sessions and provides health monitoring
/// with indexes for efficient lookup by platform and tenant.
pub struct SessionManager {
    /// Active sessions indexed by session_id
    sessions: HashMap<String, SessionEntry>,
    /// Sessions indexed by platform
    by_platform: HashMap<Platform, Vec<String>>,
    /// Sessions indexed by tenant
    by_tenant: HashMap<String, Vec<String>>,
    /// Session health checker
    health_checker: SessionHealthChecker,
}

impl SessionManager {
    /// Creates a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            by_platform: HashMap::new(),
            by_tenant: HashMap::new(),
            health_checker: SessionHealthChecker::new(),
        }
    }

    /// Creates a new session manager with capacity hint
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            sessions: HashMap::with_capacity(capacity),
            by_platform: HashMap::new(),
            by_tenant: HashMap::new(),
            health_checker: SessionHealthChecker::new(),
        }
    }

    /// Registers a new session
    ///
    /// If a session with the same ID already exists, it will be replaced.
    pub fn register(&mut self, entry: SessionEntry) {
        let session_id = entry.session_id.clone();
        let platform = entry.platform;
        let tenant_id = entry.tenant_id.clone();

        // Remove old indexes if updating
        if let Some(old_session) = self.sessions.get(&session_id) {
            if let Some(ids) = self.by_platform.get_mut(&old_session.platform) {
                ids.retain(|id| id != &session_id);
            }
            if let Some(ids) = self.by_tenant.get_mut(&old_session.tenant_id) {
                ids.retain(|id| id != &session_id);
            }
        }

        // Index by platform
        self.by_platform
            .entry(platform)
            .or_default()
            .push(session_id.clone());

        // Index by tenant
        self.by_tenant
            .entry(tenant_id)
            .or_default()
            .push(session_id.clone());

        self.sessions.insert(session_id, entry);
    }

    /// Creates and registers a new session from components
    pub fn create_session(
        &mut self,
        session_id: impl Into<String>,
        tenant_id: impl Into<String>,
        platform: Platform,
        data: SessionData,
    ) -> &SessionEntry {
        let entry = SessionEntry::new(session_id, tenant_id, platform, data);
        let id = entry.session_id.clone();
        self.register(entry);
        self.sessions.get(&id).unwrap()
    }

    /// Unregisters a session
    ///
    /// Returns the removed session, or None if not found.
    pub fn unregister(&mut self, session_id: &str) -> Option<SessionEntry> {
        let session = self.sessions.remove(session_id);

        if let Some(ref s) = session {
            // Clean up platform index
            if let Some(ids) = self.by_platform.get_mut(&s.platform) {
                ids.retain(|id| id != session_id);
                if ids.is_empty() {
                    self.by_platform.remove(&s.platform);
                }
            }

            // Clean up tenant index
            if let Some(ids) = self.by_tenant.get_mut(&s.tenant_id) {
                ids.retain(|id| id != session_id);
                if ids.is_empty() {
                    self.by_tenant.remove(&s.tenant_id);
                }
            }
        }

        session
    }

    /// Gets a session by ID
    pub fn get(&self, session_id: &str) -> Option<&SessionEntry> {
        self.sessions.get(session_id)
    }

    /// Gets a mutable reference to a session
    pub fn get_mut(&mut self, session_id: &str) -> Option<&mut SessionEntry> {
        self.sessions.get_mut(session_id)
    }

    /// Gets an active (usable) session for a platform
    pub fn get_active(&self, platform: Platform) -> Option<&SessionEntry> {
        self.by_platform
            .get(&platform)
            .and_then(|ids| {
                ids.iter()
                    .filter_map(|id| self.sessions.get(id))
                    .find(|s| s.status.is_usable())
            })
    }

    /// Gets all sessions for a tenant
    pub fn get_for_tenant(&self, tenant_id: &str) -> Vec<&SessionEntry> {
        self.by_tenant
            .get(tenant_id)
            .map(|ids| ids.iter().filter_map(|id| self.sessions.get(id)).collect())
            .unwrap_or_default()
    }

    /// Gets active sessions for a tenant
    pub fn get_active_for_tenant(&self, tenant_id: &str) -> Vec<&SessionEntry> {
        self.get_for_tenant(tenant_id)
            .into_iter()
            .filter(|s| s.status.is_usable())
            .collect()
    }

    /// Gets sessions for a specific platform
    pub fn get_for_platform(&self, platform: Platform) -> Vec<&SessionEntry> {
        self.by_platform
            .get(&platform)
            .map(|ids| ids.iter().filter_map(|id| self.sessions.get(id)).collect())
            .unwrap_or_default()
    }

    /// Updates session status
    pub fn update_status(&mut self, session_id: &str, status: SessionStatus) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.set_status(status);
        }
    }

    /// Marks a session as rate limited
    pub fn mark_rate_limited(&mut self, session_id: &str) {
        self.update_status(session_id, SessionStatus::RateLimited);
    }

    /// Marks a session as suspended
    pub fn mark_suspended(&mut self, session_id: &str) {
        self.update_status(session_id, SessionStatus::Suspended);
    }

    /// Marks a session as active
    pub fn mark_active(&mut self, session_id: &str) {
        self.update_status(session_id, SessionStatus::Active);
    }

    /// Runs health checks on all sessions
    ///
    /// Returns health check results for all sessions.
    pub async fn check_health(&mut self) -> Result<Vec<SessionHealthResult>, GhostError> {
        let mut results = Vec::with_capacity(self.sessions.len());

        for session in self.sessions.values_mut() {
            let result = self.health_checker.check(session).await;
            session.status = if result.is_healthy {
                SessionStatus::Active
            } else {
                SessionStatus::RequiresAttention
            };
            session.health_result = Some(result.clone());
            results.push(result);
        }

        Ok(results)
    }

    /// Runs health check on a specific session
    pub async fn check_session_health(&mut self, session_id: &str) -> Option<SessionHealthResult> {
        let session = self.sessions.get_mut(session_id)?;
        let result = self.health_checker.check(session).await;
        session.status = if result.is_healthy {
            SessionStatus::Active
        } else {
            SessionStatus::RequiresAttention
        };
        session.health_result = Some(result.clone());
        Some(result)
    }

    /// Returns sessions filtered by status
    pub fn by_status(&self, status: SessionStatus) -> Vec<&SessionEntry> {
        self.sessions
            .values()
            .filter(|s| s.status == status)
            .collect()
    }

    /// Returns all platform IDs
    pub fn platforms(&self) -> impl Iterator<Item = Platform> + '_ {
        self.by_platform.keys().copied()
    }

    /// Returns all tenant IDs
    pub fn tenants(&self) -> impl Iterator<Item = &String> {
        self.by_tenant.keys()
    }

    /// Returns session count
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Returns whether the manager is empty
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Returns count of active sessions
    pub fn active_count(&self) -> usize {
        self.sessions.values().filter(|s| s.status.is_usable()).count()
    }

    /// Clears all sessions
    pub fn clear(&mut self) {
        self.sessions.clear();
        self.by_platform.clear();
        self.by_tenant.clear();
    }

    /// Gets statistics about sessions
    pub fn stats(&self) -> SessionManagerStats {
        let mut stats = SessionManagerStats {
            total: self.sessions.len(),
            ..SessionManagerStats::default()
        };

        for session in self.sessions.values() {
            match session.status {
                SessionStatus::Active => stats.active += 1,
                SessionStatus::CoolingDown => stats.cooling_down += 1,
                SessionStatus::RateLimited => stats.rate_limited += 1,
                SessionStatus::Suspended => stats.suspended += 1,
                SessionStatus::Expired => stats.expired += 1,
                SessionStatus::RequiresAttention => stats.requires_attention += 1,
            }
        }

        stats
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session health checker
///
/// Performs health checks on sessions to verify they are still valid.
pub struct SessionHealthChecker {
    /// Check interval in seconds
    check_interval_secs: u64,
    /// Endpoint to use for checking
    check_endpoint: String,
}

impl SessionHealthChecker {
    /// Creates a new health checker
    pub fn new() -> Self {
        Self {
            check_interval_secs: 300,
            check_endpoint: "get_own_profile".to_string(),
        }
    }

    /// Creates a health checker with custom settings
    pub fn with_settings(check_interval_secs: u64, check_endpoint: impl Into<String>) -> Self {
        Self {
            check_interval_secs,
            check_endpoint: check_endpoint.into(),
        }
    }

    /// Checks session health
    ///
    /// This performs a basic health check. In production, this would
    /// make actual API calls to verify the session is still valid.
    pub async fn check(&self, session: &SessionEntry) -> SessionHealthResult {
        // Basic health check logic
        // In production, this would make actual API calls to verify the session

        let start = std::time::Instant::now();

        // Simulate health check based on session status
        let is_healthy = matches!(session.status, SessionStatus::Active);

        let latency_ms = start.elapsed().as_millis() as u64;

        if is_healthy {
            SessionHealthResult::healthy(latency_ms)
        } else {
            let error = format!("Session status: {:?}", session.status);
            SessionHealthResult::unhealthy(error, "Refresh session credentials")
        }
    }

    /// Returns the check interval in seconds
    pub fn check_interval(&self) -> u64 {
        self.check_interval_secs
    }

    /// Returns the check endpoint
    pub fn endpoint(&self) -> &str {
        &self.check_endpoint
    }
}

impl Default for SessionHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about session management
#[derive(Debug, Clone, Default)]
pub struct SessionManagerStats {
    /// Total number of sessions
    pub total: usize,
    /// Number of active sessions
    pub active: usize,
    /// Number of cooling down sessions
    pub cooling_down: usize,
    /// Number of rate limited sessions
    pub rate_limited: usize,
    /// Number of suspended sessions
    pub suspended: usize,
    /// Number of expired sessions
    pub expired: usize,
    /// Number of sessions requiring attention
    pub requires_attention: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session(id: &str, tenant_id: &str, platform: Platform) -> SessionEntry {
        SessionEntry::new(
            id,
            tenant_id,
            platform,
            SessionData::from_cookies("test_cookie=value"),
        )
    }

    #[test]
    fn test_session_manager_new() {
        let manager = SessionManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_session_manager_default() {
        let manager = SessionManager::default();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_session_manager_register() {
        let mut manager = SessionManager::new();
        let session = create_test_session("session1", "tenant1", Platform::X);

        manager.register(session);
        assert_eq!(manager.len(), 1);
        assert!(manager.get("session1").is_some());
    }

    #[test]
    fn test_session_manager_unregister() {
        let mut manager = SessionManager::new();
        manager.register(create_test_session("session1", "tenant1", Platform::X));

        let removed = manager.unregister("session1");
        assert!(removed.is_some());
        assert!(manager.is_empty());
    }

    #[test]
    fn test_session_manager_get_for_tenant() {
        let mut manager = SessionManager::new();
        manager.register(create_test_session("s1", "tenant1", Platform::X));
        manager.register(create_test_session("s2", "tenant1", Platform::Threads));
        manager.register(create_test_session("s3", "tenant2", Platform::X));

        let results = manager.get_for_tenant("tenant1");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_session_manager_get_for_platform() {
        let mut manager = SessionManager::new();
        manager.register(create_test_session("s1", "tenant1", Platform::X));
        manager.register(create_test_session("s2", "tenant1", Platform::X));
        manager.register(create_test_session("s3", "tenant2", Platform::Threads));

        let results = manager.get_for_platform(Platform::X);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_session_manager_update_status() {
        let mut manager = SessionManager::new();
        manager.register(create_test_session("s1", "tenant1", Platform::X));

        manager.update_status("s1", SessionStatus::RateLimited);
        assert_eq!(manager.get("s1").unwrap().status, SessionStatus::RateLimited);
    }

    #[test]
    fn test_session_manager_get_active() {
        let mut manager = SessionManager::new();
        let mut session = create_test_session("s1", "tenant1", Platform::X);
        session.status = SessionStatus::Active;
        manager.register(session);

        let mut session2 = create_test_session("s2", "tenant1", Platform::X);
        session2.status = SessionStatus::Suspended;
        manager.register(session2);

        let active = manager.get_active(Platform::X);
        assert_eq!(active.unwrap().session_id, "s1");
    }

    #[test]
    fn test_session_manager_by_status() {
        let mut manager = SessionManager::new();

        let mut s1 = create_test_session("s1", "tenant1", Platform::X);
        s1.status = SessionStatus::Active;
        manager.register(s1);

        let mut s2 = create_test_session("s2", "tenant1", Platform::X);
        s2.status = SessionStatus::Suspended;
        manager.register(s2);

        let active = manager.by_status(SessionStatus::Active);
        assert_eq!(active.len(), 1);

        let suspended = manager.by_status(SessionStatus::Suspended);
        assert_eq!(suspended.len(), 1);
    }

    #[test]
    fn test_session_manager_stats() {
        let mut manager = SessionManager::new();

        let mut s1 = create_test_session("s1", "tenant1", Platform::X);
        s1.status = SessionStatus::Active;
        manager.register(s1);

        let mut s2 = create_test_session("s2", "tenant1", Platform::X);
        s2.status = SessionStatus::RateLimited;
        manager.register(s2);

        let stats = manager.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.active, 1);
        assert_eq!(stats.rate_limited, 1);
    }

    #[tokio::test]
    async fn test_session_health_checker() {
        let checker = SessionHealthChecker::new();

        let session = create_test_session("s1", "tenant1", Platform::X);
        let result = checker.check(&session).await;
        assert!(result.is_healthy);
    }

    #[tokio::test]
    async fn test_session_manager_check_health() {
        let mut manager = SessionManager::new();
        manager.register(create_test_session("s1", "tenant1", Platform::X));
        manager.register(create_test_session("s2", "tenant1", Platform::Threads));

        let results = manager.check_health().await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_session_manager_clear() {
        let mut manager = SessionManager::new();
        manager.register(create_test_session("s1", "tenant1", Platform::X));
        manager.register(create_test_session("s2", "tenant2", Platform::Threads));

        manager.clear();
        assert!(manager.is_empty());
    }
}
