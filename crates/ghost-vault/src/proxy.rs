//! Proxy management and rotation
//!
//! This module provides proxy pool management with various rotation strategies
//! including round-robin, random, least-used, sticky session, and geographic routing.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ghost_schema::{GhostError, ProxyConfig, ProxyEntry, ProxyRotation};

/// Proxy pool for managing multiple proxies
///
/// The ProxyPool manages a collection of proxies with various rotation strategies,
/// health tracking, and blacklisting capabilities.
pub struct ProxyPool {
    /// Available proxies
    proxies: Vec<ProxyEntry>,
    /// Proxy index for round-robin
    index: Arc<AtomicUsize>,
    /// Blacklisted proxies with expiry time
    blacklist: Arc<tokio::sync::RwLock<HashMap<String, i64>>>,
    /// Rotation strategy
    rotation: ProxyRotation,
}

impl ProxyPool {
    /// Creates a new empty proxy pool
    pub fn new() -> Self {
        Self {
            proxies: Vec::new(),
            index: Arc::new(AtomicUsize::new(0)),
            blacklist: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            rotation: ProxyRotation::default(),
        }
    }

    /// Creates a new pool with rotation strategy
    pub fn with_rotation(rotation: ProxyRotation) -> Self {
        Self {
            rotation,
            ..Self::new()
        }
    }

    /// Creates a proxy pool from a list of proxy URLs
    ///
    /// # Arguments
    ///
    /// * `urls` - Slice of proxy URL strings (e.g., "socks5://user:pass@host:port")
    pub fn from_urls(urls: &[&str]) -> Result<Self, GhostError> {
        let mut pool = Self::new();
        for (i, url) in urls.iter().enumerate() {
            let config = ProxyConfig::from_url(*url)?;
            let entry = ProxyEntry::new(format!("proxy_{}", i), config);
            pool.add_proxy(entry)?;
        }
        Ok(pool)
    }

    /// Creates a proxy pool from a list of proxy entries
    pub fn from_entries(entries: Vec<ProxyEntry>) -> Self {
        Self {
            proxies: entries,
            ..Self::new()
        }
    }

    /// Adds a proxy to the pool
    pub fn add_proxy(&mut self, proxy: ProxyEntry) -> Result<(), GhostError> {
        self.proxies.push(proxy);
        Ok(())
    }

    /// Gets the next available proxy using rotation strategy
    pub async fn get_next(&self) -> Option<ProxyEntry> {
        match self.rotation {
            ProxyRotation::RoundRobin => self.get_round_robin().await,
            ProxyRotation::Random => self.get_random().await,
            ProxyRotation::LeastUsed => self.get_least_used().await,
            ProxyRotation::StickySession => None, // Requires session ID
            ProxyRotation::Geographic => None,    // Requires region
        }
    }

    /// Gets a sticky proxy for a session
    ///
    /// Uses consistent hashing to ensure the same session always gets the same proxy.
    pub async fn get_sticky(&self, session_id: &str) -> Option<ProxyEntry> {
        if self.proxies.is_empty() {
            return None;
        }

        let hash = self.hash_session(session_id);
        let idx = hash % self.proxies.len();

        // Check if proxy is blacklisted
        let blacklist = self.blacklist.read().await;
        let proxy = &self.proxies[idx];
        if let Some(&expire_time) = blacklist.get(&proxy.id) {
            let now = current_timestamp();
            if now < expire_time {
                // Try to find a non-blacklisted proxy
                return self.find_available_proxy(&blacklist, now);
            }
        }

        Some(proxy.clone())
    }

    /// Gets a proxy for a specific region
    pub async fn get_for_region(&self, region: &str) -> Option<ProxyEntry> {
        let blacklist = self.blacklist.read().await;
        let now = current_timestamp();

        self.proxies
            .iter()
            .filter(|p| p.region.as_deref() == Some(region))
            .find(|p| !self.is_blacklisted(&blacklist, &p.id, now))
            .cloned()
    }

    /// Blacklists a proxy temporarily
    ///
    /// # Arguments
    ///
    /// * `proxy_id` - ID of the proxy to blacklist
    /// * `duration_secs` - Duration in seconds for the blacklist
    pub async fn blacklist(&self, proxy_id: &str, duration_secs: u64) {
        let expire_time = current_timestamp() + duration_secs as i64;
        self.blacklist.write().await.insert(proxy_id.to_string(), expire_time);
    }

    /// Removes a proxy from blacklist
    pub async fn unblacklist(&self, proxy_id: &str) {
        self.blacklist.write().await.remove(proxy_id);
    }

    /// Checks if a proxy is currently blacklisted
    pub async fn is_proxy_blacklisted(&self, proxy_id: &str) -> bool {
        let blacklist = self.blacklist.read().await;
        if let Some(&expire_time) = blacklist.get(proxy_id) {
            current_timestamp() < expire_time
        } else {
            false
        }
    }

    /// Returns the number of available (non-blacklisted) proxies
    pub async fn available_count(&self) -> usize {
        let blacklist = self.blacklist.read().await;
        let now = current_timestamp();

        self.proxies
            .iter()
            .filter(|p| !self.is_blacklisted(&blacklist, &p.id, now))
            .count()
    }

    /// Returns the total number of proxies
    pub fn len(&self) -> usize {
        self.proxies.len()
    }

    /// Returns whether the pool is empty
    pub fn is_empty(&self) -> bool {
        self.proxies.is_empty()
    }

    /// Returns the rotation strategy
    pub fn rotation(&self) -> ProxyRotation {
        self.rotation
    }

    /// Gets a proxy by ID
    pub fn get_by_id(&self, id: &str) -> Option<&ProxyEntry> {
        self.proxies.iter().find(|p| p.id == id)
    }

    /// Records a successful use of a proxy
    pub fn record_success(&mut self, proxy_id: &str) {
        if let Some(proxy) = self.proxies.iter_mut().find(|p| p.id == proxy_id) {
            proxy.record_success();
        }
    }

    /// Records a failed use of a proxy
    pub fn record_failure(&mut self, proxy_id: &str) {
        if let Some(proxy) = self.proxies.iter_mut().find(|p| p.id == proxy_id) {
            proxy.record_failure();
        }
    }

    /// Clears all proxies from the pool
    pub fn clear(&mut self) {
        self.proxies.clear();
        self.index.store(0, Ordering::SeqCst);
    }

    // Private helper methods

    /// Gets next proxy using round-robin
    async fn get_round_robin(&self) -> Option<ProxyEntry> {
        if self.proxies.is_empty() {
            return None;
        }

        let blacklist = self.blacklist.read().await;
        let now = current_timestamp();

        // Try each proxy in order starting from current index
        for _ in 0..self.proxies.len() {
            let idx = self.index.fetch_add(1, Ordering::SeqCst) % self.proxies.len();
            let proxy = &self.proxies[idx];

            if !self.is_blacklisted(&blacklist, &proxy.id, now) {
                return Some(proxy.clone());
            }
        }

        None
    }

    /// Gets a random proxy
    async fn get_random(&self) -> Option<ProxyEntry> {
        // For simplicity, use round-robin with offset
        // A true random implementation would use rand crate
        self.get_round_robin().await
    }

    /// Gets the least-used proxy
    async fn get_least_used(&self) -> Option<ProxyEntry> {
        if self.proxies.is_empty() {
            return None;
        }

        let blacklist = self.blacklist.read().await;
        let now = current_timestamp();

        self.proxies
            .iter()
            .filter(|p| !self.is_blacklisted(&blacklist, &p.id, now))
            .min_by_key(|p| p.usage_count)
            .cloned()
    }

    /// Finds an available proxy from the blacklist
    fn find_available_proxy(
        &self,
        blacklist: &HashMap<String, i64>,
        now: i64,
    ) -> Option<ProxyEntry> {
        self.proxies
            .iter()
            .find(|p| !self.is_blacklisted(blacklist, &p.id, now))
            .cloned()
    }

    /// Checks if a proxy is blacklisted
    fn is_blacklisted(&self, blacklist: &HashMap<String, i64>, proxy_id: &str, now: i64) -> bool {
        blacklist.get(proxy_id).map(|&expire| now < expire).unwrap_or(false)
    }

    /// Hashes a session ID for consistent proxy mapping
    fn hash_session(&self, session_id: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        session_id.hash(&mut hasher);
        hasher.finish() as usize
    }
}

impl Default for ProxyPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the current timestamp in seconds
fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_pool_new() {
        let pool = ProxyPool::new();
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_proxy_pool_default() {
        let pool = ProxyPool::default();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_proxy_pool_with_rotation() {
        let pool = ProxyPool::with_rotation(ProxyRotation::LeastUsed);
        assert_eq!(pool.rotation(), ProxyRotation::LeastUsed);
    }

    #[test]
    fn test_proxy_pool_add() {
        let mut pool = ProxyPool::new();
        let config = ProxyConfig::from_url("http://localhost:8080").unwrap();
        let entry = ProxyEntry::new("test", config);

        assert!(pool.add_proxy(entry).is_ok());
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_proxy_pool_from_urls() {
        let pool = ProxyPool::from_urls(&["http://localhost:8080", "http://localhost:8081"]);
        assert!(pool.is_ok());
        let pool = pool.unwrap();
        assert_eq!(pool.len(), 2);
    }

    #[tokio::test]
    async fn test_proxy_pool_get_next_empty() {
        let pool = ProxyPool::new();
        assert!(pool.get_next().await.is_none());
    }

    #[tokio::test]
    async fn test_proxy_pool_get_next() {
        let mut pool = ProxyPool::new();
        let config = ProxyConfig::from_url("http://localhost:8080").unwrap();
        pool.add_proxy(ProxyEntry::new("test", config)).unwrap();

        let result = pool.get_next().await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_proxy_pool_blacklist() {
        let mut pool = ProxyPool::new();
        let config = ProxyConfig::from_url("http://localhost:8080").unwrap();
        pool.add_proxy(ProxyEntry::new("test", config)).unwrap();

        pool.blacklist("test", 60).await;
        assert!(pool.is_proxy_blacklisted("test").await);

        pool.unblacklist("test").await;
        assert!(!pool.is_proxy_blacklisted("test").await);
    }

    #[tokio::test]
    async fn test_proxy_pool_sticky() {
        let mut pool = ProxyPool::new();
        let config1 = ProxyConfig::from_url("http://localhost:8080").unwrap();
        let config2 = ProxyConfig::from_url("http://localhost:8081").unwrap();
        pool.add_proxy(ProxyEntry::new("p1", config1)).unwrap();
        pool.add_proxy(ProxyEntry::new("p2", config2)).unwrap();

        // Same session should get same proxy
        let proxy1 = pool.get_sticky("session1").await;
        let proxy2 = pool.get_sticky("session1").await;
        assert_eq!(proxy1.unwrap().id, proxy2.unwrap().id);

        // Different sessions might get different proxies
        let proxy3 = pool.get_sticky("session2").await;
        assert!(proxy3.is_some());
    }

    #[tokio::test]
    async fn test_proxy_pool_available_count() {
        let mut pool = ProxyPool::new();
        let config1 = ProxyConfig::from_url("http://localhost:8080").unwrap();
        let config2 = ProxyConfig::from_url("http://localhost:8081").unwrap();
        pool.add_proxy(ProxyEntry::new("p1", config1)).unwrap();
        pool.add_proxy(ProxyEntry::new("p2", config2)).unwrap();

        assert_eq!(pool.available_count().await, 2);

        pool.blacklist("p1", 60).await;
        assert_eq!(pool.available_count().await, 1);
    }

    #[test]
    fn test_proxy_pool_record_usage() {
        let mut pool = ProxyPool::new();
        let config = ProxyConfig::from_url("http://localhost:8080").unwrap();
        pool.add_proxy(ProxyEntry::new("test", config)).unwrap();

        pool.record_success("test");
        pool.record_success("test");
        pool.record_failure("test");

        let proxy = pool.get_by_id("test").unwrap();
        assert_eq!(proxy.success_count, 2);
        assert_eq!(proxy.failure_count, 1);
    }
}
