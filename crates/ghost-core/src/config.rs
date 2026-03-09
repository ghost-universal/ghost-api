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
    ///
    /// Default path is `config/ghost.toml` relative to the current directory.
    pub fn load() -> Result<Self, GhostError> {
        Self::from_file("config/ghost.toml")
    }

    /// Loads configuration from a file path
    ///
    /// Parses the file as TOML and constructs the configuration.
    pub fn from_file(path: &str) -> Result<Self, GhostError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| GhostError::ConfigError(format!("Failed to read config file '{}': {}", path, e)))?;

        Self::from_toml(&content)
    }

    /// Parses configuration from a TOML string
    pub fn from_toml(toml: &str) -> Result<Self, GhostError> {
        // Parse TOML
        let value: toml::Value = toml::parse(toml)
            .map_err(|e| GhostError::ConfigError(format!("Failed to parse TOML: {}", e)))?;

        Self::parse_from_toml_value(&value)
    }

    /// Parses from a toml::Value
    fn parse_from_toml_value(value: &toml::Value) -> Result<Self, GhostError> {
        let mut config = Self::new();

        if let Some(engine) = value.get("engine") {
            if let Some(strategy) = engine.get("default_strategy").and_then(|v| v.as_str()) {
                config.default_strategy = Self::parse_strategy(strategy)?;
            }
            if let Some(retries) = engine.get("retry_count").and_then(|v| v.as_integer()) {
                config.max_retries = retries as u32;
            }
        }

        // Parse scrapers
        if let Some(scrapers) = value.get("scrapers").and_then(|v| v.as_table()) {
            for (name, scraper_config) in scrapers {
                if let Some(scraper) = Self::parse_scraper_config(scraper_config) {
                    config.scrapers.insert(name.clone(), scraper);
                }
            }
        }

        // Parse shields
        if let Some(shields) = value.get("shield").and_then(|v| v.as_table()) {
            for (platform_name, shield_config) in shields {
                let platform = ghost_schema::Platform::from_str(platform_name);
                if platform != ghost_schema::Platform::Unknown {
                    if let Some(shield) = Self::parse_shield_config(shield_config, platform) {
                        config.shields.insert(platform, shield);
                    }
                }
            }
        }

        // Parse autoscaling
        if let Some(autoscaling) = value.get("autoscaling") {
            config.autoscaling = Self::parse_autoscaling_config(autoscaling);
        }

        Ok(config)
    }

    /// Parses a strategy string
    fn parse_strategy(s: &str) -> Result<ghost_schema::Strategy, GhostError> {
        match s.to_lowercase().as_str() {
            "health_first" => Ok(ghost_schema::Strategy::HealthFirst),
            "fastest" => Ok(ghost_schema::Strategy::Fastest),
            "cost_optimized" => Ok(ghost_schema::Strategy::CostOptimized),
            "official_first" => Ok(ghost_schema::Strategy::OfficialFirst),
            "official_only" => Ok(ghost_schema::Strategy::OfficialOnly),
            "scrapers_only" => Ok(ghost_schema::Strategy::ScrapersOnly),
            "round_robin" => Ok(ghost_schema::Strategy::RoundRobin),
            _ => Err(GhostError::ConfigError(format!("Unknown strategy: {}", s))),
        }
    }

    /// Parses a scraper config from TOML
    fn parse_scraper_config(value: &toml::Value) -> Option<ScraperConfig> {
        let table = value.as_table()?;
        let mut config = ScraperConfig::new();

        if let Some(enabled) = table.get("enabled").and_then(|v| v.as_bool()) {
            config.enabled = enabled;
        }
        if let Some(capabilities) = table.get("capabilities").and_then(|v| v.as_array()) {
            config.capabilities = capabilities
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
        if let Some(threshold) = table.get("health_threshold").and_then(|v| v.as_float()) {
            config.health_threshold = threshold;
        }
        if let Some(max_concurrent) = table.get("max_concurrent").and_then(|v| v.as_integer()) {
            config.max_concurrent = max_concurrent as u32;
        }
        if let Some(priority) = table.get("priority").and_then(|v| v.as_integer()) {
            config.priority = priority as u32;
        }
        if let Some(api_key) = table.get("api_key").and_then(|v| v.as_str()) {
            config.api_key = Some(api_key.to_string());
        }
        if let Some(endpoint) = table.get("endpoint").and_then(|v| v.as_str()) {
            config.endpoint = Some(endpoint.to_string());
        }

        Some(config)
    }

    /// Parses a shield config from TOML
    fn parse_shield_config(value: &toml::Value, platform: ghost_schema::Platform) -> Option<PlatformShieldConfig> {
        let table = value.as_table()?;
        let mut config = PlatformShieldConfig::new(platform);

        if let Some(fingerprint) = table.get("fingerprint").and_then(|v| v.as_str()) {
            config.fingerprint = fingerprint.to_string();
        }
        if let Some(header_profile) = table.get("header_profile").and_then(|v| v.as_str()) {
            config.header_profile = header_profile.to_string();
        }
        if let Some(jitter) = table.get("jitter_range_ms").and_then(|v| v.as_array()) {
            if jitter.len() >= 2 {
                let min = jitter[0].as_integer().unwrap_or(500) as u64;
                let max = jitter[1].as_integer().unwrap_or(3000) as u64;
                config.jitter_range_ms = (min, max);
            }
        }
        if let Some(respect) = table.get("respect_retry_after").and_then(|v| v.as_bool()) {
            config.respect_retry_after = respect;
        }

        Some(config)
    }

    /// Parses autoscaling config from TOML
    fn parse_autoscaling_config(value: &toml::Value) -> AutoscalingConfig {
        let table = match value.as_table() {
            Some(t) => t,
            None => return AutoscalingConfig::default(),
        };

        let mut config = AutoscalingConfig::new();

        if let Some(enabled) = table.get("enabled").and_then(|v| v.as_bool()) {
            config.enabled = enabled;
        }
        if let Some(min) = table.get("min_workers").and_then(|v| v.as_integer()) {
            config.min_workers = min as u32;
        }
        if let Some(max) = table.get("max_workers").and_then(|v| v.as_integer()) {
            config.max_workers = max as u32;
        }
        if let Some(spot) = table.get("prefer_spot").and_then(|v| v.as_bool()) {
            config.prefer_spot = spot;
        }
        if let Some(fallback) = table.get("spot_fallback_on_demand").and_then(|v| v.as_bool()) {
            config.spot_fallback_on_demand = fallback;
        }

        config
    }

    /// Validates all nested configurations
    pub fn validate_all(&self) -> Result<(), GhostError> {
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
        &self.health
    }

    /// Returns the effective strategy
    ///
    /// If an override is provided, returns that; otherwise returns the default.
    pub fn effective_strategy(&self, override_strategy: Option<ghost_schema::Strategy>) -> ghost_schema::Strategy {
        override_strategy.unwrap_or(self.default_strategy)
    }

    /// Checks if a platform is enabled
    pub fn is_platform_enabled(&self, platform: ghost_schema::Platform) -> bool {
        self.shields.contains_key(&platform)
    }

    /// Returns the effective timeout in milliseconds
    pub fn effective_timeout_ms(&self) -> u64 {
        self.request_timeout_secs * 1000
    }

    /// Exports configuration to TOML string
    pub fn to_toml_string(&self) -> Result<String, GhostError> {
        let mut output = String::new();

        // Engine section
        output.push_str("[engine]\n");
        output.push_str(&format!("default_strategy = \"{:?}\"\n", self.default_strategy).to_lowercase());
        output.push_str(&format!("retry_count = {}\n", self.max_retries));
        output.push_str("\n");

        // Health section
        output.push_str("[health]\n");
        output.push_str(&format!("healthy_threshold = {}\n", self.health.healthy_threshold));
        output.push_str(&format!("degraded_threshold = {}\n", self.health.degraded_threshold));
        output.push_str(&format!("max_latency_ms = {}\n", self.health.max_latency_ms));
        output.push_str("\n");

        // Scrapers
        for (name, scraper) in &self.scrapers {
            output.push_str(&format!("[scrapers.{}]\n", name));
            output.push_str(&format!("enabled = {}\n", scraper.enabled));
            if !scraper.capabilities.is_empty() {
                output.push_str(&format!("capabilities = {:?}\n", scraper.capabilities));
            }
            output.push_str(&format!("health_threshold = {}\n", scraper.health_threshold));
            output.push_str(&format!("max_concurrent = {}\n", scraper.max_concurrent));
            output.push_str(&format!("priority = {}\n", scraper.priority));
            output.push_str("\n");
        }

        // Shields
        for (platform, shield) in &self.shields {
            output.push_str(&format!("[shield.{:?}]\n", platform).to_lowercase());
            output.push_str(&format!("fingerprint = \"{}\"\n", shield.fingerprint));
            output.push_str(&format!("header_profile = \"{}\"\n", shield.header_profile));
            output.push_str(&format!("jitter_range_ms = [{}, {}]\n", shield.jitter_range_ms.0, shield.jitter_range_ms.1));
            output.push_str(&format!("respect_retry_after = {}\n", shield.respect_retry_after));
            output.push_str("\n");
        }

        // Autoscaling
        output.push_str("[autoscaling]\n");
        output.push_str(&format!("enabled = {}\n", self.autoscaling.enabled));
        output.push_str(&format!("min_workers = {}\n", self.autoscaling.min_workers));
        output.push_str(&format!("max_workers = {}\n", self.autoscaling.max_workers));
        output.push_str(&format!("prefer_spot = {}\n", self.autoscaling.prefer_spot));

        Ok(output)
    }

    /// Creates a builder for configuration
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
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
        let toml = r#"
[engine]
default_strategy = "fastest"
retry_count = 5

[scrapers.test-scraper]
enabled = true
capabilities = ["X_READ", "X_SEARCH"]
health_threshold = 0.8
"#;

        let config = GhostConfig::from_toml(toml);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.default_strategy, ghost_schema::Strategy::Fastest);
        assert_eq!(config.max_retries, 5);
        assert!(config.scrapers.contains_key("test-scraper"));
    }
}
