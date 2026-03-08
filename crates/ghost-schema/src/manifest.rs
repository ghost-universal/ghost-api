//! Polyglot Manifest Types
//!
//! Defines the schema for polyglot worker manifests, which declare
//! capabilities, dependencies, and configuration for external scrapers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Polyglot manifest structure
///
/// Each polyglot worker must provide a manifest.json declaring its
/// capabilities, dependencies, and configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyglotManifest {
    /// JSON schema URL (optional)
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    
    /// Unique identifier for this worker
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Semantic version
    pub version: String,
    
    /// Description of what this worker does
    pub description: String,
    
    /// Runtime configuration
    pub runtime: RuntimeConfig,
    
    /// External source configuration (for git submodules)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<ExternalSource>,
    
    /// Capabilities provided by this worker
    pub capabilities: Vec<CapabilityDefinition>,
    
    /// Supported platforms
    pub platforms: Vec<String>,
    
    /// Python/Node.js/Go dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    
    /// Configuration schema with defaults
    #[serde(default)]
    pub configuration: HashMap<String, ConfigField>,
    
    /// Health check configuration
    #[serde(default)]
    pub health: HealthConfig,
    
    /// Rate limit configuration
    #[serde(default)]
    pub rate_limits: RateLimitConfig,
    
    /// Output format specification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<OutputSpec>,
    
    /// FFI entry points
    #[serde(default)]
    pub ffi: FfiConfig,
}

impl PolyglotManifest {
    /// Load manifest from a file
    pub fn from_file(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ManifestError::FileNotFound(format!("{}: {}", path.display(), e)))?;
        
        let manifest: Self = serde_json::from_str(&content)
            .map_err(|e| ManifestError::InvalidJson(e.to_string()))?;
        
        manifest.validate()?;
        Ok(manifest)
    }
    
    /// Load manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self, ManifestError> {
        let manifest: Self = serde_json::from_str(json)
            .map_err(|e| ManifestError::InvalidJson(e.to_string()))?;
        
        manifest.validate()?;
        Ok(manifest)
    }
    
    /// Validate manifest structure
    pub fn validate(&self) -> Result<(), ManifestError> {
        // Check required fields
        if self.id.is_empty() {
            return Err(ManifestError::ValidationError("id is required".into()));
        }
        
        if self.name.is_empty() {
            return Err(ManifestError::ValidationError("name is required".into()));
        }
        
        if self.capabilities.is_empty() {
            return Err(ManifestError::ValidationError(
                "at least one capability is required".into()
            ));
        }
        
        // Validate capabilities
        for cap in &self.capabilities {
            if cap.name.is_empty() {
                return Err(ManifestError::ValidationError(
                    "capability name is required".into()
                ));
            }
        }
        
        // Validate platforms
        if self.platforms.is_empty() {
            return Err(ManifestError::ValidationError(
                "at least one platform is required".into()
            ));
        }
        
        Ok(())
    }
    
    /// Get capability by name
    pub fn get_capability(&self, name: &str) -> Option<&CapabilityDefinition> {
        self.capabilities.iter().find(|c| c.name == name)
    }
    
    /// Check if worker supports a capability
    pub fn has_capability(&self, name: &str) -> bool {
        self.capabilities.iter().any(|c| c.name == name)
    }
    
    /// Check if worker supports a platform
    pub fn supports_platform(&self, platform: &str) -> bool {
        self.platforms.iter().any(|p| p == platform)
    }
    
    /// Get FFI entry point function name
    pub fn get_ffi_entry(&self, entry_type: &str) -> Option<&str> {
        self.ffi.entry_points.get(entry_type).map(|s| s.as_str())
    }
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Runtime type: python, nodejs, go
    #[serde(rename = "type")]
    pub runtime_type: String,
    
    /// Version constraint (e.g., ">=3.11")
    pub version: String,
    
    /// Entrypoint (module:class or file:function)
    pub entrypoint: String,
    
    /// Module name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}

impl RuntimeConfig {
    /// Check if this is a Python runtime
    pub fn is_python(&self) -> bool {
        self.runtime_type == "python"
    }
    
    /// Check if this is a Node.js runtime
    pub fn is_nodejs(&self) -> bool {
        self.runtime_type == "nodejs" || self.runtime_type == "node"
    }
    
    /// Check if this is a Go runtime
    pub fn is_go(&self) -> bool {
        self.runtime_type == "go"
    }
}

/// External source configuration (git submodule)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSource {
    /// Source type: git_submodule, local, pip, npm
    #[serde(rename = "type")]
    pub source_type: String,
    
    /// Path to external source relative to project root
    pub path: String,
    
    /// Upstream repository URL
    pub upstream: String,
    
    /// License
    pub license: String,
    
    /// Attribution (original author)
    pub attribution: String,
    
    /// Files included from external source
    #[serde(default)]
    pub files: Vec<String>,
}

impl ExternalSource {
    /// Check if this is a git submodule
    pub fn is_git_submodule(&self) -> bool {
        self.source_type == "git_submodule"
    }
}

/// Capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDefinition {
    /// Capability name (e.g., "threads_read", "x_search")
    pub name: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Tier (1 = fast, 2 = heavy, 3 = fallback)
    #[serde(default = "default_tier")]
    pub tier: u8,
    
    /// Health weight for this capability (0.0 - 1.0)
    #[serde(default = "default_health_weight")]
    pub health_weight: f64,
    
    /// Parameter schema
    #[serde(default)]
    pub parameters: HashMap<String, ParameterDef>,
}

fn default_tier() -> u8 { 1 }
fn default_health_weight() -> f64 { 1.0 }

impl CapabilityDefinition {
    /// Get a parameter by name
    pub fn get_parameter(&self, name: &str) -> Option<&ParameterDef> {
        self.parameters.get(name)
    }
    
    /// Check if a parameter is required
    pub fn is_parameter_required(&self, name: &str) -> bool {
        self.parameters.get(name).map(|p| p.required).unwrap_or(false)
    }
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    /// Parameter type: string, integer, boolean, array, object
    #[serde(rename = "type")]
    pub param_type: String,
    
    /// Whether this parameter is required
    #[serde(default)]
    pub required: bool,
    
    /// Default value if not provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    
    /// Human-readable description
    #[serde(default)]
    pub description: String,
}

/// Configuration field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    /// Field type
    #[serde(rename = "type")]
    pub field_type: String,
    
    /// Default value
    pub default: serde_json::Value,
    
    /// Description
    #[serde(default)]
    pub description: String,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Interval between health checks in milliseconds
    #[serde(default = "default_check_interval")]
    pub check_interval_ms: u64,
    
    /// Threshold for healthy status (0.0 - 1.0)
    #[serde(default = "default_healthy_threshold")]
    pub healthy_threshold: f64,
    
    /// Threshold for degraded status (0.0 - 1.0)
    #[serde(default = "default_degraded_threshold")]
    pub degraded_threshold: f64,
    
    /// Health metric weights
    #[serde(default = "default_health_metrics")]
    pub metrics: HealthMetrics,
}

fn default_check_interval() -> u64 { 300_000 }  // 5 minutes
fn default_healthy_threshold() -> f64 { 0.8 }
fn default_degraded_threshold() -> f64 { 0.5 }
fn default_health_metrics() -> HealthMetrics {
    HealthMetrics {
        success_weight: 0.60,
        latency_weight: 0.25,
        quality_weight: 0.15,
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: default_check_interval(),
            healthy_threshold: default_healthy_threshold(),
            degraded_threshold: default_degraded_threshold(),
            metrics: default_health_metrics(),
        }
    }
}

/// Health metric weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Weight for success rate in health score
    pub success_weight: f64,
    
    /// Weight for latency in health score
    pub latency_weight: f64,
    
    /// Weight for data quality in health score
    pub quality_weight: f64,
}

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    #[serde(default = "default_rpm")]
    pub requests_per_minute: u32,
    
    /// Whether to back off on 429 responses
    #[serde(default = "default_backoff")]
    pub backoff_on_429: bool,
    
    /// Random jitter range in milliseconds [min, max]
    #[serde(default = "default_jitter")]
    pub jitter_range_ms: [u32; 2],
}

fn default_rpm() -> u32 { 10 }
fn default_backoff() -> bool { true }
fn default_jitter() -> [u32; 2] { [500, 3000] }

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: default_rpm(),
            backoff_on_429: default_backoff(),
            jitter_range_ms: default_jitter(),
        }
    }
}

/// Output format specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    /// Output format: json, html, raw
    pub format: String,
    
    /// JSON schema for output validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

/// FFI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiConfig {
    /// Entry point function names
    #[serde(default = "default_entry_points")]
    pub entry_points: HashMap<String, String>,
}

fn default_entry_points() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("execute".into(), "execute_worker".into());
    map.insert("info".into(), "get_worker_info".into());
    map.insert("health".into(), "health_check".into());
    map
}

impl Default for FfiConfig {
    fn default() -> Self {
        Self {
            entry_points: default_entry_points(),
        }
    }
}

/// Manifest error types
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manifest_deserialization() {
        let json = r#"
        {
            "id": "test-worker",
            "name": "Test Worker",
            "version": "1.0.0",
            "description": "A test worker",
            "runtime": {
                "type": "python",
                "version": ">=3.11",
                "entrypoint": "test:Worker"
            },
            "capabilities": [
                {
                    "name": "test_read",
                    "description": "Read test data",
                    "tier": 1
                }
            ],
            "platforms": ["test"]
        }
        "#;
        
        let manifest = PolyglotManifest::from_json(json).unwrap();
        assert_eq!(manifest.id, "test-worker");
        assert!(manifest.has_capability("test_read"));
    }
    
    #[test]
    fn test_manifest_validation() {
        let json = r#"
        {
            "id": "",
            "name": "Test",
            "version": "1.0.0",
            "description": "",
            "runtime": {
                "type": "python",
                "version": ">=3.11",
                "entrypoint": "test:Worker"
            },
            "capabilities": [],
            "platforms": []
        }
        "#;
        
        let result = PolyglotManifest::from_json(json);
        assert!(result.is_err());
    }
}
