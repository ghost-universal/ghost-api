//! Worker health, status, and statistics types
//!
//! This module contains all types related to worker management,
//! health tracking, and operational status.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Capability, Platform};

/// Worker health status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorkerHealth {
    /// Health score (0.0 - 1.0)
    pub score: f64,
    /// Success rate over recent requests
    pub success_rate: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// Whether the worker is cooling down
    pub is_cooling_down: bool,
}

impl WorkerHealth {
    /// Creates a new WorkerHealth with defaults
    pub fn new() -> Self {
        // TODO: Implement WorkerHealth construction
        Self {
            score: 1.0,
            success_rate: 1.0,
            avg_latency_ms: 0,
            consecutive_failures: 0,
            is_cooling_down: false,
        }
    }

    /// Updates health based on a successful request
    pub fn record_success(&mut self, latency_ms: u64) {
        // TODO: Implement success recording with rolling window
        self.consecutive_failures = 0;
        self.avg_latency_ms = latency_ms;
    }

    /// Updates health based on a failed request
    pub fn record_failure(&mut self) {
        // TODO: Implement failure recording with rolling window
        self.consecutive_failures += 1;
        self.success_rate = (self.success_rate * 0.9).max(0.0);
    }

    /// Calculates the health score
    pub fn calculate_score(&self, success_rate: f64, latency_ms: u64, max_latency_ms: u64) -> f64 {
        // TODO: Implement health score calculation
        // Health = (S_rate × 0.6) + (L_norm × 0.4)
        let latency_norm = 1.0 - (latency_ms as f64 / max_latency_ms as f64).min(1.0);
        (success_rate * 0.6) + (latency_norm * 0.4)
    }

    /// Returns the health tier
    pub fn tier(&self) -> HealthTier {
        // TODO: Implement tier determination
        if self.score > 0.8 {
            HealthTier::Healthy
        } else if self.score > 0.5 {
            HealthTier::Degraded
        } else if self.score > 0.0 {
            HealthTier::Unhealthy
        } else {
            HealthTier::Dead
        }
    }

    /// Returns whether this worker should be used
    pub fn is_usable(&self, threshold: f64) -> bool {
        // TODO: Implement usability check
        self.score >= threshold && !self.is_cooling_down
    }
}

impl Default for WorkerHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// Health tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthTier {
    /// Health > 0.8 - Preferred for routing
    Healthy,
    /// Health 0.5 - 0.8 - Used when healthy workers unavailable
    Degraded,
    /// Health < 0.5 - Only used as last resort
    Unhealthy,
    /// Health = 0.0 - Circuit breaker engaged
    Dead,
}

impl HealthTier {
    /// Returns the priority for this tier (higher = better)
    pub fn priority(&self) -> u8 {
        // TODO: Implement tier priority
        match self {
            HealthTier::Healthy => 3,
            HealthTier::Degraded => 2,
            HealthTier::Unhealthy => 1,
            HealthTier::Dead => 0,
        }
    }
}

/// Worker operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    /// Worker is idle and ready
    Idle,
    /// Worker is processing requests
    Busy,
    /// Worker is in cooldown mode
    CoolingDown,
    /// Worker is offline
    Offline,
    /// Worker is initializing
    Initializing,
    /// Worker encountered an error
    Error,
}

/// Worker statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkerStats {
    /// Total requests handled
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average latency
    pub avg_latency_ms: u64,
    /// P95 latency
    pub p95_latency_ms: u64,
    /// Last request timestamp
    pub last_request_at: Option<i64>,
    /// Current load (0.0 - 1.0)
    pub current_load: f64,
}

impl WorkerStats {
    /// Creates new empty stats
    pub fn new() -> Self {
        // TODO: Implement stats construction
        Self::default()
    }

    /// Records a request result
    pub fn record(&mut self, success: bool, _latency_ms: u64) {
        // TODO: Implement stats recording
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
        self.last_request_at = Some(0); // TODO: Use actual timestamp
    }

    /// Returns the success rate
    pub fn success_rate(&self) -> f64 {
        // TODO: Implement success rate calculation
        if self.total_requests == 0 {
            1.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }
}

/// Result of worker selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerSelection {
    /// Selected worker ID
    pub worker_id: String,
    /// Tier of the selection
    pub tier: crate::CapabilityTier,
}

impl WorkerSelection {
    /// Creates a new worker selection
    pub fn new(worker_id: impl Into<String>, tier: crate::CapabilityTier) -> Self {
        // TODO: Implement selection construction
        Self {
            worker_id: worker_id.into(),
            tier,
        }
    }
}

/// Aggregated health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Number of healthy workers
    pub healthy_count: usize,
    /// Number of degraded workers
    pub degraded_count: usize,
    /// Number of unhealthy workers
    pub unhealthy_count: usize,
    /// Total number of workers
    pub total_count: usize,
    /// Average health score
    pub avg_score: f64,
    /// Per-platform status
    pub platform_status: HashMap<Platform, PlatformHealthStatus>,
}

impl HealthStatus {
    /// Creates a new empty health status
    pub fn new() -> Self {
        // TODO: Implement health status construction
        Self {
            healthy_count: 0,
            degraded_count: 0,
            unhealthy_count: 0,
            total_count: 0,
            avg_score: 0.0,
            platform_status: HashMap::new(),
        }
    }

    /// Returns whether all workers are healthy
    pub fn all_healthy(&self) -> bool {
        // TODO: Implement health check
        self.unhealthy_count == 0 && self.degraded_count == 0
    }

    /// Returns whether any workers are available
    pub fn has_available_workers(&self) -> bool {
        // TODO: Implement availability check
        self.healthy_count > 0 || self.degraded_count > 0
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-platform health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformHealthStatus {
    /// Platform
    pub platform: Platform,
    /// Available workers for this platform
    pub available_workers: usize,
    /// Average latency in ms
    pub avg_latency_ms: u64,
    /// Health score
    pub health_score: f64,
}

/// Circuit breaker for managing failing workers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    /// Whether the circuit is open (blocking requests)
    pub is_open: bool,
    /// Time when the circuit was opened (timestamp)
    pub opened_at: Option<i64>,
    /// Timeout in seconds before attempting to close
    pub timeout_secs: u64,
    /// Number of successful probes in half-open state
    pub successful_probes: u32,
    /// Number of probes required to close
    pub probes_required: u32,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker
    pub fn new(timeout_secs: u64) -> Self {
        // TODO: Implement circuit breaker construction
        Self {
            is_open: false,
            opened_at: None,
            timeout_secs,
            successful_probes: 0,
            probes_required: 3,
        }
    }

    /// Trips the circuit breaker
    pub fn trip(&mut self) {
        // TODO: Implement circuit tripping
        self.is_open = true;
        self.opened_at = Some(0); // TODO: Use actual timestamp
        self.successful_probes = 0;
    }

    /// Resets the circuit breaker
    pub fn reset(&mut self) {
        // TODO: Implement circuit reset
        self.is_open = false;
        self.opened_at = None;
        self.successful_probes = 0;
    }

    /// Checks if the circuit is open
    pub fn is_open(&self) -> bool {
        // TODO: Implement circuit status check with timeout
        self.is_open
    }

    /// Checks if in half-open state
    pub fn is_half_open(&self) -> bool {
        // TODO: Implement half-open detection
        self.is_open && self.opened_at.is_some()
    }

    /// Records a successful probe
    pub fn record_probe_success(&mut self) {
        // TODO: Implement probe success recording
        if self.is_half_open() {
            self.successful_probes += 1;
            if self.successful_probes >= self.probes_required {
                self.reset();
            }
        }
    }

    /// Records a failed probe
    pub fn record_probe_failure(&mut self) {
        // TODO: Implement probe failure recording
        self.trip();
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Worker ID
    pub worker_id: String,
    /// Whether the check passed
    pub passed: bool,
    /// Latency in ms
    pub latency_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
    /// Timestamp
    pub timestamp: i64,
}

impl HealthCheckResult {
    /// Creates a successful health check result
    pub fn success(worker_id: impl Into<String>, latency_ms: u64) -> Self {
        // TODO: Implement success result construction
        Self {
            worker_id: worker_id.into(),
            passed: true,
            latency_ms,
            error: None,
            timestamp: 0, // TODO: Use actual timestamp
        }
    }

    /// Creates a failed health check result
    pub fn failure(worker_id: impl Into<String>, error: impl Into<String>) -> Self {
        // TODO: Implement failure result construction
        Self {
            worker_id: worker_id.into(),
            passed: false,
            latency_ms: 0,
            error: Some(error.into()),
            timestamp: 0, // TODO: Use actual timestamp
        }
    }
}

/// Worker selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerSelectionCriteria {
    /// Required capability
    pub capability: Capability,
    /// Required platform
    pub platform: Platform,
    /// Minimum health score
    pub min_health_score: f64,
    /// Exclude workers
    pub exclude_workers: Vec<String>,
    /// Prefer certain worker types
    pub prefer_worker_type: Option<crate::WorkerType>,
}

impl WorkerSelectionCriteria {
    /// Creates new selection criteria
    pub fn new(capability: Capability, platform: Platform) -> Self {
        // TODO: Implement criteria construction
        Self {
            capability,
            platform,
            min_health_score: 0.0,
            exclude_workers: Vec::new(),
            prefer_worker_type: None,
        }
    }
}
