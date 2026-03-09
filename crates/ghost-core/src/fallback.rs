//! Fallback hierarchy and tier escalation logic
//!
//! This module manages fallback strategies when workers fail,
//! including tier escalation and retry policies.

use ghost_schema::{
    Capability, CapabilityTier, GhostError, Platform, Strategy,
    FallbackContext, FailureReason, FallbackAction, FallbackStep,
    FallbackEvent, FallbackTracker,
};

use crate::GhostConfig;

/// Fallback engine for managing request escalation
///
/// The fallback engine determines what action to take when a worker fails,
/// including pivoting to other workers, escalating tiers, or giving up.
pub struct FallbackEngine {
    config: GhostConfig,
    tracker: FallbackTracker,
}

impl FallbackEngine {
    /// Creates a new fallback engine
    pub fn new(config: &GhostConfig) -> Self {
        Self {
            config: config.clone(),
            tracker: FallbackTracker::new(),
        }
    }

    /// Determines the next fallback option
    ///
    /// Based on the failure reason and current context, returns the
    /// appropriate fallback action to take.
    pub fn get_fallback(&self, current: &FallbackContext) -> Option<FallbackAction> {
        // Check if we've exceeded max retries
        if current.attempt >= self.config.max_retries {
            return Some(FallbackAction::GiveUp);
        }

        // Determine action based on failure reason
        match current.failure_reason {
            FailureReason::RateLimited => {
                // Try pivot to different worker in same tier
                Some(FallbackAction::PivotWorker {
                    exclude_worker: current.worker_id.clone(),
                })
            }
            FailureReason::WafChallenge => {
                // Escalate to heavier tier for browser-based solving
                Some(FallbackAction::EscalateTier)
            }
            FailureReason::ProxyBlocked => {
                // Try different proxy or worker
                Some(FallbackAction::RotateProxy)
            }
            FailureReason::SessionExpired => {
                // Need new session credentials
                Some(FallbackAction::RefreshSession)
            }
            FailureReason::WorkerError => {
                // Try different worker or escalate based on attempt count
                if current.attempt < self.config.max_retries / 2 {
                    Some(FallbackAction::PivotWorker {
                        exclude_worker: current.worker_id.clone(),
                    })
                } else {
                    Some(FallbackAction::EscalateTier)
                }
            }
            FailureReason::Timeout => {
                // Try faster worker or escalate
                Some(FallbackAction::PivotWorker {
                    exclude_worker: current.worker_id.clone(),
                })
            }
            FailureReason::AllWorkersExhausted => {
                // Escalate to next tier
                Some(FallbackAction::EscalateTier)
            }
        }
    }

    /// Returns the next tier to escalate to
    ///
    /// Tiers escalate in order: Fast -> Heavy -> Official
    pub fn next_tier(&self, current_tier: CapabilityTier) -> Option<CapabilityTier> {
        current_tier.fallback()
    }

    /// Checks if fallback should be attempted
    ///
    /// Returns true if there are remaining attempts and the failure is retryable.
    pub fn should_fallback(&self, context: &FallbackContext) -> bool {
        context.attempt < self.config.max_retries
            && context.failure_reason.is_retryable()
    }

    /// Creates a fallback chain for a request
    ///
    /// The chain defines the order of tiers to try, based on the strategy.
    pub fn create_fallback_chain(
        &self,
        capability: Capability,
        platform: Platform,
        strategy: Strategy,
    ) -> Vec<FallbackStep> {
        let mut steps = Vec::new();

        match strategy {
            Strategy::OfficialOnly => {
                // Only official tier
                steps.push(FallbackStep::new(CapabilityTier::Official, capability, platform));
            }
            Strategy::ScrapersOnly => {
                // Only scraper tiers
                steps.push(FallbackStep::new(CapabilityTier::Fast, capability, platform));
                steps.push(FallbackStep::new(CapabilityTier::Heavy, capability, platform));
            }
            Strategy::OfficialFirst => {
                // Official first, then scrapers
                steps.push(FallbackStep::new(CapabilityTier::Official, capability, platform));
                steps.push(FallbackStep::new(CapabilityTier::Fast, capability, platform));
                steps.push(FallbackStep::new(CapabilityTier::Heavy, capability, platform));
            }
            _ => {
                // Default: Fast -> Heavy -> Official
                steps.push(FallbackStep::new(CapabilityTier::Fast, capability, platform));
                steps.push(FallbackStep::new(CapabilityTier::Heavy, capability, platform));
                steps.push(FallbackStep::new(CapabilityTier::Official, capability, platform));
            }
        }

        steps
    }

    /// Records a fallback event for analytics
    ///
    /// This is used for monitoring and alerting on high fallback rates.
    pub fn record_fallback(&mut self, event: &FallbackEvent) {
        tracing::info!(
            worker_id = %event.from_worker,
            to_worker = ?event.to_worker,
            from_tier = ?event.from_tier,
            to_tier = ?event.to_tier,
            reason = ?event.reason,
            "Fallback event recorded"
        );
        self.tracker.record(event);
    }

    /// Returns the fallback tracker statistics
    pub fn tracker(&self) -> &FallbackTracker {
        &self.tracker
    }

    /// Returns the total number of fallbacks
    pub fn total_fallbacks(&self) -> u64 {
        self.tracker.total_fallbacks
    }

    /// Returns the fallback rate
    ///
    /// Calculated as total fallbacks / total requests
    pub fn fallback_rate(&self, total_requests: u64) -> f64 {
        self.tracker.fallback_rate(total_requests)
    }

    /// Determines failure reason from an error
    pub fn classify_error(&self, error: &GhostError) -> FailureReason {
        match error {
            GhostError::RateLimited { .. } => FailureReason::RateLimited,
            GhostError::WafChallenge { .. } => FailureReason::WafChallenge,
            GhostError::ProxyError { .. } => FailureReason::ProxyBlocked,
            GhostError::SessionExpired { .. } => FailureReason::SessionExpired,
            GhostError::Timeout(_) => FailureReason::Timeout,
            GhostError::WorkersExhausted(_) => FailureReason::AllWorkersExhausted,
            _ => FailureReason::WorkerError,
        }
    }

    /// Gets the recommended delay before retry
    ///
    /// Uses exponential backoff with jitter.
    pub fn get_retry_delay(&self, attempt: u32, reason: FailureReason) -> std::time::Duration {
        let base_delay = reason.recommended_delay_ms();

        // Exponential backoff: base * 2^attempt
        let backoff = base_delay * (1u64 << attempt.min(5));

        // Cap at max delay
        let max_delay = 60_000; // 60 seconds
        let delay = backoff.min(max_delay);

        // Add 10% jitter
        let jitter = delay / 10;

        std::time::Duration::from_millis(delay + jitter)
    }

    /// Creates a fallback context for a new request
    pub fn create_context(&self, worker_id: &str, tier: CapabilityTier) -> FallbackContext {
        FallbackContext::new(worker_id, tier)
    }

    /// Returns statistics by failure reason
    pub fn stats_by_reason(&self) -> &std::collections::HashMap<FailureReason, u64> {
        &self.tracker.by_reason
    }

    /// Returns the number of tier escalations
    pub fn tier_escalations(&self) -> u64 {
        self.tracker.tier_escalations
    }

    /// Returns the number of worker pivots
    pub fn worker_pivots(&self) -> u64 {
        self.tracker.worker_pivots
    }

    /// Returns the configuration
    pub fn config(&self) -> &GhostConfig {
        &self.config
    }
}

impl Default for FallbackEngine {
    fn default() -> Self {
        Self::new(&GhostConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_engine_creation() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);
        assert_eq!(engine.total_fallbacks(), 0);
    }

    #[test]
    fn test_get_fallback_rate_limited() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        let ctx = FallbackContext {
            worker_id: "test".to_string(),
            tier: CapabilityTier::Fast,
            attempt: 1,
            failure_reason: FailureReason::RateLimited,
            error: None,
            tried_workers: vec![],
        };

        let action = engine.get_fallback(&ctx);
        assert!(matches!(action, Some(FallbackAction::PivotWorker { .. })));
    }

    #[test]
    fn test_get_fallback_waf_challenge() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        let ctx = FallbackContext {
            worker_id: "test".to_string(),
            tier: CapabilityTier::Fast,
            attempt: 1,
            failure_reason: FailureReason::WafChallenge,
            error: None,
            tried_workers: vec![],
        };

        let action = engine.get_fallback(&ctx);
        assert!(matches!(action, Some(FallbackAction::EscalateTier)));
    }

    #[test]
    fn test_get_fallback_max_retries() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        let ctx = FallbackContext {
            worker_id: "test".to_string(),
            tier: CapabilityTier::Fast,
            attempt: 10, // Exceeds max retries
            failure_reason: FailureReason::WorkerError,
            error: None,
            tried_workers: vec![],
        };

        let action = engine.get_fallback(&ctx);
        assert!(matches!(action, Some(FallbackAction::GiveUp)));
    }

    #[test]
    fn test_next_tier() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        assert_eq!(engine.next_tier(CapabilityTier::Fast), Some(CapabilityTier::Heavy));
        assert_eq!(engine.next_tier(CapabilityTier::Heavy), Some(CapabilityTier::Official));
        assert_eq!(engine.next_tier(CapabilityTier::Official), None);
    }

    #[test]
    fn test_create_fallback_chain() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        let chain = engine.create_fallback_chain(Capability::XRead, Platform::X, Strategy::HealthFirst);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].tier, CapabilityTier::Fast);
        assert_eq!(chain[1].tier, CapabilityTier::Heavy);
        assert_eq!(chain[2].tier, CapabilityTier::Official);
    }

    #[test]
    fn test_create_fallback_chain_official_only() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        let chain = engine.create_fallback_chain(Capability::XRead, Platform::X, Strategy::OfficialOnly);
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].tier, CapabilityTier::Official);
    }

    #[test]
    fn test_classify_error() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        assert_eq!(
            engine.classify_error(&GhostError::RateLimited { retry_after: None, platform: Platform::X }),
            FailureReason::RateLimited
        );
        assert_eq!(
            engine.classify_error(&GhostError::Timeout("test".into())),
            FailureReason::Timeout
        );
    }

    #[test]
    fn test_get_retry_delay() {
        let config = GhostConfig::default();
        let engine = FallbackEngine::new(&config);

        let delay1 = engine.get_retry_delay(1, FailureReason::RateLimited);
        let delay2 = engine.get_retry_delay(2, FailureReason::RateLimited);

        // Delay should increase with attempt
        assert!(delay2 > delay1);
    }

    #[test]
    fn test_record_fallback() {
        let mut config = GhostConfig::default();
        config.max_retries = 3;
        let mut engine = FallbackEngine::new(&config);

        let event = FallbackEvent::new("worker1", FailureReason::RateLimited, CapabilityTier::Fast);
        engine.record_fallback(&event);

        assert_eq!(engine.total_fallbacks(), 1);
    }
}
