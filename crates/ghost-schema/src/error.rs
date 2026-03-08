//! Error types for the Ghost AST

use thiserror::Error;

/// Main error type for ghost-api
#[derive(Debug, Error)]
pub enum GhostError {
    /// The requested feature is not yet implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Error during parsing of response data
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Network/HTTP error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Platform returned an error response
    #[error("Platform error: {0}")]
    PlatformError {
        code: u16,
        message: String,
        platform: crate::Platform,
    },

    /// Rate limit exceeded
    #[error("Rate limited: {0}")]
    RateLimited {
        retry_after: Option<u64>,
        platform: crate::Platform,
    },

    /// Authentication/authorization error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Session/credential expired or invalid
    #[error("Session expired: {0}")]
    SessionExpired(String),

    /// Account suspended or banned
    #[error("Account suspended: {0}")]
    AccountSuspended {
        account_id: String,
        platform: crate::Platform,
        reason: Option<String>,
    },

    /// Proxy error
    #[error("Proxy error: {0}")]
    ProxyError(String),

    /// Scraper worker error
    #[error("Scraper error: {worker}: {message}")]
    ScraperError {
        worker: String,
        message: String,
    },

    /// Adapter error (parsing platform-specific data)
    #[error("Adapter error: {0}")]
    AdapterError(String),

    /// Health check failed
    #[error("Health check failed for worker: {0}")]
    HealthCheckFailed(String),

    /// All workers exhausted
    #[error("All workers exhausted: {0}")]
    WorkersExhausted(String),

    /// Circuit breaker tripped
    #[error("Circuit breaker tripped for worker: {0}")]
    CircuitBreakerTripped(String),

    /// Budget exceeded
    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// WAF/Challenge detected
    #[error("WAF challenge detected: {0}")]
    WafChallenge {
        challenge_type: String,
        platform: crate::Platform,
    },

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

impl GhostError {
    /// Returns whether this error is retryable
    pub fn is_retryable(&self) -> bool {
        // TODO: Implement retryability determination logic
        matches!(
            self,
            GhostError::NetworkError(_)
                | GhostError::RateLimited { .. }
                | GhostError::Timeout(_)
                | GhostError::WafChallenge { .. }
        )
    }

    /// Returns whether this error indicates an account issue
    pub fn is_account_issue(&self) -> bool {
        // TODO: Implement account issue detection
        matches!(
            self,
            GhostError::AuthError(_)
                | GhostError::SessionExpired(_)
                | GhostError::AccountSuspended { .. }
        )
    }

    /// Returns whether this error indicates a proxy issue
    pub fn is_proxy_issue(&self) -> bool {
        // TODO: Implement proxy issue detection
        matches!(self, GhostError::ProxyError(_))
    }

    /// Returns the retry-after duration if available
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        // TODO: Implement retry-after extraction
        match self {
            GhostError::RateLimited { retry_after, .. } => {
                retry_after.map(|s| std::time::Duration::from_secs(s))
            }
            _ => None,
        }
    }

    /// Returns the platform if this error is platform-specific
    pub fn platform(&self) -> Option<crate::Platform> {
        // TODO: Implement platform extraction
        match self {
            GhostError::PlatformError { platform, .. } => Some(*platform),
            GhostError::RateLimited { platform, .. } => Some(*platform),
            GhostError::AccountSuspended { platform, .. } => Some(*platform),
            GhostError::WafChallenge { platform, .. } => Some(*platform),
            _ => None,
        }
    }

    /// Creates a trace string for debugging
    pub fn to_trace(&self) -> String {
        // TODO: Implement detailed error trace generation
        format!("{:#?}", self)
    }
}

/// Result type alias for GhostError
pub type GhostResult<T> = Result<T, GhostError>;
