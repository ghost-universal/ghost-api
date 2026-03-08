//! Event types for Ghost API
//!
//! This module contains all event-related types used for
//! monitoring and reacting to system events.

use serde::{Deserialize, Serialize};

use crate::{Capability, GhostError, Platform};

/// Events emitted by the Ghost engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GhostEvent {
    /// Worker registered
    WorkerRegistered {
        worker_id: String,
        capabilities: Vec<Capability>,
    },
    /// Worker unregistered
    WorkerUnregistered {
        worker_id: String,
    },
    /// Worker health changed
    WorkerHealthChanged {
        worker_id: String,
        old_score: f64,
        new_score: f64,
    },
    /// Worker went offline
    WorkerOffline {
        worker_id: String,
        reason: String,
    },
    /// Request started
    RequestStarted {
        request_id: String,
        worker_id: String,
        platform: Platform,
        capability: Capability,
    },
    /// Request completed
    RequestCompleted {
        request_id: String,
        worker_id: String,
        platform: Platform,
        latency_ms: u64,
    },
    /// Request failed
    RequestFailed {
        request_id: String,
        worker_id: String,
        platform: Platform,
        #[serde(with = "ghost_error_serde")]
        error: GhostError,
    },
    /// Fallback triggered
    FallbackTriggered {
        from_worker: String,
        to_worker: Option<String>,
        reason: String,
        tier_escalation: bool,
    },
    /// Circuit breaker opened
    CircuitBreakerOpened {
        worker_id: String,
    },
    /// Circuit breaker closed
    CircuitBreakerClosed {
        worker_id: String,
    },
    /// Rate limit detected
    RateLimitDetected {
        worker_id: String,
        platform: Platform,
        retry_after: Option<u64>,
    },
    /// Session updated
    SessionUpdated {
        tenant_id: String,
        session_id: String,
    },
    /// Session unhealthy
    SessionUnhealthy {
        session_id: String,
        reason: SessionUnhealthyReason,
    },
    /// Session recovered
    SessionRecovered {
        session_id: String,
    },
    /// Budget approaching limit
    BudgetApproachingLimit {
        tenant_id: String,
        usage_percent: u8,
    },
    /// Budget exceeded
    BudgetExceeded {
        tenant_id: String,
        limit_type: String,
    },
    /// Budget reset
    BudgetReset {
        tenant_id: String,
    },
    /// Autoscaling event
    AutoscaleEvent {
        event_type: AutoscaleEventType,
        from_count: usize,
        to_count: usize,
        reason: String,
    },
    /// Health check completed
    HealthCheckCompleted {
        worker_id: String,
        passed: bool,
        latency_ms: u64,
    },
}

/// Helper module for serializing GhostError
mod ghost_error_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use crate::GhostError;

    pub fn serialize<S>(error: &GhostError, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: Implement proper error serialization
        error.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<GhostError, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO: Implement proper error deserialization
        let s = String::deserialize(deserializer)?;
        Ok(GhostError::Other(s))
    }
}

impl GhostEvent {
    /// Returns the event type name
    pub fn event_type(&self) -> &'static str {
        // TODO: Implement event type extraction
        match self {
            GhostEvent::WorkerRegistered { .. } => "worker_registered",
            GhostEvent::WorkerUnregistered { .. } => "worker_unregistered",
            GhostEvent::WorkerHealthChanged { .. } => "worker_health_changed",
            GhostEvent::WorkerOffline { .. } => "worker_offline",
            GhostEvent::RequestStarted { .. } => "request_started",
            GhostEvent::RequestCompleted { .. } => "request_completed",
            GhostEvent::RequestFailed { .. } => "request_failed",
            GhostEvent::FallbackTriggered { .. } => "fallback_triggered",
            GhostEvent::CircuitBreakerOpened { .. } => "circuit_breaker_opened",
            GhostEvent::CircuitBreakerClosed { .. } => "circuit_breaker_closed",
            GhostEvent::RateLimitDetected { .. } => "rate_limit_detected",
            GhostEvent::SessionUpdated { .. } => "session_updated",
            GhostEvent::SessionUnhealthy { .. } => "session_unhealthy",
            GhostEvent::SessionRecovered { .. } => "session_recovered",
            GhostEvent::BudgetApproachingLimit { .. } => "budget_approaching_limit",
            GhostEvent::BudgetExceeded { .. } => "budget_exceeded",
            GhostEvent::BudgetReset { .. } => "budget_reset",
            GhostEvent::AutoscaleEvent { .. } => "autoscale_event",
            GhostEvent::HealthCheckCompleted { .. } => "health_check_completed",
        }
    }

    /// Returns a timestamp for the event
    pub fn timestamp(&self) -> i64 {
        // TODO: Implement timestamp extraction or generation
        0
    }
}

/// Reasons for session being unhealthy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum SessionUnhealthyReason {
    /// Account is suspended
    Suspended,
    /// Account is rate limited
    RateLimited {
        /// Seconds until rate limit resets
        retry_after: u64,
    },
    /// Cookies have expired
    CookieExpired,
    /// Account is locked
    Locked,
    /// Challenge required
    ChallengeRequired {
        /// Type of challenge
        challenge_type: String,
    },
    /// Unknown reason
    Unknown,
}

impl SessionUnhealthyReason {
    /// Returns whether the session can recover
    pub fn can_recover(&self) -> bool {
        // TODO: Implement recoverability check
        matches!(
            self,
            SessionUnhealthyReason::RateLimited { .. }
                | SessionUnhealthyReason::CookieExpired
        )
    }

    /// Returns the recommended action
    pub fn recommended_action(&self) -> SessionAction {
        // TODO: Implement action recommendation
        match self {
            SessionUnhealthyReason::Suspended => SessionAction::Remove,
            SessionUnhealthyReason::RateLimited { retry_after } => {
                SessionAction::Park { duration_secs: *retry_after }
            }
            SessionUnhealthyReason::CookieExpired => SessionAction::RefreshCredentials,
            SessionUnhealthyReason::Locked => SessionAction::Remove,
            SessionUnhealthyReason::ChallengeRequired { .. } => SessionAction::Challenge,
            SessionUnhealthyReason::Unknown => SessionAction::Emit,
        }
    }
}

/// Actions to take for unhealthy sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SessionAction {
    /// Remove the session permanently
    Remove,
    /// Park the session temporarily
    Park {
        /// Duration in seconds
        duration_secs: u64,
    },
    /// Request fresh credentials
    RefreshCredentials,
    /// Handle challenge
    Challenge,
    /// Just emit an event
    Emit,
}

/// Types of autoscaling events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoscaleEventType {
    /// Scaling up
    ScalingUp,
    /// Scaling down
    ScalingDown,
    /// Spot instance interrupted
    SpotInterrupted,
}
