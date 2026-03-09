//! Configuration management for Ghost API
//!
//! Configuration types are imported from ghost-schema.
//! This module provides loading and validation utilities.

pub use ghost_schema::{
    GhostConfig, HealthConfig, ScraperConfig, PlatformShieldConfig,
    AutoscalingConfig, ConfigBuilder,
};

use ghost_schema::GhostError;

/// Configuration loader for GhostConfig
pub struct ConfigLoader;

impl ConfigLoader {
    /// Loads configuration from the default path
    ///
    /// Default path is `config/ghost.toml` relative to the current directory.
    pub fn load() -> Result<GhostConfig, GhostError> {
        Self::from_file("config/ghost.toml")
    }

    /// Loads configuration from a file path
    ///
    /// Parses the file as TOML and constructs the configuration.
    pub fn from_file(path: &str) -> Result<GhostConfig, GhostError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| GhostError::ConfigError(format!("Failed to read config file '{}': {}", path, e)))?;

        Self::from_toml(&content)
    }

    /// Parses configuration from a TOML string
    pub fn from_toml(toml_str: &str) -> Result<GhostConfig, GhostError> {
        // Use serde to parse TOML directly into GhostConfig
        let config: GhostConfig = toml::from_str(toml_str)
            .map_err(|e| GhostError::ConfigError(format!("Failed to parse TOML: {}", e)))?;

        Ok(config)
    }

    /// Validates all nested configurations
    pub fn validate_all(config: &GhostConfig) -> Result<(), GhostError> {
        config.validate()?;
        config.health.validate()?;

        for (name, scraper) in &config.scrapers {
            scraper.validate()
                .map_err(|e| GhostError::ConfigError(
                    format!("Invalid scraper config '{}': {}", name, e)
                ))?;
        }

        for (platform, shield) in &config.shields {
            shield.validate()
                .map_err(|e| GhostError::ConfigError(
                    format!("Invalid shield config for {:?}: {}", platform, e)
                ))?;
        }

        config.autoscaling.validate()?;

        Ok(())
    }

    /// Returns the effective health configuration
    pub fn effective_health_config(config: &GhostConfig) -> &HealthConfig {
        &config.health
    }

    /// Returns the effective strategy
    ///
    /// If an override is provided, returns that; otherwise returns the default.
    pub fn effective_strategy(config: &GhostConfig, override_strategy: Option<ghost_schema::Strategy>) -> ghost_schema::Strategy {
        override_strategy.unwrap_or(config.default_strategy)
    }

    /// Checks if a platform is enabled
    pub fn is_platform_enabled(config: &GhostConfig, platform: ghost_schema::Platform) -> bool {
        config.shields.contains_key(&platform)
    }

    /// Returns the effective timeout in milliseconds
    pub fn effective_timeout_ms(config: &GhostConfig) -> u64 {
        config.request_timeout_secs * 1000
    }
}

/// Extension trait for GhostConfig validation
pub trait GhostConfigExt {
    /// Validates all nested configurations
    fn validate_all(&self) -> Result<(), GhostError>;

    /// Returns the effective strategy
    fn effective_strategy(&self, override_strategy: Option<ghost_schema::Strategy>) -> ghost_schema::Strategy;

    /// Checks if a platform is enabled
    fn is_platform_enabled(&self, platform: ghost_schema::Platform) -> bool;

    /// Returns the effective timeout in milliseconds
    fn effective_timeout_ms(&self) -> u64;
}

impl GhostConfigExt for GhostConfig {
    fn validate_all(&self) -> Result<(), GhostError> {
        ConfigLoader::validate_all(self)
    }

    fn effective_strategy(&self, override_strategy: Option<ghost_schema::Strategy>) -> ghost_schema::Strategy {
        ConfigLoader::effective_strategy(self, override_strategy)
    }

    fn is_platform_enabled(&self, platform: ghost_schema::Platform) -> bool {
        ConfigLoader::is_platform_enabled(self, platform)
    }

    fn effective_timeout_ms(&self) -> u64 {
        ConfigLoader::effective_timeout_ms(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = GhostConfig::new();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.default_strategy, ghost_schema::Strategy::HealthFirst);
    }

    #[test]
    fn test_config_validate() {
        let config = GhostConfig::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_all() {
        let config = GhostConfig::new();
        assert!(config.validate_all().is_ok());
    }

    #[test]
    fn test_config_effective_strategy() {
        let config = GhostConfig::new();

        assert_eq!(config.effective_strategy(None), ghost_schema::Strategy::HealthFirst);
        assert_eq!(
            config.effective_strategy(Some(ghost_schema::Strategy::Fastest)),
            ghost_schema::Strategy::Fastest
        );
    }

    #[test]
    fn test_config_platform_enabled() {
        let config = GhostConfig::new();
        assert!(!config.is_platform_enabled(ghost_schema::Platform::X));
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .strategy(ghost_schema::Strategy::Fastest)
            .max_retries(5)
            .timeout(60)
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.default_strategy, ghost_schema::Strategy::Fastest);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.request_timeout_secs, 60);
    }

    #[test]
    fn test_health_config_validate() {
        let config = HealthConfig::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_health_config_invalid_threshold() {
        let mut config = HealthConfig::new();
        config.healthy_threshold = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_scraper_config_new() {
        let config = ScraperConfig::new();
        assert!(config.enabled);
        assert_eq!(config.health_threshold, 0.7);
    }

    #[test]
    fn test_autoscaling_config_validate() {
        let config = AutoscalingConfig::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_autoscaling_config_invalid_range() {
        let mut config = AutoscalingConfig::new();
        config.min_workers = 10;
        config.max_workers = 5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_from_toml() {
        // Test with minimal TOML that matches the expected structure
        let toml = r#"
        default_strategy = "health_first"
        max_retries = 5
        request_timeout_secs = 30

        [health]
        healthy_threshold = 0.7
        degraded_threshold = 0.5
        max_latency_ms = 2000
        consecutive_failure_threshold = 5
        circuit_breaker_timeout_secs = 60
        check_interval_secs = 30
        stats_window_size = 100

        [autoscaling]
        enabled = false
        min_workers = 2
        max_workers = 20
        prefer_spot = true
        spot_fallback_on_demand = true

        [autoscaling.scale_up_threshold]
        type = "queue_depth"
        depth = 50
        duration_secs = 30

        [autoscaling.scale_down_threshold]
        type = "utilization"
        threshold = 0.3
        duration_secs = 300

        # Empty tables for required maps
        [scrapers]

        [shields]
        "#;

        let config = ConfigLoader::from_toml(toml);
        if config.is_err() {
            eprintln!("TOML parse error: {:?}", config.as_ref().err());
        }
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.default_strategy, ghost_schema::Strategy::HealthFirst);
        assert_eq!(config.max_retries, 5);
    }
}
