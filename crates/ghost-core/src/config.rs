//! Configuration management for Ghost API
//!
//! Configuration types are imported from ghost-schema.
//! This module provides loading and validation utilities.

pub use ghost_schema::{
    GhostConfig, HealthConfig, ScraperConfig, PlatformShieldConfig,
    AutoscalingConfig, ScaleThreshold, ConfigBuilder,
};

use ghost_schema::GhostError;

impl GhostConfig {
    /// Loads configuration from the default path
    pub fn load() -> Result<Self, GhostError> {
        // TODO: Implement config loading from default path
        Self::from_file("config/ghost.toml")
    }

    /// Validates all nested configurations
    pub fn validate_all(&self) -> Result<(), GhostError> {
        // TODO: Implement comprehensive validation
        self.validate()?;
        self.health.validate()?;

        for (name, scraper) in &self.scrapers {
            scraper.validate()
                .map_err(|e| GhostError::ConfigError(
                    format!("Invalid scraper config '{}': {}", name, e)
                ))?;
        }

        for (platform, shield) in &self.shields {
            shield.validate()
                .map_err(|e| GhostError::ConfigError(
                    format!("Invalid shield config for {:?}: {}", platform, e)
                ))?;
        }

        self.autoscaling.validate()?;

        Ok(())
    }

    /// Returns the effective health configuration
    pub fn effective_health_config(&self) -> &HealthConfig {
        // TODO: Implement effective config resolution
        &self.health
    }

    /// Returns the effective strategy
    pub fn effective_strategy(&self, override_strategy: Option<ghost_schema::Strategy>) -> ghost_schema::Strategy {
        // TODO: Implement strategy override logic
        override_strategy.unwrap_or(self.default_strategy)
    }

    /// Checks if a platform is enabled
    pub fn is_platform_enabled(&self, platform: ghost_schema::Platform) -> bool {
        // TODO: Implement platform enablement check
        self.shields.contains_key(&platform)
    }

    /// Returns the effective timeout in milliseconds
    pub fn effective_timeout_ms(&self) -> u64 {
        // TODO: Implement timeout calculation
        self.request_timeout_secs * 1000
    }
}
