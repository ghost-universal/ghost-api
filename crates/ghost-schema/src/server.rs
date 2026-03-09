//! Server types for Ghost API
//!
//! This module contains all types related to HTTP server responses,
//! API request/response DTOs, and server configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{GhostContext, GhostPost, Strategy};

// ============================================================================
// Server Configuration
// ============================================================================

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server port
    pub port: u16,
    /// Server host
    pub host: String,
    /// Whether to enable Swagger UI
    pub swagger_enabled: bool,
    /// Whether to enable CORS
    pub cors_enabled: bool,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum request body size
    pub max_body_size: usize,
    /// Log level
    pub log_level: String,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS key path
    pub tls_key_path: Option<String>,
}

impl ServerConfig {
    /// Creates a new server config
    pub fn new() -> Self {
        // TODO: Implement server config construction
        Self {
            port: 3000,
            host: "0.0.0.0".to_string(),
            swagger_enabled: true,
            cors_enabled: true,
            timeout_secs: 30,
            max_body_size: 1024 * 1024, // 1MB
            log_level: "info".to_string(),
            tls_cert_path: None,
            tls_key_path: None,
        }
    }

    /// Loads configuration from environment
    pub fn from_env() -> Self {
        // TODO: Implement environment loading
        Self::new()
    }

    /// Loads configuration from file
    pub fn from_file(path: &str) -> Result<Self, crate::GhostError> {
        // TODO: Implement file loading
        let _content = std::fs::read_to_string(path)
            .map_err(|e| crate::GhostError::ConfigError(format!("Failed to read config: {}", e)))?;
        Ok(Self::new())
    }

    /// Sets the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the host
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Enables TLS
    pub fn with_tls(mut self, cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        self.tls_cert_path = Some(cert_path.into());
        self.tls_key_path = Some(key_path.into());
        self
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Health Check Types
// ============================================================================

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Server status
    pub status: String,
    /// Version
    pub version: String,
    /// Worker count
    pub workers: usize,
    /// Healthy workers
    pub healthy_workers: usize,
    /// Uptime in seconds
    pub uptime_secs: u64,
}

impl HealthResponse {
    /// Creates a new health response
    pub fn new() -> Self {
        // TODO: Implement health response construction
        Self {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            workers: 0,
            healthy_workers: 0,
            uptime_secs: 0,
        }
    }

    /// Creates an unhealthy response
    pub fn unhealthy(reason: impl Into<String>) -> Self {
        // TODO: Implement unhealthy response construction
        Self {
            status: reason.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            workers: 0,
            healthy_workers: 0,
            uptime_secs: 0,
        }
    }
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Request Types
// ============================================================================

/// Query parameters for post requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostQuery {
    /// Routing strategy
    pub strategy: Option<String>,
    /// Tenant ID
    pub tenant_id: Option<String>,
    /// Proxy URL
    pub proxy: Option<String>,
}

impl PostQuery {
    /// Creates new post query
    pub fn new() -> Self {
        // TODO: Implement query construction
        Self {
            strategy: None,
            tenant_id: None,
            proxy: None,
        }
    }

    /// Parses the strategy
    pub fn parse_strategy(&self) -> Strategy {
        // TODO: Implement strategy parsing
        match self.strategy.as_deref() {
            Some("health_first") => Strategy::HealthFirst,
            Some("fastest") => Strategy::Fastest,
            Some("cost_optimized") => Strategy::CostOptimized,
            Some("official_first") => Strategy::OfficialFirst,
            Some("official_only") => Strategy::OfficialOnly,
            Some("scrapers_only") => Strategy::ScrapersOnly,
            Some("round_robin") => Strategy::RoundRobin,
            _ => Strategy::default(),
        }
    }
}

impl Default for PostQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,
    /// Routing strategy
    pub strategy: Option<String>,
    /// Maximum results
    pub limit: Option<usize>,
    /// Cursor for pagination
    pub cursor: Option<String>,
}

impl SearchQuery {
    /// Creates new search query
    pub fn new(query: impl Into<String>) -> Self {
        // TODO: Implement search query construction
        Self {
            q: query.into(),
            strategy: None,
            limit: None,
            cursor: None,
        }
    }

    /// Sets the limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the cursor
    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }
}

/// Headers for context injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionHeaders {
    /// Proxy URL
    pub proxy: Option<String>,
    /// Session cookies
    pub session: Option<String>,
    /// Tenant ID
    pub tenant_id: Option<String>,
    /// Bearer token
    pub bearer: Option<String>,
}

impl InjectionHeaders {
    /// Creates new injection headers
    pub fn new() -> Self {
        // TODO: Implement injection headers construction
        Self {
            proxy: None,
            session: None,
            tenant_id: None,
            bearer: None,
        }
    }

    /// Builds a GhostContext from these headers
    pub fn to_context(&self) -> GhostContext {
        // TODO: Implement context building
        let mut builder = GhostContext::builder();

        if let Some(ref tenant_id) = self.tenant_id {
            builder = builder.tenant_id(tenant_id);
        }

        if let Some(ref proxy) = self.proxy {
            builder = builder.proxy(proxy);
        }

        if let Some(ref session) = self.session {
            builder = builder.session(session);
        }

        builder.build()
    }
}

impl Default for InjectionHeaders {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Response Types
// ============================================================================

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<GhostPost>,
    /// Query string
    pub query: String,
    /// Total count
    pub total: usize,
    /// Next page cursor
    pub next_cursor: Option<String>,
}

impl SearchResponse {
    /// Creates a new search response
    pub fn new(query: impl Into<String>) -> Self {
        // TODO: Implement search response construction
        Self {
            results: Vec::new(),
            query: query.into(),
            total: 0,
            next_cursor: None,
        }
    }

    /// Creates a search response with results
    pub fn with_results(query: impl Into<String>, results: Vec<GhostPost>) -> Self {
        // TODO: Implement search response with results
        let total = results.len();
        Self {
            results,
            query: query.into(),
            total,
            next_cursor: None,
        }
    }

    /// Sets the cursor
    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.next_cursor = Some(cursor.into());
        self
    }
}

/// Timeline response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineResponse {
    /// Timeline posts
    pub posts: Vec<GhostPost>,
    /// Top cursor
    pub cursor_top: Option<String>,
    /// Bottom cursor
    pub cursor_bottom: Option<String>,
}

impl TimelineResponse {
    /// Creates a new timeline response
    pub fn new() -> Self {
        // TODO: Implement timeline response construction
        Self {
            posts: Vec::new(),
            cursor_top: None,
            cursor_bottom: None,
        }
    }

    /// Creates with posts
    pub fn with_posts(posts: Vec<GhostPost>) -> Self {
        Self {
            posts,
            cursor_top: None,
            cursor_bottom: None,
        }
    }
}

impl Default for TimelineResponse {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Worker Info Types
// ============================================================================

/// Worker information for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// Worker ID
    pub id: String,
    /// Worker type
    pub worker_type: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Health score
    pub health_score: f64,
    /// Status
    pub status: String,
}

impl WorkerInfo {
    /// Creates new worker info
    pub fn new(id: impl Into<String>) -> Self {
        // TODO: Implement worker info construction
        Self {
            id: id.into(),
            worker_type: "unknown".to_string(),
            capabilities: Vec::new(),
            health_score: 0.0,
            status: "unknown".to_string(),
        }
    }

    /// Sets the worker type
    pub fn with_type(mut self, worker_type: impl Into<String>) -> Self {
        self.worker_type = worker_type.into();
        self
    }

    /// Sets the capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }
}

/// Worker health info for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHealthInfo {
    /// Worker ID
    pub worker_id: String,
    /// Health score
    pub health_score: f64,
    /// Status
    pub status: String,
    /// Average latency
    pub avg_latency_ms: u64,
    /// Success rate
    pub success_rate: f64,
}

impl WorkerHealthInfo {
    /// Creates new worker health info
    pub fn new(worker_id: impl Into<String>) -> Self {
        // TODO: Implement worker health info construction
        Self {
            worker_id: worker_id.into(),
            health_score: 0.0,
            status: "unknown".to_string(),
            avg_latency_ms: 0,
            success_rate: 0.0,
        }
    }
}

// ============================================================================
// Error Response Types
// ============================================================================

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional details
    pub details: Option<HashMap<String, String>>,
}

impl ErrorResponse {
    /// Creates a new error response
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        // TODO: Implement error response construction
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Creates a bad request error
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message)
    }

    /// Creates a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", format!("{} not found", resource.into()))
    }

    /// Creates an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }

    /// Adds details to the error
    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = Some(details);
        self
    }
}

/// Not found response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotFoundResponse {
    /// Resource type
    pub resource: String,
    /// Resource ID
    pub id: String,
}

impl NotFoundResponse {
    /// Creates a new not found response
    pub fn new(resource: impl Into<String>, id: impl Into<String>) -> Self {
        // TODO: Implement not found response construction
        Self {
            resource: resource.into(),
            id: id.into(),
        }
    }
}
