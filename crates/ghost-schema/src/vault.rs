//! Vault types for secret management, proxy pools, credentials, and sessions
//!
//! This module provides types for managing secrets, proxies, credentials, and sessions.
//! Only memory and file-based storage are supported (no third-party integrations).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{GhostError, Platform, ProxyConfig, SessionData};

// ============================================================================
// Vault Provider Types
// ============================================================================

/// Type of vault provider (only memory and file-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultProviderType {
    /// In-memory storage (non-persistent)
    Memory,
    /// File-based storage (persistent)
    File,
}

impl VaultProviderType {
    /// Returns whether this provider is persistent
    pub fn is_persistent(&self) -> bool {
        // TODO: Implement persistence check
        matches!(self, VaultProviderType::File)
    }

    /// Returns the display name
    pub fn display_name(&self) -> &'static str {
        // TODO: Implement display name
        match self {
            VaultProviderType::Memory => "Memory Vault",
            VaultProviderType::File => "File Vault",
        }
    }
}

/// Vault configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Provider type
    pub provider: VaultProviderType,
    /// Path for file-based storage
    pub file_path: Option<String>,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// Whether to encrypt cached secrets
    pub cache_encrypted: bool,
    /// Auto-save interval for file-based storage (seconds)
    pub auto_save_interval_secs: u64,
}

impl VaultConfig {
    /// Creates a new vault config
    pub fn new(provider: VaultProviderType) -> Self {
        // TODO: Implement vault config construction
        Self {
            provider,
            file_path: None,
            cache_ttl_secs: 300,
            cache_encrypted: false,
            auto_save_interval_secs: 60,
        }
    }

    /// Creates a memory vault config
    pub fn memory() -> Self {
        // TODO: Implement memory config
        Self::new(VaultProviderType::Memory)
    }

    /// Creates a file vault config
    pub fn file(path: impl Into<String>) -> Self {
        // TODO: Implement file config
        Self {
            file_path: Some(path.into()),
            ..Self::new(VaultProviderType::File)
        }
    }

    /// Sets the cache TTL
    pub fn with_cache_ttl(mut self, secs: u64) -> Self {
        // TODO: Implement cache TTL setter
        self.cache_ttl_secs = secs;
        self
    }

    /// Enables cache encryption
    pub fn with_encryption(mut self) -> Self {
        // TODO: Implement encryption setter
        self.cache_encrypted = true;
        self
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), GhostError> {
        // TODO: Implement config validation
        if self.provider == VaultProviderType::File && self.file_path.is_none() {
            return Err(GhostError::ValidationError(
                "file_path required for File provider".into(),
            ));
        }
        Ok(())
    }
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self::memory()
    }
}

/// Cached secret entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSecret {
    /// The secret value
    pub value: String,
    /// Timestamp when cached
    pub cached_at: i64,
    /// TTL in seconds
    pub ttl_secs: u64,
}

impl CachedSecret {
    /// Creates a new cached secret
    pub fn new(value: impl Into<String>, ttl_secs: u64) -> Self {
        // TODO: Implement cached secret construction
        Self {
            value: value.into(),
            cached_at: 0, // TODO: Use actual timestamp
            ttl_secs,
        }
    }

    /// Checks if the secret is expired
    pub fn is_expired(&self) -> bool {
        // TODO: Implement expiration check
        false
    }

    /// Returns remaining TTL in seconds
    pub fn remaining_ttl(&self) -> u64 {
        // TODO: Implement remaining TTL calculation
        self.ttl_secs
    }
}

// ============================================================================
// Proxy Pool Types
// ============================================================================

/// Proxy rotation strategy
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyRotation {
    /// Round-robin rotation
    #[default]
    RoundRobin,
    /// Random selection
    Random,
    /// Least used first
    LeastUsed,
    /// Sticky by session
    StickySession,
    /// Geographic routing
    Geographic,
}

impl ProxyRotation {
    /// Returns the display name
    pub fn display_name(&self) -> &'static str {
        // TODO: Implement display name
        match self {
            ProxyRotation::RoundRobin => "Round Robin",
            ProxyRotation::Random => "Random",
            ProxyRotation::LeastUsed => "Least Used",
            ProxyRotation::StickySession => "Sticky Session",
            ProxyRotation::Geographic => "Geographic",
        }
    }
}

/// Proxy pool entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEntry {
    /// Proxy ID
    pub id: String,
    /// Proxy configuration
    pub config: ProxyConfig,
    /// Usage count
    pub usage_count: u64,
    /// Success count
    pub success_count: u64,
    /// Failure count
    pub failure_count: u64,
    /// Whether the proxy is active
    pub is_active: bool,
    /// Last used timestamp
    pub last_used_at: Option<i64>,
    /// Geographic region (for geographic routing)
    pub region: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl ProxyEntry {
    /// Creates a new proxy entry
    pub fn new(id: impl Into<String>, config: ProxyConfig) -> Self {
        // TODO: Implement proxy entry construction
        Self {
            id: id.into(),
            config,
            usage_count: 0,
            success_count: 0,
            failure_count: 0,
            is_active: true,
            last_used_at: None,
            region: None,
            tags: Vec::new(),
        }
    }

    /// Records a successful use
    pub fn record_success(&mut self) {
        // TODO: Implement success recording
        self.usage_count += 1;
        self.success_count += 1;
    }

    /// Records a failed use
    pub fn record_failure(&mut self) {
        // TODO: Implement failure recording
        self.usage_count += 1;
        self.failure_count += 1;
    }

    /// Returns the success rate
    pub fn success_rate(&self) -> f64 {
        // TODO: Implement success rate calculation
        if self.usage_count == 0 {
            1.0
        } else {
            self.success_count as f64 / self.usage_count as f64
        }
    }

    /// Sets the region
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        // TODO: Implement region setter
        self.region = Some(region.into());
        self
    }

    /// Adds a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        // TODO: Implement tag addition
        self.tags.push(tag.into());
        self
    }
}

/// Proxy pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPoolConfig {
    /// Rotation strategy
    pub rotation: ProxyRotation,
    /// Minimum health score to use proxy
    pub min_health_score: f64,
    /// Maximum failures before disabling
    pub max_failures: u32,
    /// Cooldown period after failures (seconds)
    pub cooldown_secs: u64,
    /// Enable automatic health checks
    pub health_check_enabled: bool,
    /// Health check interval (seconds)
    pub health_check_interval_secs: u64,
}

impl ProxyPoolConfig {
    /// Creates a new proxy pool config
    pub fn new() -> Self {
        // TODO: Implement proxy pool config construction
        Self {
            rotation: ProxyRotation::default(),
            min_health_score: 0.5,
            max_failures: 5,
            cooldown_secs: 300,
            health_check_enabled: true,
            health_check_interval_secs: 60,
        }
    }

    /// Sets the rotation strategy
    pub fn with_rotation(mut self, rotation: ProxyRotation) -> Self {
        // TODO: Implement rotation setter
        self.rotation = rotation;
        self
    }
}

impl Default for ProxyPoolConfig {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Credential Store Types
// ============================================================================

/// Credential entry for storing session/cookie data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialEntry {
    /// Credential ID
    pub id: String,
    /// Tenant ID this credential belongs to
    pub tenant_id: String,
    /// Platform this credential is for
    pub platform: Platform,
    /// Session data
    pub session: SessionData,
    /// Whether the credential is active
    pub is_active: bool,
    /// Usage count
    pub usage_count: u64,
    /// Last used timestamp
    pub last_used_at: Option<i64>,
    /// Last validated timestamp
    pub last_validated_at: Option<i64>,
    /// Validation status
    pub validation_status: CredentialStatus,
    /// Associated proxy ID (for sticky sessions)
    pub sticky_proxy_id: Option<String>,
    /// Notes/metadata
    pub notes: Option<String>,
}

impl CredentialEntry {
    /// Creates a new credential entry
    pub fn new(
        id: impl Into<String>,
        tenant_id: impl Into<String>,
        platform: Platform,
        session: SessionData,
    ) -> Self {
        // TODO: Implement credential entry construction
        Self {
            id: id.into(),
            tenant_id: tenant_id.into(),
            platform,
            session,
            is_active: true,
            usage_count: 0,
            last_used_at: None,
            last_validated_at: None,
            validation_status: CredentialStatus::Unknown,
            sticky_proxy_id: None,
            notes: None,
        }
    }

    /// Records usage
    pub fn record_usage(&mut self) {
        // TODO: Implement usage recording
        self.usage_count += 1;
    }

    /// Sets validation status
    pub fn set_status(&mut self, status: CredentialStatus) {
        // TODO: Implement status setting
        self.validation_status = status;
    }

    /// Sets the sticky proxy
    pub fn with_sticky_proxy(mut self, proxy_id: impl Into<String>) -> Self {
        // TODO: Implement sticky proxy setter
        self.sticky_proxy_id = Some(proxy_id.into());
        self
    }
}

/// Credential validation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStatus {
    /// Not yet validated
    Unknown,
    /// Valid and working
    Valid,
    /// Invalid or expired
    Invalid,
    /// Rate limited
    RateLimited,
    /// Suspended/banned
    Suspended,
    /// Requires challenge
    ChallengeRequired,
}

impl CredentialStatus {
    /// Returns whether this status is usable
    pub fn is_usable(&self) -> bool {
        // TODO: Implement usability check
        matches!(self, CredentialStatus::Unknown | CredentialStatus::Valid)
    }
}

// ============================================================================
// Session Store Types
// ============================================================================

/// Session entry for tracking session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    /// Session ID
    pub session_id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Platform
    pub platform: Platform,
    /// Session data
    pub data: SessionData,
    /// Session status
    pub status: SessionStatus,
    /// Created timestamp
    pub created_at: i64,
    /// Last activity timestamp
    pub last_activity_at: i64,
    /// Health check result
    pub health_result: Option<SessionHealthResult>,
}

impl SessionEntry {
    /// Creates a new session entry
    pub fn new(
        session_id: impl Into<String>,
        tenant_id: impl Into<String>,
        platform: Platform,
        data: SessionData,
    ) -> Self {
        // TODO: Implement session entry construction
        Self {
            session_id: session_id.into(),
            tenant_id: tenant_id.into(),
            platform,
            data,
            status: SessionStatus::Active,
            created_at: 0, // TODO: Use actual timestamp
            last_activity_at: 0,
            health_result: None,
        }
    }

    /// Updates the last activity
    pub fn touch(&mut self) {
        // TODO: Implement activity update
    }

    /// Sets the status
    pub fn set_status(&mut self, status: SessionStatus) {
        // TODO: Implement status setting
        self.status = status;
    }
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is active and healthy
    Active,
    /// Session is cooling down
    CoolingDown,
    /// Session is rate limited
    RateLimited,
    /// Session is suspended
    Suspended,
    /// Session is expired
    Expired,
    /// Session requires attention
    RequiresAttention,
}

impl SessionStatus {
    /// Returns whether the session is usable
    pub fn is_usable(&self) -> bool {
        // TODO: Implement usability check
        matches!(self, SessionStatus::Active)
    }
}

/// Session health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHealthResult {
    /// Whether the session is healthy
    pub is_healthy: bool,
    /// Timestamp of the check
    pub checked_at: i64,
    /// Latency in ms
    pub latency_ms: u64,
    /// Error message if unhealthy
    pub error: Option<String>,
    /// Recommended action
    pub recommended_action: Option<String>,
}

impl SessionHealthResult {
    /// Creates a healthy result
    pub fn healthy(latency_ms: u64) -> Self {
        // TODO: Implement healthy result construction
        Self {
            is_healthy: true,
            checked_at: 0, // TODO: Use actual timestamp
            latency_ms,
            error: None,
            recommended_action: None,
        }
    }

    /// Creates an unhealthy result
    pub fn unhealthy(error: impl Into<String>, recommended_action: impl Into<String>) -> Self {
        // TODO: Implement unhealthy result construction
        Self {
            is_healthy: false,
            checked_at: 0,
            latency_ms: 0,
            error: Some(error.into()),
            recommended_action: Some(recommended_action.into()),
        }
    }
}

// ============================================================================
// Injection Types
// ============================================================================

/// Options for context injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionOptions {
    /// Override proxy
    pub proxy_override: Option<ProxyConfig>,
    /// Override session
    pub session_override: Option<SessionData>,
    /// Force specific worker
    pub worker_override: Option<String>,
    /// Force specific tier
    pub tier_override: Option<crate::CapabilityTier>,
    /// Skip health check
    pub skip_health_check: bool,
    /// Custom timeout
    pub timeout_ms: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl InjectionOptions {
    /// Creates new injection options
    pub fn new() -> Self {
        // TODO: Implement injection options construction
        Self {
            proxy_override: None,
            session_override: None,
            worker_override: None,
            tier_override: None,
            skip_health_check: false,
            timeout_ms: None,
            metadata: HashMap::new(),
        }
    }

    /// Sets the proxy override
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        // TODO: Implement proxy setter
        self.proxy_override = Some(proxy);
        self
    }

    /// Sets the session override
    pub fn with_session(mut self, session: SessionData) -> Self {
        // TODO: Implement session setter
        self.session_override = Some(session);
        self
    }

    /// Sets the worker override
    pub fn with_worker(mut self, worker_id: impl Into<String>) -> Self {
        // TODO: Implement worker setter
        self.worker_override = Some(worker_id.into());
        self
    }

    /// Sets the timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        // TODO: Implement timeout setter
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Adds metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        // TODO: Implement metadata addition
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl Default for InjectionOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of context injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionResult {
    /// The built context
    pub context: crate::GhostContext,
    /// Selected proxy ID (if any)
    pub selected_proxy_id: Option<String>,
    /// Selected credential ID (if any)
    pub selected_credential_id: Option<String>,
    /// Whether a fallback was used
    pub used_fallback: bool,
    /// Injection warnings
    pub warnings: Vec<String>,
}

impl InjectionResult {
    /// Creates a new injection result
    pub fn new(context: crate::GhostContext) -> Self {
        // TODO: Implement injection result construction
        Self {
            context,
            selected_proxy_id: None,
            selected_credential_id: None,
            used_fallback: false,
            warnings: Vec::new(),
        }
    }

    /// Adds a warning
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        // TODO: Implement warning addition
        self.warnings.push(warning.into());
    }
}
