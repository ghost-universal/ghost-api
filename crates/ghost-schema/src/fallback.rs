//! Fallback types for Ghost API
//!
//! This module contains all types related to fallback handling,
//! tier escalation, and failure recovery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Capability, CapabilityTier, GhostError, Platform};

/// Context for fallback decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackContext {
    /// Current worker ID
    pub worker_id: String,
    /// Current tier
    pub tier: CapabilityTier,
    /// Current attempt number
    pub attempt: u32,
    /// Reason for failure
    pub failure_reason: FailureReason,
    /// Original error
    #[serde(with = "option_ghost_error_serde")]
    pub error: Option<GhostError>,
    /// Workers already tried
    pub tried_workers: Vec<String>,
}

/// Helper module for optional GhostError serialization
mod option_ghost_error_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use crate::GhostError;

    pub fn serialize<S>(error: &Option<GhostError>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: Implement proper error serialization
        match error {
            Some(e) => Some(e.to_string()).serialize(serializer),
            None => None::<String>.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<GhostError>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO: Implement proper error deserialization
        let opt = Option::<String>::deserialize(deserializer)?;
        Ok(opt.map(GhostError::Other))
    }
}

impl FallbackContext {
    /// Creates a new fallback context
    pub fn new(worker_id: impl Into<String>, tier: CapabilityTier) -> Self {
        // TODO: Implement context construction
        Self {
            worker_id: worker_id.into(),
            tier,
            attempt: 1,
            failure_reason: FailureReason::WorkerError,
            error: None,
            tried_workers: Vec::new(),
        }
    }

    /// Records a failed attempt
    pub fn record_failure(&mut self, reason: FailureReason, error: GhostError) {
        // TODO: Implement failure recording
        self.tried_workers.push(self.worker_id.clone());
        self.failure_reason = reason;
        self.error = Some(error);
        self.attempt += 1;
    }

    /// Sets the next worker to try
    pub fn set_next_worker(&mut self, worker_id: impl Into<String>) {
        // TODO: Implement worker update
        self.worker_id = worker_id.into();
    }

    /// Escalates to a higher tier
    pub fn escalate_tier(&mut self, tier: CapabilityTier) {
        // TODO: Implement tier escalation
        self.tier = tier;
        self.tried_workers.clear();
    }
}

/// Reason for fallback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureReason {
    /// Rate limited by platform
    RateLimited,
    /// WAF/challenge detected
    WafChallenge,
    /// Proxy blocked
    ProxyBlocked,
    /// Session expired or invalid
    SessionExpired,
    /// Worker error
    WorkerError,
    /// Request timeout
    Timeout,
    /// All workers in tier exhausted
    AllWorkersExhausted,
}

impl FailureReason {
    /// Returns whether this is a retryable failure
    pub fn is_retryable(&self) -> bool {
        // TODO: Implement retryability determination
        matches!(
            self,
            FailureReason::RateLimited
                | FailureReason::WafChallenge
                | FailureReason::ProxyBlocked
                | FailureReason::Timeout
        )
    }

    /// Returns whether this requires tier escalation
    pub fn requires_escalation(&self) -> bool {
        // TODO: Implement escalation requirement determination
        matches!(
            self,
            FailureReason::WafChallenge | FailureReason::AllWorkersExhausted
        )
    }

    /// Returns the recommended delay before retry (ms)
    pub fn recommended_delay_ms(&self) -> u64 {
        // TODO: Implement delay recommendation
        match self {
            FailureReason::RateLimited => 60000,   // 1 minute
            FailureReason::WafChallenge => 5000,   // 5 seconds
            FailureReason::ProxyBlocked => 1000,   // 1 second
            FailureReason::Timeout => 2000,        // 2 seconds
            _ => 0,
        }
    }
}

/// Action to take for fallback
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum FallbackAction {
    /// Pivot to a different worker in the same tier
    PivotWorker {
        /// Worker to exclude from selection
        exclude_worker: String,
    },
    /// Escalate to the next tier
    EscalateTier,
    /// Rotate to a different proxy
    RotateProxy,
    /// Refresh the session
    RefreshSession,
    /// Wait and retry
    WaitAndRetry {
        /// Delay in milliseconds
        delay_ms: u64,
    },
    /// No more fallbacks available
    GiveUp,
}

/// Step in a fallback chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStep {
    /// Tier for this step
    pub tier: CapabilityTier,
    /// Capability needed
    pub capability: Capability,
    /// Platform target
    pub platform: Platform,
}

impl FallbackStep {
    /// Creates a new fallback step
    pub fn new(tier: CapabilityTier, capability: Capability, platform: Platform) -> Self {
        // TODO: Implement step construction
        Self {
            tier,
            capability,
            platform,
        }
    }
}

/// Event recording a fallback occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackEvent {
    /// Worker that failed
    pub from_worker: String,
    /// Worker being tried
    pub to_worker: Option<String>,
    /// Tier being escalated from
    pub from_tier: CapabilityTier,
    /// Tier being escalated to
    pub to_tier: Option<CapabilityTier>,
    /// Reason for fallback
    pub reason: FailureReason,
    /// Timestamp
    pub timestamp: i64,
}

impl FallbackEvent {
    /// Creates a new fallback event
    pub fn new(
        from_worker: impl Into<String>,
        reason: FailureReason,
        from_tier: CapabilityTier,
    ) -> Self {
        // TODO: Implement event construction
        Self {
            from_worker: from_worker.into(),
            to_worker: None,
            from_tier,
            to_tier: None,
            reason,
            timestamp: 0, // TODO: Use actual timestamp
        }
    }

    /// Sets the target worker
    pub fn with_to_worker(mut self, worker_id: impl Into<String>) -> Self {
        // TODO: Implement target worker setting
        self.to_worker = Some(worker_id.into());
        self
    }

    /// Sets the target tier
    pub fn with_to_tier(mut self, tier: CapabilityTier) -> Self {
        // TODO: Implement target tier setting
        self.to_tier = Some(tier);
        self
    }
}

/// Tracker for fallback attempts
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FallbackTracker {
    /// Total fallbacks triggered
    pub total_fallbacks: u64,
    /// Fallbacks by reason
    pub by_reason: HashMap<FailureReason, u64>,
    /// Tier escalations
    pub tier_escalations: u64,
    /// Worker pivots
    pub worker_pivots: u64,
    /// Proxy rotations
    pub proxy_rotations: u64,
    /// Session refreshes
    pub session_refreshes: u64,
}

impl FallbackTracker {
    /// Creates a new tracker
    pub fn new() -> Self {
        // TODO: Implement tracker construction
        Self::default()
    }

    /// Records a fallback event
    pub fn record(&mut self, event: &FallbackEvent) {
        // TODO: Implement event recording
        self.total_fallbacks += 1;
        *self.by_reason.entry(event.reason).or_insert(0) += 1;

        if event.to_tier.is_some() {
            self.tier_escalations += 1;
        } else if event.to_worker.is_some() {
            self.worker_pivots += 1;
        }
    }

    /// Returns the fallback rate
    pub fn fallback_rate(&self, total_requests: u64) -> f64 {
        // TODO: Implement rate calculation
        if total_requests == 0 {
            0.0
        } else {
            self.total_fallbacks as f64 / total_requests as f64
        }
    }
}
