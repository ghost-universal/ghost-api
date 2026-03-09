//! Tests for Ghost Schema
//!
//! This module contains tests for the schema types.

mod types_tests {
    use ghost_schema::{GhostPost, GhostUser, GhostMedia, MediaType, Platform};

    #[test]
    fn test_ghost_post_creation() {
        // TODO: Implement GhostPost creation test
        let post = GhostPost::new("123", Platform::X, "Hello world");
        assert_eq!(post.id, "123");
        assert_eq!(post.platform, Platform::X);
        assert_eq!(post.text, "Hello world");
    }

    #[test]
    fn test_ghost_user_creation() {
        // TODO: Implement GhostUser creation test
        let user = GhostUser::new("456", Platform::Threads, "testuser");
        assert_eq!(user.id, "456");
        assert_eq!(user.platform, Platform::Threads);
        assert_eq!(user.username, "testuser");
    }

    #[test]
    fn test_ghost_media_creation() {
        // TODO: Implement GhostMedia creation test
        let media = GhostMedia::new(MediaType::Image, "https://example.com/image.png");
        assert_eq!(media.media_type, MediaType::Image);
        assert_eq!(media.url, "https://example.com/image.png");
    }
}

mod context_tests {
    use ghost_schema::{GhostContext, Strategy, BudgetLimits};

    #[test]
    fn test_context_builder() {
        // TODO: Implement context builder test
        let ctx = GhostContext::builder()
            .tenant_id("test_tenant")
            .strategy(Strategy::HealthFirst)
            .build();

        assert_eq!(ctx.tenant_id, Some("test_tenant".to_string()));
        assert_eq!(ctx.strategy, Strategy::HealthFirst);
    }

    #[test]
    fn test_context_with_budget() {
        // TODO: Implement context with budget test
        let budget = BudgetLimits::new(1000, 50.0, 80);
        let ctx = GhostContext::builder()
            .tenant_id("test_tenant")
            .budget(budget)
            .build();

        assert!(ctx.budget.is_some());
    }

    #[test]
    fn test_context_with_session() {
        // TODO: Implement context with session test
        let ctx = GhostContext::builder()
            .tenant_id("test_tenant")
            .session("auth_token=abc123; ct0=xyz789")
            .build();

        assert!(ctx.session.is_some());
    }
}

mod platform_tests {
    use ghost_schema::Platform;

    #[test]
    fn test_platform_from_str() {
        // TODO: Implement platform parsing test
        assert_eq!(Platform::from_str("x"), Platform::X);
        assert_eq!(Platform::from_str("twitter"), Platform::X);
        assert_eq!(Platform::from_str("threads"), Platform::Threads);
        assert_eq!(Platform::from_str("unknown"), Platform::Unknown);
    }

    #[test]
    fn test_platform_base_url() {
        // TODO: Implement platform base URL test
        assert_eq!(Platform::X.base_url(), "https://x.com");
        assert_eq!(Platform::Threads.base_url(), "https://www.threads.net");
    }
}

mod capability_tests {
    use ghost_schema::{Capability, CapabilityTier, Platform};

    #[test]
    fn test_capability_platform_mapping() {
        // TODO: Implement capability platform mapping test
        assert_eq!(Capability::XRead.platform(), Some(Platform::X));
        assert_eq!(Capability::ThreadsRead.platform(), Some(Platform::Threads));
        assert_eq!(Capability::MediaDownload.platform(), None);
    }

    #[test]
    fn test_capability_tier() {
        // TODO: Implement capability tier test
        assert_eq!(Capability::OfficialApi.tier(), CapabilityTier::Official);
        assert_eq!(Capability::BrowserBased.tier(), CapabilityTier::Heavy);
        assert_eq!(Capability::RequestBased.tier(), CapabilityTier::Fast);
    }

    #[test]
    fn test_capability_for_platform() {
        // TODO: Implement capability for platform test
        let x_caps = Capability::for_platform(Platform::X);
        assert!(x_caps.contains(&Capability::XRead));
        assert!(x_caps.contains(&Capability::XSearch));
    }
}

mod fallback_tests {
    use ghost_schema::{FailureReason, CapabilityTier};

    #[test]
    fn test_failure_reason_retryable() {
        // TODO: Implement failure reason retryable test
        assert!(FailureReason::RateLimited.is_retryable());
        assert!(FailureReason::WafChallenge.is_retryable());
        assert!(!FailureReason::SessionExpired.is_retryable());
    }

    #[test]
    fn test_failure_reason_escalation() {
        // TODO: Implement failure reason escalation test
        assert!(FailureReason::WafChallenge.requires_escalation());
        assert!(FailureReason::AllWorkersExhausted.requires_escalation());
        assert!(!FailureReason::Timeout.requires_escalation());
    }

    #[test]
    fn test_capability_tier_fallback() {
        // TODO: Implement capability tier fallback test
        assert_eq!(CapabilityTier::Fast.fallback(), Some(CapabilityTier::Heavy));
        assert_eq!(CapabilityTier::Heavy.fallback(), Some(CapabilityTier::Official));
        assert_eq!(CapabilityTier::Official.fallback(), None);
    }
}
