//! Capability definitions for workers and adapters
//!
//! This module defines capabilities that workers can provide and
//! the tier system for capability classification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{GhostError, Platform};

/// Defines what a worker can do
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Capability {
    // X (Twitter) capabilities
    /// Read posts/tweets
    XRead,
    /// Search content on X
    XSearch,
    /// Read user profiles on X
    XUserRead,
    /// Read trending topics on X
    XTrending,
    /// Read X timeline
    XTimeline,
    /// Post tweets (if supported)
    XWrite,

    // Threads capabilities
    /// Read threads posts
    ThreadsRead,
    /// Search threads content
    ThreadsSearch,
    /// Read user profiles on Threads
    ThreadsUserRead,
    /// Read Threads timeline
    ThreadsTimeline,
    /// Post on Threads (if supported)
    ThreadsWrite,

    // General capabilities
    /// Supports media downloads
    MediaDownload,
    /// Supports high-volume batching
    BatchProcessing,
    /// Supports session refresh
    SessionRefresh,
    /// Official API access
    OfficialApi,
    /// Supports proxy rotation
    ProxyRotation,
    /// Browser-based scraping (stealth)
    BrowserBased,
    /// Headless request-based scraping (fast)
    RequestBased,
}

impl Capability {
    /// Returns all capabilities for a given platform
    pub fn for_platform(platform: Platform) -> Vec<Capability> {
        // TODO: Implement platform capability mapping
        match platform {
            Platform::X => vec![
                Capability::XRead,
                Capability::XSearch,
                Capability::XUserRead,
                Capability::XTrending,
                Capability::XTimeline,
                Capability::XWrite,
            ],
            Platform::Threads => vec![
                Capability::ThreadsRead,
                Capability::ThreadsSearch,
                Capability::ThreadsUserRead,
                Capability::ThreadsTimeline,
                Capability::ThreadsWrite,
            ],
            Platform::Unknown => vec![],
        }
    }

    /// Returns whether this capability requires authentication
    pub fn requires_auth(&self) -> bool {
        // TODO: Implement auth requirement determination
        matches!(
            self,
            Capability::XTimeline
                | Capability::XWrite
                | Capability::ThreadsTimeline
                | Capability::ThreadsWrite
                | Capability::SessionRefresh
        )
    }

    /// Returns the tier level for this capability
    pub fn tier(&self) -> CapabilityTier {
        // TODO: Implement capability tier determination
        match self {
            Capability::OfficialApi => CapabilityTier::Official,
            Capability::BrowserBased => CapabilityTier::Heavy,
            Capability::RequestBased => CapabilityTier::Fast,
            _ => CapabilityTier::Fast,
        }
    }

    /// Returns the estimated cost multiplier for this capability
    pub fn cost_multiplier(&self) -> f64 {
        // TODO: Implement cost estimation
        match self {
            Capability::OfficialApi => 1.0,
            Capability::BrowserBased => 0.5,
            Capability::RequestBased => 0.1,
            Capability::BatchProcessing => 0.05,
            _ => 0.1,
        }
    }

    /// Returns the platform for this capability
    pub fn platform(&self) -> Option<Platform> {
        // TODO: Implement platform extraction
        match self {
            Capability::XRead
            | Capability::XSearch
            | Capability::XUserRead
            | Capability::XTrending
            | Capability::XTimeline
            | Capability::XWrite => Some(Platform::X),
            Capability::ThreadsRead
            | Capability::ThreadsSearch
            | Capability::ThreadsUserRead
            | Capability::ThreadsTimeline
            | Capability::ThreadsWrite => Some(Platform::Threads),
            _ => None,
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Tier level for capabilities (affects routing and fallback)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityTier {
    /// Fast, lightweight scrapers (HTTP clients)
    Fast,
    /// Heavy, browser-based scrapers (Playwright, Puppeteer)
    Heavy,
    /// Official API (highest tier, used as last resort)
    Official,
}

impl CapabilityTier {
    /// Returns the fallback tier if this tier fails
    pub fn fallback(&self) -> Option<CapabilityTier> {
        // TODO: Implement tier fallback logic
        match self {
            CapabilityTier::Fast => Some(CapabilityTier::Heavy),
            CapabilityTier::Heavy => Some(CapabilityTier::Official),
            CapabilityTier::Official => None,
        }
    }

    /// Returns the estimated latency for this tier
    pub fn estimated_latency_ms(&self) -> u64 {
        // TODO: Implement latency estimation
        match self {
            CapabilityTier::Fast => 500,
            CapabilityTier::Heavy => 3000,
            CapabilityTier::Official => 200,
        }
    }

    /// Returns the cost multiplier for this tier
    pub fn cost_multiplier(&self) -> f64 {
        // TODO: Implement cost multiplier
        match self {
            CapabilityTier::Fast => 0.1,
            CapabilityTier::Heavy => 0.5,
            CapabilityTier::Official => 1.0,
        }
    }
}

/// Capability manifest for a worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityManifest {
    /// Worker identifier
    pub worker_id: String,
    /// Worker version
    pub version: String,
    /// List of supported capabilities
    pub capabilities: Vec<Capability>,
    /// Worker type
    pub worker_type: WorkerType,
    /// Maximum concurrent requests
    pub max_concurrent: u32,
    /// Health threshold (0.0 - 1.0)
    pub health_threshold: f64,
    /// Priority weight (higher = preferred)
    pub priority: u32,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl CapabilityManifest {
    /// Creates a new capability manifest
    pub fn new(worker_id: impl Into<String>, capabilities: Vec<Capability>) -> Self {
        // TODO: Implement capability manifest construction
        Self {
            worker_id: worker_id.into(),
            version: "0.1.0".to_string(),
            capabilities,
            worker_type: WorkerType::Unknown,
            max_concurrent: 5,
            health_threshold: 0.7,
            priority: 50,
            tags: Vec::new(),
        }
    }

    /// Checks if this manifest supports a given capability
    pub fn supports(&self, capability: Capability) -> bool {
        // TODO: Implement capability support check
        self.capabilities.contains(&capability)
    }

    /// Validates the manifest
    pub fn validate(&self) -> Result<(), GhostError> {
        // TODO: Implement manifest validation
        if self.worker_id.is_empty() {
            return Err(GhostError::ValidationError(
                "worker_id cannot be empty".into(),
            ));
        }
        if self.health_threshold < 0.0 || self.health_threshold > 1.0 {
            return Err(GhostError::ValidationError(
                "health_threshold must be between 0.0 and 1.0".into(),
            ));
        }
        Ok(())
    }

    /// Loads a manifest from JSON
    pub fn from_json(json: &str) -> Result<Self, GhostError> {
        // TODO: Implement JSON deserialization with validation
        serde_json::from_str(json).map_err(|e| GhostError::ParseError(e.to_string()))
    }

    /// Exports the manifest to JSON
    pub fn to_json(&self) -> Result<String, GhostError> {
        // TODO: Implement JSON serialization
        serde_json::to_string_pretty(self).map_err(|e| GhostError::ParseError(e.to_string()))
    }
}

/// Type of scraper worker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerType {
    /// Node.js/TypeScript based scraper
    NodeJs,
    /// Python based scraper
    Python,
    /// Go based scraper
    Go,
    /// Rust native scraper
    Rust,
    /// Official API client
    Official,
    /// Mock/simulator
    Mock,
    /// Unknown type
    Unknown,
}

impl WorkerType {
    /// Returns the bridge type needed for this worker
    pub fn bridge_type(&self) -> Option<BridgeType> {
        // TODO: Implement bridge type determination
        match self {
            WorkerType::NodeJs => Some(BridgeType::Napi),
            WorkerType::Python => Some(BridgeType::PyO3),
            WorkerType::Go => Some(BridgeType::Grpc),
            WorkerType::Rust => None, // Native, no bridge needed
            WorkerType::Official => None,
            WorkerType::Mock => None,
            WorkerType::Unknown => None,
        }
    }

    /// Returns whether this worker type is browser-based
    pub fn is_browser_based(&self) -> bool {
        // TODO: Implement browser detection
        matches!(self, WorkerType::NodeJs | WorkerType::Python)
    }
}

/// Type of FFI bridge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeType {
    /// NAPI (Node.js)
    Napi,
    /// PyO3 (Python)
    PyO3,
    /// gRPC (Go and others)
    Grpc,
    /// Unix Domain Socket
    Uds,
}

impl BridgeType {
    /// Returns whether this bridge requires external runtime
    pub fn requires_runtime(&self) -> bool {
        // TODO: Implement runtime requirement check
        matches!(
            self,
            BridgeType::PyO3 | BridgeType::Napi | BridgeType::Grpc
        )
    }

    /// Returns the runtime name
    pub fn runtime_name(&self) -> &'static str {
        // TODO: Implement runtime name
        match self {
            BridgeType::PyO3 => "python",
            BridgeType::Napi => "node",
            BridgeType::Grpc => "go",
            BridgeType::Uds => "uds",
        }
    }
}
