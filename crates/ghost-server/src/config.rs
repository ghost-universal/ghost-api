//! Server configuration

use std::net::SocketAddr;

use ghost_schema::GhostError;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server address
    pub addr: SocketAddr,
    /// Whether to enable CORS
    pub cors_enabled: bool,
    /// Whether to enable Swagger UI
    pub swagger_enabled: bool,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Maximum request body size
    pub max_body_size: usize,
    /// Log level
    pub log_level: String,
}

impl ServerConfig {
    /// Creates a new server config
    pub fn new() -> Self {
        // TODO: Implement config construction
        Self {
            addr: SocketAddr::from(([0, 0, 0, 0], 3000)),
            cors_enabled: true,
            swagger_enabled: true,
            request_timeout_secs: 30,
            max_body_size: 1024 * 1024, // 1MB
            log_level: "info".to_string(),
        }
    }

    /// Loads configuration from environment
    pub fn from_env() -> Self {
        // TODO: Implement environment config loading
        let mut config = Self::new();

        if let Ok(addr) = std::env::var("GHOST_ADDR") {
            if let Ok(parsed) = addr.parse() {
                config.addr = parsed;
            }
        }

        if let Ok(level) = std::env::var("RUST_LOG") {
            config.log_level = level;
        }

        config
    }

    /// Loads configuration from file
    pub fn from_file(path: &str) -> Result<Self, GhostError> {
        // TODO: Implement file config loading
        let _content = std::fs::read_to_string(path)
            .map_err(|e| GhostError::ConfigError(format!("Failed to read config: {}", e)))?;

        Ok(Self::new())
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), GhostError> {
        // TODO: Implement config validation
        if self.request_timeout_secs == 0 {
            return Err(GhostError::ConfigError("request_timeout_secs must be > 0".into()));
        }

        if self.max_body_size == 0 {
            return Err(GhostError::ConfigError("max_body_size must be > 0".into()));
        }

        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Ghost engine configuration for the server
#[derive(Debug, Clone)]
pub struct GhostEngineConfig {
    /// Path to ghost.toml
    pub config_path: Option<String>,
    /// Default strategy
    pub default_strategy: String,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Enable auto-discovery of scrapers
    pub auto_discovery: bool,
    /// Scrapers directory
    pub scrapers_dir: String,
}

impl GhostEngineConfig {
    /// Creates a new engine config
    pub fn new() -> Self {
        // TODO: Implement engine config construction
        Self {
            config_path: None,
            default_strategy: "health_first".to_string(),
            health_check_interval_secs: 30,
            auto_discovery: true,
            scrapers_dir: "./scrapers".to_string(),
        }
    }

    /// Loads from environment
    pub fn from_env() -> Self {
        // TODO: Implement environment loading
        let mut config = Self::new();

        if let Ok(path) = std::env::var("GHOST_CONFIG") {
            config.config_path = Some(path);
        }

        if let Ok(strategy) = std::env::var("GHOST_STRATEGY") {
            config.default_strategy = strategy;
        }

        config
    }
}

impl Default for GhostEngineConfig {
    fn default() -> Self {
        Self::new()
    }
}
