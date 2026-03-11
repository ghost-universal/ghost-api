# Comprehensive Test Plan for Ghost-API

## Executive Summary

This document outlines a comprehensive testing strategy for all crates in the ghost-api workspace. The testing approach emphasizes **clean isolated tests**, **real implementation verification**, **no mocks/stubs**, and **thorough assertion coverage**. Each test case must verify actual behavior, state transitions, and edge cases.

---

## Testing Philosophy

### Core Principles

1. **No Mocks or Stubs**: All tests must exercise real implementation code. Test doubles are only acceptable for external services (network, file system) when absolutely necessary.

2. **Clean Isolation**: Each test must be independently executable. Tests must not share mutable state, must clean up resources, and must not depend on execution order.

3. **Deep Verification**: Tests must verify:
   - Return values and output state
   - Internal state transitions
   - Error conditions and error types
   - Edge cases and boundary conditions
   - Concurrency behavior where applicable

4. **Realistic Test Data**: Use production-like data structures and values. Avoid trivial "foo/bar" strings that mask parsing issues.

---

## Crate Analysis Overview

| Crate | Current Test Status | Priority | Estimated Test Count |
|-------|---------------------|----------|---------------------|
| ghost-schema | Partial (TODOs) | HIGH | 150+ |
| ghost-core | Good coverage | HIGH | 100+ |
| ghost-vault | Empty (TODOs only) | HIGH | 120+ |
| ghost-bridge | Basic coverage | MEDIUM | 80+ |
| x-adapter | No tests | HIGH | 100+ |
| threads-adapter | No tests | HIGH | 100+ |
| ghost-server | No tests | MEDIUM | 80+ |

**Total Estimated Tests: 730+**

---

## 1. ghost-schema Testing Plan

### 1.1 Types Module Tests (`src/types.rs`)

#### Unit Tests for GhostPost

```rust
mod ghost_post_tests {
    // Creation & Initialization
    - test_ghost_post_new_creates_valid_instance
    - test_ghost_post_new_with_empty_id
    - test_ghost_post_new_with_special_characters_in_text
    - test_ghost_post_default_values_are_correct
    
    // Validation Tests
    - test_ghost_post_validate_succeeds_for_valid_post
    - test_ghost_post_validate_fails_for_empty_id
    - test_ghost_post_validate_fails_for_empty_text
    - test_ghost_post_validate_fails_for_invalid_platform
    
    // Serialization Tests
    - test_ghost_post_serialize_produces_valid_json
    - test_ghost_post_deserialize_from_valid_json
    - test_ghost_post_deserialize_handles_missing_optional_fields
    - test_ghost_post_deserialize_handles_null_fields
    - test_ghost_post_roundtrip_serialization
    
    // Field Tests
    - test_ghost_post_with_quoted_post
    - test_ghost_post_with_nested_quoted_post_depth_limit
    - test_ghost_post_in_reply_to_chain
    - test_ghost_post_media_attachments
    - test_ghost_post_raw_metadata_preservation
    
    // Platform Format Conversion
    - test_ghost_post_to_platform_format_x
    - test_ghost_post_to_platform_format_threads
    - test_ghost_post_to_platform_format_unsupported
}
```

#### Unit Tests for GhostUser

```rust
mod ghost_user_tests {
    // Creation Tests
    - test_ghost_user_new_basic
    - test_ghost_user_new_with_special_username
    - test_ghost_user_default_values
    - test_ghost_user_display_name_unicode
    
    // Validation Tests
    - test_ghost_user_validate_succeeds_valid
    - test_ghost_user_validate_empty_id_fails
    - test_ghost_user_validate_empty_username_fails
    - test_ghost_user_validate_username_with_reserved_chars
    
    // Serialization Tests
    - test_ghost_user_serialize_deserialize_roundtrip
    - test_ghost_user_deserialize_partial_json
    - test_ghost_user_serialize_includes_all_fields
    
    // Metadata Tests
    - test_ghost_user_raw_metadata_optional
    - test_ghost_user_verified_status
    - test_ghost_user_private_status
    - test_ghost_user_bot_flag
    - test_ghost_user_counts_optional
}
```

#### Unit Tests for GhostMedia

```rust
mod ghost_media_tests {
    // Creation Tests
    - test_ghost_media_new_image
    - test_ghost_media_new_video
    - test_ghost_media_new_gif
    - test_ghost_media_new_audio
    - test_ghost_media_new_unknown_type
    
    // Validation Tests
    - test_ghost_media_validate_url_required
    - test_ghost_media_validate_url_malformed
    - test_ghost_media_validate_dimensions
    - test_ghost_media_validate_duration
    
    // Field Tests
    - test_ghost_media_preview_url_optional
    - test_ghost_media_alt_text_accessibility
    - test_ghost_media_content_type_detection
    - test_ghost_media_size_bytes_tracking
}
```

#### Unit Tests for GhostContext

```rust
mod ghost_context_tests {
    // Builder Pattern Tests
    - test_ghost_context_builder_minimal
    - test_ghost_context_builder_full
    - test_ghost_context_builder_chained_calls
    - test_ghost_context_builder_default_strategy
    
    // Tenant Tests
    - test_ghost_context_tenant_id_optional
    - test_ghost_context_tenant_id_empty_string
    - test_ghost_context_tenant_id_unicode
    
    // Proxy Tests
    - test_ghost_context_proxy_from_url_http
    - test_ghost_context_proxy_from_url_https
    - test_ghost_context_proxy_from_url_socks5
    - test_ghost_context_proxy_from_url_socks4
    - test_ghost_context_proxy_from_url_malformed
    - test_ghost_context_proxy_from_url_with_auth
    - test_ghost_context_proxy_config_direct
    
    // Session Tests
    - test_ghost_context_session_from_cookies
    - test_ghost_context_session_from_bearer
    - test_ghost_context_session_data_direct
    - test_ghost_context_session_cookies_parse
    
    // Strategy Tests
    - test_ghost_context_strategy_health_first
    - test_ghost_context_strategy_fastest
    - test_ghost_context_strategy_cost_optimized
    - test_ghost_context_strategy_official_first
    - test_ghost_context_strategy_official_only
    - test_ghost_context_strategy_scrapers_only
    - test_ghost_context_strategy_round_robin
    
    // Budget Tests
    - test_ghost_context_budget_default_none
    - test_ghost_context_budget_with_limits
    - test_ghost_context_budget_validation
    
    // Metadata Tests
    - test_ghost_context_metadata_empty
    - test_ghost_context_metadata_single_entry
    - test_ghost_context_metadata_multiple_entries
    - test_ghost_context_metadata_unicode_values
    
    // Validation Tests
    - test_ghost_context_validate_empty_succeeds
    - test_ghost_context_validate_invalid_proxy_fails
    - test_ghost_context_validate_conflicting_settings
}
```

#### Unit Tests for Strategy Enum

```rust
mod strategy_tests {
    - test_strategy_default_is_health_first
    - test_strategy_fallback_health_first_returns_official_first
    - test_strategy_fallback_fastest_returns_health_first
    - test_strategy_fallback_cost_optimized_returns_health_first
    - test_strategy_fallback_official_first_returns_none
    - test_strategy_fallback_official_only_returns_none
    - test_strategy_fallback_scrapers_only_returns_none
    - test_strategy_fallback_round_robin_returns_health_first
    - test_strategy_serialize_deserialize
}
```

#### Unit Tests for SessionData

```rust
mod session_data_tests {
    // Creation Tests
    - test_session_data_from_cookies_basic
    - test_session_data_from_cookies_multiple
    - test_session_data_from_cookies_empty
    - test_session_data_from_bearer_basic
    - test_session_data_from_bearer_empty
    
    // Type Tests
    - test_session_type_cookies
    - test_session_type_bearer
    - test_session_type_api_key
    - test_session_type_oauth2
    - test_session_type_guest
    
    // Validation Tests
    - test_session_data_validate_empty_cookies_fails
    - test_session_data_validate_valid_succeeds
    - test_session_data_auth_tokens_map
    
    // Serialization Tests
    - test_session_data_serialize_deserialize_roundtrip
}
```

#### Unit Tests for ProxyConfig

```rust
mod proxy_config_tests {
    // URL Parsing Tests
    - test_proxy_config_from_url_http
    - test_proxy_config_from_url_https
    - test_proxy_config_from_url_socks5
    - test_proxy_config_from_url_with_credentials
    - test_proxy_config_from_url_with_port
    - test_proxy_config_from_url_invalid
    
    // Protocol Tests
    - test_proxy_protocol_http
    - test_proxy_protocol_https
    - test_proxy_protocol_socks5
    - test_proxy_protocol_socks4
    
    // Field Tests
    - test_proxy_config_session_id_optional
    - test_proxy_config_parse_url_extracts_host_port
    
    // Serialization Tests
    - test_proxy_config_serialize_deserialize_roundtrip
}
```

#### Unit Tests for PayloadBlob

```rust
mod payload_blob_tests {
    // Creation Tests
    - test_payload_blob_new_json
    - test_payload_blob_new_html
    - test_payload_blob_new_binary
    - test_payload_blob_new_text
    - test_payload_blob_new_unknown
    
    // Text Parsing Tests
    - test_payload_blob_as_text_valid_utf8
    - test_payload_blob_as_text_invalid_utf8
    - test_payload_blob_as_text_empty
    
    // JSON Parsing Tests
    - test_payload_blob_as_json_valid_object
    - test_payload_blob_as_json_valid_array
    - test_payload_blob_as_json_invalid
    - test_payload_blob_as_json_empty_object
    - test_payload_blob_as_json_typed_deserialize
    
    // Field Tests
    - test_payload_blob_source_url_optional
    - test_payload_blob_status_code
    - test_payload_blob_headers_map
    - test_payload_blob_fetched_at_timestamp
    
    // Content Type Tests
    - test_payload_content_type_variants
}
```

#### Unit Tests for RawContext

```rust
mod raw_context_tests {
    // Creation Tests
    - test_raw_context_get_basic
    - test_raw_context_post_basic
    - test_raw_context_post_with_body
    
    // Header Tests
    - test_raw_context_with_header_single
    - test_raw_context_with_header_multiple
    - test_raw_context_with_header_override
    
    // Proxy Tests
    - test_raw_context_with_proxy
    - test_raw_context_without_proxy
    
    // Session Tests
    - test_raw_context_with_session
    - test_raw_context_without_session
    
    // Platform Params Tests
    - test_raw_context_platform_params_default_null
    - test_raw_context_platform_params_custom
    
    // HTTP Method Tests
    - test_http_method_get
    - test_http_method_post
    - test_http_method_put
    - test_http_method_delete
    - test_http_method_patch
    - test_http_method_head
    - test_http_method_options
}
```

### 1.2 Error Module Tests (`src/error.rs`)

```rust
mod ghost_error_tests {
    // Error Creation Tests
    - test_ghost_error_not_implemented
    - test_ghost_error_parse_error
    - test_ghost_error_network_error
    - test_ghost_error_platform_error
    - test_ghost_error_rate_limited
    - test_ghost_error_auth_error
    - test_ghost_error_session_expired
    - test_ghost_error_account_suspended
    - test_ghost_error_proxy_error
    - test_ghost_error_scraper_error
    - test_ghost_error_adapter_error
    - test_ghost_error_health_check_failed
    - test_ghost_error_workers_exhausted
    - test_ghost_error_circuit_breaker_tripped
    - test_ghost_error_budget_exceeded
    - test_ghost_error_config_error
    - test_ghost_error_validation_error
    - test_ghost_error_timeout
    - test_ghost_error_waf_challenge
    - test_ghost_error_io_error
    - test_ghost_error_json_error
    - test_ghost_error_other
    
    // Classification Tests
    - test_ghost_error_is_retryable_network_error
    - test_ghost_error_is_retryable_rate_limited
    - test_ghost_error_is_retryable_timeout
    - test_ghost_error_is_retryable_waf_challenge
    - test_ghost_error_is_retryable_others_false
    - test_ghost_error_is_account_issue_auth_error
    - test_ghost_error_is_account_issue_session_expired
    - test_ghost_error_is_account_issue_suspended
    - test_ghost_error_is_proxy_issue_true
    - test_ghost_error_is_proxy_issue_false
    
    // Retry-After Tests
    - test_ghost_error_retry_after_rate_limited_some
    - test_ghost_error_retry_after_others_none
    
    // Platform Extraction Tests
    - test_ghost_error_platform_platform_error
    - test_ghost_error_platform_rate_limited
    - test_ghost_error_platform_account_suspended
    - test_ghost_error_platform_waf_challenge
    - test_ghost_error_platform_others_none
    
    // From Implementation Tests
    - test_ghost_error_from_io_error
    - test_ghost_error_from_json_error
    
    // Trace Tests
    - test_ghost_error_to_trace_produces_debug_output
    
    // Display Tests
    - test_ghost_error_display_formats_correctly
}
```

### 1.3 Capability Module Tests (`src/capability.rs`)

```rust
mod capability_tests {
    // Capability Enum Tests
    - test_capability_x_read
    - test_capability_x_search
    - test_capability_x_user_read
    - test_capability_x_trending
    - test_capability_x_timeline
    - test_capability_threads_read
    - test_capability_threads_search
    - test_capability_threads_user_read
    - test_capability_threads_timeline
    - test_capability_media_download
    - test_capability_official_api
    - test_capability_browser_based
    - test_capability_request_based
    
    // Platform Mapping Tests
    - test_capability_platform_x_capabilities
    - test_capability_platform_threads_capabilities
    - test_capability_platform_none_for_cross_platform
    
    // Tier Tests
    - test_capability_tier_fast
    - test_capability_tier_heavy
    - test_capability_tier_official
    
    // Manifest Tests
    - test_capability_manifest_new
    - test_capability_manifest_with_capabilities
    - test_capability_manifest_validation
    - test_capability_manifest_serialization
    
    // WorkerType Tests
    - test_worker_type_variants
    - test_worker_type_serialization
}
```

### 1.4 Platform Module Tests (`src/platform.rs`)

```rust
mod platform_tests {
    // Enum Tests
    - test_platform_x
    - test_platform_threads
    - test_platform_unknown
    
    // From Str Tests
    - test_platform_from_str_x
    - test_platform_from_str_twitter
    - test_platform_from_str_threads
    - test_platform_from_str_unknown
    - test_platform_from_str_invalid
    
    // Base URL Tests
    - test_platform_base_url_x
    - test_platform_base_url_threads
    - test_platform_base_url_unknown
    
    // Jitter Range Tests
    - test_platform_jitter_range_x
    - test_platform_jitter_range_threads
    - test_platform_jitter_range_unknown
    
    // Display Tests
    - test_platform_display_x
    - test_platform_display_threads
    
    // Serialization Tests
    - test_platform_serialize_deserialize_roundtrip
}
```

### 1.5 Config Module Tests (`src/config.rs`)

```rust
mod config_tests {
    // GhostConfig Tests
    - test_ghost_config_default
    - test_ghost_config_new
    - test_ghost_config_validation_valid
    - test_ghost_config_validation_invalid_retries
    - test_ghost_config_validation_invalid_timeout
    - test_ghost_config_builder_pattern
    
    // HealthConfig Tests
    - test_health_config_default
    - test_health_config_new
    - test_health_config_validation_valid
    - test_health_config_validation_invalid_threshold_high
    - test_health_config_validation_invalid_threshold_negative
    - test_health_config_threshold_ordering
    
    // ScraperConfig Tests
    - test_scraper_config_default
    - test_scraper_config_enabled
    - test_scraper_config_health_threshold
    
    // PlatformShieldConfig Tests
    - test_platform_shield_config_default
    - test_platform_shield_fingerprint_profiles
    - test_platform_shield_jitter_ranges
    
    // AutoscalingConfig Tests
    - test_autoscaling_config_default
    - test_autoscaling_config_disabled
    - test_autoscaling_config_thresholds
    
    // ConfigBuilder Tests
    - test_config_builder_minimal
    - test_config_builder_full
    - test_config_builder_strategy
    - test_config_builder_timeout
    - test_config_builder_retries
}
```

### 1.6 Event Module Tests (`src/event.rs`)

```rust
mod event_tests {
    // GhostEvent Enum Tests
    - test_ghost_event_worker_registered
    - test_ghost_event_worker_unregistered
    - test_ghost_event_worker_offline
    - test_ghost_event_health_changed
    - test_ghost_event_fallback_triggered
    - test_ghost_event_session_updated
    - test_ghost_event_rate_limited
    - test_ghost_event_circuit_breaker_tripped
    - test_ghost_event_circuit_breaker_reset
    
    // Session Event Tests
    - test_session_unhealthy_reason_suspended
    - test_session_unhealthy_reason_rate_limited
    - test_session_unhealthy_reason_cookie_expired
    
    // Autoscale Event Tests
    - test_autoscale_event_scaling_up
    - test_autoscale_event_scaling_down
    - test_autoscale_event_spot_interrupted
    
    // Serialization Tests
    - test_ghost_event_serialize_deserialize_roundtrip
}
```

### 1.7 Fallback Module Tests (`src/fallback.rs`)

```rust
mod fallback_tests {
    // FallbackContext Tests
    - test_fallback_context_new
    - test_fallback_context_with_attempts
    - test_fallback_context_max_attempts
    
    // FailureReason Tests
    - test_failure_reason_rate_limited_is_retryable
    - test_failure_reason_waf_challenge_is_retryable
    - test_failure_reason_proxy_blocked_is_retryable
    - test_failure_reason_timeout_is_retryable
    - test_failure_reason_session_expired_not_retryable
    - test_failure_reason_not_found_not_retryable
    - test_failure_reason_waf_challenge_requires_escalation
    - test_failure_reason_all_workers_exhausted_requires_escalation
    - test_failure_reason_rate_limited_no_escalation
    
    // FallbackAction Tests
    - test_fallback_action_retry_same_tier
    - test_fallback_action_escalate_tier
    - test_fallback_action_pivot_worker
    - test_fallback_action_abort
    
    // CapabilityTier Tests
    - test_capability_tier_fast_fallback
    - test_capability_tier_heavy_fallback
    - test_capability_tier_official_fallback_none
    - test_capability_tier_ordering
    
    // FallbackTracker Tests
    - test_fallback_tracker_new
    - test_fallback_tracker_record_step
    - test_fallback_tracker_max_steps
    - test_fallback_tracker_should_abort
}
```

### 1.8 Vault Module Tests (`src/vault.rs`)

```rust
mod vault_types_tests {
    // VaultProviderType Tests
    - test_vault_provider_type_memory
    - test_vault_provider_type_file
    
    // VaultConfig Tests
    - test_vault_config_default
    - test_vault_config_memory
    - test_vault_config_file
    - test_vault_config_cache_ttl
    - test_vault_config_validation_file_missing_path
    
    // ProxyEntry Tests
    - test_proxy_entry_new
    - test_proxy_entry_with_credentials
    - test_proxy_entry_health_tracking
    
    // CredentialEntry Tests
    - test_credential_entry_new
    - test_credential_entry_status
    - test_credential_entry_last_used
    
    // CachedSecret Tests
    - test_cached_secret_new
    - test_cached_secret_is_expired_false_initially
    - test_cached_secret_is_expired_true_after_ttl
    - test_cached_secret_value_access
}
```

### 1.9 Worker Module Tests (`src/worker.rs`)

```rust
mod worker_types_tests {
    // WorkerHealth Tests
    - test_worker_health_new
    - test_worker_health_default_score
    - test_worker_health_record_success
    - test_worker_health_record_failure
    - test_worker_health_consecutive_failures_reset_on_success
    
    // WorkerStatus Tests
    - test_worker_status_idle
    - test_worker_status_busy
    - test_worker_status_offline
    - test_worker_status_error
    
    // WorkerStats Tests
    - test_worker_stats_new
    - test_worker_stats_record_success
    - test_worker_stats_record_failure
    - test_worker_stats_total_requests
    - test_worker_stats_success_rate
    
    // CircuitBreaker Tests
    - test_circuit_breaker_new
    - test_circuit_breaker_trip
    - test_circuit_breaker_reset
    - test_circuit_breaker_is_open
    - test_circuit_breaker_half_open_state
    - test_circuit_breaker_probe_success
    - test_circuit_breaker_probe_failure
    
    // WorkerSelection Tests
    - test_worker_selection_new
    - test_worker_selection_tier
}
```

### 1.10 Adapter Types Module Tests (`src/adapter_types.rs`)

```rust
mod adapter_types_tests {
    // AdapterParseResult Tests
    - test_adapter_parse_result_new
    - test_adapter_parse_result_with_post
    - test_adapter_parse_result_with_user
    - test_adapter_parse_result_with_posts
    - test_adapter_parse_result_with_error
    - test_adapter_parse_result_source
    - test_adapter_parse_result_cursor
    - test_adapter_parse_result_into_posts
    - test_adapter_parse_result_into_user
    
    // AdapterError Tests
    - test_adapter_error_rate_limited
    - test_adapter_error_account_suspended
    - test_adapter_error_not_found
    - test_adapter_error_protected_account
    - test_adapter_error_login_required
    - test_adapter_error_suspicious_activity
    - test_adapter_error_parse_error
    
    // X-Specific Types Tests
    - test_x_tweet_data_deserialize
    - test_x_user_data_deserialize
    - test_x_tweet_metrics
    - test_x_user_metrics
    - test_x_media_data
    - test_x_error_variants
    
    // Threads-Specific Types Tests
    - test_threads_post_type
    - test_threads_auth
    - test_threads_media_container
    - test_threads_user_response
    - test_threads_error_variants
    
    // TrendingTopic Tests
    - test_trending_topic_new
    - test_trending_topic_with_volume
    
    // Entity Types Tests
    - test_hashtag_entity
    - test_cashtag_entity
    - test_user_mention
    - test_url_entity
    - test_coordinates
    - test_place
}
```

---

## 2. ghost-core Testing Plan

### 2.1 Ghost Module Tests (`src/ghost.rs`)

```rust
mod ghost_tests {
    // Initialization Tests
    - test_ghost_init_default_succeeds
    - test_ghost_init_with_config_succeeds
    - test_ghost_init_validates_config
    - test_ghost_init_with_invalid_config_fails
    
    // Platform Client Tests
    - test_ghost_x_client_platform
    - test_ghost_threads_client_platform
    - test_ghost_platform_client_not_connected
    
    // Worker Management Tests
    - test_ghost_register_worker_succeeds
    - test_ghost_register_worker_emits_event
    - test_ghost_register_duplicate_worker_fails
    - test_ghost_unregister_worker_succeeds
    - test_ghost_unregister_worker_emits_event
    - test_ghost_unregister_nonexistent_worker_fails
    - test_ghost_worker_count_accurate
    
    // Health Status Tests
    - test_ghost_health_status_empty
    - test_ghost_health_status_with_workers
    - test_ghost_check_health_delegates_to_engine
    - test_ghost_is_platform_supported_true_with_worker
    - test_ghost_is_platform_supported_false_empty
    
    // Capabilities Tests
    - test_ghost_capabilities_for_platform_empty
    - test_ghost_capabilities_for_platform_with_workers
    
    // Event Tests
    - test_ghost_events_subscriber_receives_events
    - test_ghost_events_broadcasts_to_multiple_subscribers
    
    // Configuration Tests
    - test_ghost_config_returns_configuration
    - test_ghost_router_returns_router
    - test_ghost_health_engine_returns_engine
    
    // Shutdown Tests
    - test_ghost_shutdown_succeeds
    - test_ghost_shutdown_emits_offline_events
    - test_ghost_shutdown_clears_workers
    
    // Drop Tests
    - test_ghost_drop_logs_debug_message
    
    // Concurrency Tests
    - test_ghost_concurrent_worker_registration
    - test_ghost_concurrent_worker_unregistration
    - test_ghost_concurrent_health_checks
}
```

### 2.2 Router Module Tests (`src/router.rs`)

```rust
mod router_tests {
    // Capability Mapping Tests
    - test_get_post_capability_x
    - test_get_post_capability_threads
    - test_get_post_capability_unknown_fails
    - test_get_user_capability_x
    - test_get_user_capability_threads
    - test_get_search_capability_x
    - test_get_search_capability_threads
    - test_get_trending_capability_x
    - test_get_timeline_capability_x
    
    // Worker Selection Tests
    - test_select_worker_health_first_chooses_best_score
    - test_select_worker_fastest_chooses_lowest_latency
    - test_select_worker_cost_optimized_chooses_cheapest
    - test_select_worker_round_robin_cycles
    - test_select_worker_official_first_prefers_official
    - test_select_worker_official_only_filters_scrapers
    - test_select_worker_scrapers_only_filters_official
    - test_select_worker_excludes_failed_workers
    - test_select_worker_no_candidates_returns_error
    
    // Route Execution Tests
    - test_route_get_post_executes_worker
    - test_route_get_post_records_success
    - test_route_get_post_records_failure
    - test_route_get_user_executes_worker
    - test_route_search_executes_worker
    - test_route_trending_executes_worker
    - test_route_timeline_executes_worker
    
    // Fallback Tests
    - test_execute_with_fallback_retries_on_failure
    - test_execute_with_fallback_respects_max_retries
    - test_execute_with_fallback_excludes_failed_workers
    - test_execute_with_fallback_applies_jitter
    
    // Context Building Tests
    - test_build_raw_context_includes_url
    - test_build_raw_context_includes_proxy
    - test_build_raw_context_includes_session
    - test_build_raw_context_includes_shield_headers
    
    // Parsing Tests
    - test_parse_post_extracts_id_text
    - test_parse_post_extracts_counts
    - test_parse_user_extracts_id_username
    - test_parse_user_extracts_profile_data
    - test_parse_posts_handles_array
    - test_parse_posts_handles_wrapped_response
    
    // Integration Tests
    - test_router_full_request_flow
    - test_router_multiple_workers_different_platforms
    - test_router_worker_failure_triggers_fallback
}
```

### 2.3 Health Engine Tests (`src/health.rs`)

```rust
mod health_engine_tests {
    // Rolling Window Tests
    - test_rolling_window_new
    - test_rolling_window_record_success
    - test_rolling_window_record_failure
    - test_rolling_window_record_overflow
    - test_rolling_window_success_rate_empty
    - test_rolling_window_success_rate_all_success
    - test_rolling_window_success_rate_mixed
    - test_rolling_window_avg_latency_empty
    - test_rolling_window_avg_latency_values
    - test_rolling_window_p95_latency_empty
    - test_rolling_window_p95_latency_values
    - test_rolling_window_p95_latency_single
    
    // Health Engine Initialization Tests
    - test_health_engine_new
    - test_health_engine_initialize_worker
    - test_health_engine_remove_worker
    - test_health_engine_get_health_uninitialized
    
    // Health Score Tests
    - test_health_engine_calculate_score_perfect
    - test_health_engine_calculate_score_success_rate_dominant
    - test_health_engine_calculate_score_latency_dominant
    - test_health_engine_calculate_score_zero_success
    - test_health_engine_calculate_score_max_latency
    
    // Success Recording Tests
    - test_health_engine_record_success_updates_score
    - test_health_engine_record_success_updates_latency
    - test_health_engine_record_success_resets_failures
    - test_health_engine_record_success_updates_stats
    
    // Failure Recording Tests
    - test_health_engine_record_failure_increments_failures
    - test_health_engine_record_failure_decreases_score
    - test_health_engine_record_failure_trips_breaker
    
    // Circuit Breaker Tests
    - test_health_engine_trip_circuit_breaker
    - test_health_engine_reset_circuit_breaker
    - test_health_engine_is_circuit_open_true
    - test_health_engine_is_circuit_open_false
    
    // Worker Classification Tests
    - test_health_engine_healthy_workers
    - test_health_engine_degraded_workers
    - test_health_engine_unhealthy_workers
    - test_health_engine_classification_by_threshold
    
    // Aggregation Tests
    - test_health_engine_aggregate_status_empty
    - test_health_engine_aggregate_status_with_workers
    - test_health_engine_aggregate_status_by_platform
    
    // Stats Tests
    - test_health_engine_get_stats
    - test_health_engine_get_detailed_stats
    - test_health_engine_get_top_workers
    
    // Configuration Tests
    - test_health_engine_config_returns_config
}
```

### 2.4 Worker Registry Tests (`src/worker.rs`)

```rust
mod worker_registry_tests {
    // Registry Creation Tests
    - test_registry_new
    - test_registry_default
    - test_registry_is_empty
    - test_registry_len
    
    // Registration Tests
    - test_registry_register_single_worker
    - test_registry_register_multiple_workers
    - test_registry_register_updates_indices
    
    // Lookup Tests
    - test_registry_get_by_id
    - test_registry_get_nonexistent
    - test_registry_get_by_capability
    - test_registry_get_by_platform
    - test_registry_get_by_capability_and_platform
    - test_registry_get_ids_by_capability
    - test_registry_get_ids_by_capability_and_platform
    - test_registry_worker_ids
    
    // Sorting Tests
    - test_registry_get_by_capability_sorted
    - test_registry_get_round_robin_cycles
    
    // Unregistration Tests
    - test_registry_unregister_existing
    - test_registry_unregister_nonexistent
    - test_registry_unregister_updates_indices
    
    // Clear Tests
    - test_registry_clear
    - test_registry_clear_updates_indices
    
    // Index Integrity Tests
    - test_registry_capability_index_updated
    - test_registry_platform_index_updated
    
    // GhostWorker Trait Tests
    - test_ghost_worker_id
    - test_ghost_worker_capabilities
    - test_ghost_worker_platforms
    - test_ghost_worker_execute
    - test_ghost_worker_manifest
    - test_ghost_worker_status
    - test_ghost_worker_load
    - test_ghost_worker_worker_type
    - test_ghost_worker_priority
    - test_ghost_worker_health_check
}
```

### 2.5 Fallback Engine Tests (`src/fallback.rs`)

```rust
mod fallback_engine_tests {
    // Creation Tests
    - test_fallback_engine_new
    - test_fallback_engine_default
    - test_fallback_engine_total_fallbacks_initial
    
    // Tier Escalation Tests
    - test_fallback_engine_next_tier_fast_to_heavy
    - test_fallback_engine_next_tier_heavy_to_official
    - test_fallback_engine_next_tier_official_none
    
    // Action Determination Tests
    - test_fallback_engine_determine_action_retryable
    - test_fallback_engine_determine_action_escalation
    - test_fallback_engine_determine_action_abort
    
    // Tracker Tests
    - test_fallback_engine_tracker_new
    - test_fallback_engine_record_fallback
    - test_fallback_engine_should_abort_true
    
    // Configuration Tests
    - test_fallback_engine_config
}
```

### 2.6 Event Module Tests (`src/events.rs`)

```rust
mod events_tests {
    // Event Broadcast Tests
    - test_event_broadcast_single_subscriber
    - test_event_broadcast_multiple_subscribers
    - test_event_broadcast_no_subscribers_no_error
    
    // Event Channel Tests
    - test_event_channel_capacity
    - test_event_channel_late_subscriber_misses_events
}
```

### 2.7 Configuration Module Tests (`src/config.rs`)

```rust
mod config_tests {
    // GhostConfig Tests
    - test_ghost_config_default
    - test_ghost_config_new
    - test_ghost_config_validate_success
    - test_ghost_config_validate_max_retries_zero_fails
    - test_ghost_config_validate_timeout_zero_fails
    - test_ghost_config_validate_all_success
    
    // ConfigBuilder Tests
    - test_config_builder_default
    - test_config_builder_strategy
    - test_config_builder_max_retries
    - test_config_builder_timeout
    - test_config_builder_shields
    - test_config_builder_build
    
    // Shield Config Tests
    - test_platform_shield_config_default
    - test_platform_shield_config_validation
    
    // Config Extension Tests
    - test_ghost_config_ext_load_toml
    - test_ghost_config_ext_load_toml_missing_file
    - test_ghost_config_ext_load_toml_invalid
}
```

### 2.8 Integration Tests

```rust
mod integration_tests {
    // Full Workflow Tests
    - test_ghost_full_workflow_init_register_shutdown
    - test_ghost_request_routing_with_real_worker
    - test_ghost_health_scoring_integration
    - test_ghost_fallback_integration
    - test_ghost_multiple_platforms_integration
    
    // Concurrency Integration Tests
    - test_ghost_concurrent_requests_different_workers
    - test_ghost_concurrent_requests_same_worker_queued
    - test_ghost_worker_registration_during_requests
    
    // Error Recovery Integration Tests
    - test_ghost_worker_failure_recovery
    - test_ghost_circuit_breaker_integration
    - test_ghost_rate_limit_handling
}
```

---

## 3. ghost-vault Testing Plan

### 3.1 Vault Module Tests (`src/vault.rs`)

```rust
mod vault_tests {
    // MemoryVault Tests
    - test_memory_vault_new
    - test_memory_vault_with_capacity
    - test_memory_vault_with_secrets
    - test_memory_vault_add_secret
    - test_memory_vault_len
    - test_memory_vault_is_empty
    - test_memory_vault_get_secret_existing
    - test_memory_vault_get_secret_nonexistent
    - test_memory_vault_list_secrets_prefix
    - test_memory_vault_list_secrets_all
    - test_memory_vault_provider_name
    - test_memory_vault_provider_type
    
    // AsyncMemoryVault Tests
    - test_async_memory_vault_new
    - test_async_memory_vault_with_secrets
    - test_async_memory_vault_put_secret
    - test_async_memory_vault_get_secret
    - test_async_memory_vault_delete_secret
    - test_async_memory_vault_delete_nonexistent
    - test_async_memory_vault_list_secrets
    - test_async_memory_vault_concurrent_access
    
    // FileVault Tests
    - test_file_vault_new
    - test_file_vault_open_existing
    - test_file_vault_open_nonexistent_creates_empty
    - test_file_vault_load
    - test_file_vault_save
    - test_file_vault_save_if_dirty
    - test_file_vault_ensure_exists
    - test_file_vault_add_secret_marks_dirty
    - test_file_vault_remove_secret
    - test_file_vault_is_dirty
    - test_file_vault_file_path
    - test_file_vault_get_secret
    - test_file_vault_list_secrets
    - test_file_vault_persistence_roundtrip
    
    // VaultManager Tests
    - test_vault_manager_new
    - test_vault_manager_memory
    - test_vault_manager_file
    - test_vault_manager_async_memory
    - test_vault_manager_get_uncached
    - test_vault_manager_get_caches_result
    - test_vault_manager_get_expired_refetches
    - test_vault_manager_put
    - test_vault_manager_delete
    - test_vault_manager_list
    - test_vault_manager_list_all
    - test_vault_manager_invalidate
    - test_vault_manager_clear_cache
    - test_vault_manager_cache_size
    - test_vault_manager_provider_name
    - test_vault_manager_provider_type
    - test_vault_manager_config
    - test_vault_manager_preload
    
    // Factory Function Tests
    - test_create_vault_provider_memory
    - test_create_vault_provider_file
    - test_create_vault_provider_file_missing_path
    - test_create_vault_manager
}
```

### 3.2 Credential Store Tests (`src/credential.rs`)

```rust
mod credential_store_tests {
    // CredentialStore Creation Tests
    - test_credential_store_new
    - test_credential_store_default
    - test_credential_store_with_capacity
    
    // Credential Management Tests
    - test_credential_store_add_credential
    - test_credential_store_add_credential_duplicate_updates
    - test_credential_store_get_by_id
    - test_credential_store_get_by_id_nonexistent
    - test_credential_store_get_by_tenant
    - test_credential_store_get_by_platform
    - test_credential_store_remove_credential
    - test_credential_store_remove_nonexistent
    - test_credential_store_clear
    
    // Status Tracking Tests
    - test_credential_store_update_status
    - test_credential_store_get_active_count
    - test_credential_store_get_expired_count
    
    // Index Tests
    - test_credential_store_tenant_index_updated
    - test_credential_store_platform_index_updated
    
    // Stats Tests
    - test_credential_store_stats
    - test_credential_store_len
    - test_credential_store_is_empty
    
    // CredentialEntry Tests
    - test_credential_entry_new
    - test_credential_entry_update_last_used
    - test_credential_entry_mark_expired
    - test_credential_entry_is_valid
    
    // Concurrency Tests
    - test_credential_store_concurrent_add
    - test_credential_store_concurrent_get
}
```

### 3.3 Proxy Pool Tests (`src/proxy.rs`)

```rust
mod proxy_pool_tests {
    // ProxyPool Creation Tests
    - test_proxy_pool_new
    - test_proxy_pool_from_urls
    - test_proxy_pool_from_urls_empty
    - test_proxy_pool_default
    
    // Proxy Registration Tests
    - test_proxy_pool_add_proxy
    - test_proxy_pool_add_proxy_duplicate
    - test_proxy_pool_remove_proxy
    - test_proxy_pool_clear
    
    // Proxy Selection Tests
    - test_proxy_pool_get_round_robin
    - test_proxy_pool_get_least_used
    - test_proxy_pool_get_random
    - test_proxy_pool_get_sticky_session
    - test_proxy_pool_get_empty_pool
    
    // Health Tracking Tests
    - test_proxy_pool_record_success
    - test_proxy_pool_record_failure
    - test_proxy_pool_mark_unhealthy
    - test_proxy_pool_get_healthy_count
    - test_proxy_pool_get_unhealthy_proxies
    
    // Rotation Tests
    - test_proxy_pool_rotation_cycle
    - test_proxy_pool_exclude_failing
    
    // Stats Tests
    - test_proxy_pool_len
    - test_proxy_pool_is_empty
    - test_proxy_pool_stats
    
    // ProxyEntry Tests
    - test_proxy_entry_new
    - test_proxy_entry_from_url_http
    - test_proxy_entry_from_url_socks5
    - test_proxy_entry_from_url_with_auth
    - test_proxy_entry_is_healthy_initially
    - test_proxy_entry_record_success
    - test_proxy_entry_record_failure
    
    // Concurrency Tests
    - test_proxy_pool_concurrent_selection
    - test_proxy_pool_concurrent_health_updates
}
```

### 3.4 Session Manager Tests (`src/session.rs`)

```rust
mod session_manager_tests {
    // SessionManager Creation Tests
    - test_session_manager_new
    - test_session_manager_default
    
    // Session Management Tests
    - test_session_manager_add_session
    - test_session_manager_add_session_duplicate
    - test_session_manager_remove_session
    - test_session_manager_get_session
    - test_session_manager_get_by_tenant
    - test_session_manager_get_by_platform
    - test_session_manager_clear
    
    // Health Check Tests
    - test_session_manager_check_session_health
    - test_session_manager_check_all_health
    - test_session_manager_mark_unhealthy
    - test_session_manager_get_healthy_sessions
    
    // Session Expiry Tests
    - test_session_manager_detect_expired
    - test_session_manager_cleanup_expired
    
    // Event Tests
    - test_session_manager_emit_unhealthy_event
    - test_session_manager_emit_recovered_event
    
    // Stats Tests
    - test_session_manager_stats
    - test_session_manager_len
    
    // SessionHealthChecker Tests
    - test_session_health_checker_new
    - test_session_health_checker_config
    - test_session_health_checker_check_endpoint
    
    // Concurrency Tests
    - test_session_manager_concurrent_access
}
```

### 3.5 Context Injector Tests (`src/injection.rs`)

```rust
mod context_injector_tests {
    // ContextInjector Creation Tests
    - test_context_injector_new
    - test_context_injector_default
    - test_context_injector_builder
    
    // Builder Tests
    - test_context_injector_builder_with_proxy_pool
    - test_context_injector_builder_with_credential_store
    - test_context_injector_builder_with_session_manager
    - test_context_injector_builder_with_default_proxy
    - test_context_injector_builder_with_default_session
    
    // Injection Tests
    - test_context_injector_inject_basic
    - test_context_injector_inject_with_proxy
    - test_context_injector_inject_with_session
    - test_context_injector_inject_with_credentials
    - test_context_injector_inject_full
    - test_context_injector_inject_tenant_not_found
    - test_context_injector_inject_no_resources
    
    // Override Tests
    - test_context_injector_override_proxy
    - test_context_injector_override_session
    - test_context_injector_override_credentials
    
    // Options Tests
    - test_injection_options_default
    - test_injection_options_require_proxy
    - test_injection_options_require_session
    - test_injection_options_prefer_cached
    
    // Result Tests
    - test_injection_result_success
    - test_injection_result_missing_proxy
    - test_injection_result_missing_session
    
    // Middleware Tests
    - test_injection_middleware_new
    - test_injection_middleware_inject_context
    
    // Integration Tests
    - test_context_injector_full_workflow
    - test_context_injector_multiple_tenants
}
```

---

## 4. ghost-bridge Testing Plan

### 4.1 Bridge Module Tests (`src/bridge.rs`)

```rust
mod bridge_tests {
    // BridgeManager Tests
    - test_bridge_manager_new
    - test_bridge_manager_default
    - test_bridge_manager_add_bridge
    - test_bridge_manager_add_multiple_bridges
    - test_bridge_manager_len
    - test_bridge_manager_is_empty
    - test_bridge_manager_get_by_index
    - test_bridge_manager_get_by_type
    - test_bridge_manager_remove
    - test_bridge_manager_clear
    
    // Bridge Initialization Tests
    - test_bridge_manager_initialize_all_success
    - test_bridge_manager_initialize_all_partial_failure
    - test_bridge_manager_initialize_all_all_failure
    - test_bridge_manager_initialize_empty
    
    // Bridge Shutdown Tests
    - test_bridge_manager_shutdown_all_success
    - test_bridge_manager_shutdown_all_partial_failure
    - test_bridge_manager_shutdown_all_all_failure
    
    // Health Status Tests
    - test_bridge_manager_health_status
    - test_bridge_manager_healthy_count
    - test_bridge_manager_healthy_count_all_unhealthy
    
    // Stats Tests
    - test_bridge_manager_stats
    - test_bridge_manager_aggregate_stats
    - test_bridge_manager_aggregate_stats_empty
    
    // Factory Function Tests
    - test_create_bridge_native_error
    - test_create_bridge_grpc_not_implemented
    - test_create_bridge_uds_not_implemented
    - test_create_default_bridge_manager
    
    // Bridge Trait Tests
    - test_bridge_bridge_type
    - test_bridge_initialize
    - test_bridge_shutdown
    - test_bridge_is_healthy
    - test_bridge_stats
}
```

### 4.2 Protocol Handler Tests (`src/protocol.rs`)

```rust
mod protocol_tests {
    // ProtocolHandler Creation Tests
    - test_protocol_handler_new
    - test_protocol_handler_default
    - test_protocol_handler_with_format
    - test_protocol_handler_with_protocol
    
    // Serialization Tests
    - test_protocol_handler_serialize_json
    - test_protocol_handler_deserialize_json
    - test_protocol_handler_serialize_msgpack
    - test_protocol_handler_deserialize_msgpack
    
    // Message Building Tests
    - test_protocol_handler_build_request
    - test_protocol_handler_build_response
    - test_protocol_handler_build_health_check
    - test_protocol_handler_build_manifest
    
    // Message Parsing Tests
    - test_protocol_handler_parse_request
    - test_protocol_handler_parse_response
    - test_protocol_handler_parse_health_check
    - test_protocol_handler_parse_invalid
    
    // ProtocolBuilder Tests
    - test_protocol_builder_new
    - test_protocol_builder_json
    - test_protocol_builder_msgpack
    - test_protocol_builder_build
    
    // MessageEnvelope Tests
    - test_message_envelope_new
    - test_message_envelope_message_type
    - test_message_envelope_serialize
    
    // WorkerRequest/Response Tests
    - test_worker_request_new
    - test_worker_request_with_context
    - test_worker_response_success
    - test_worker_response_failure
    
    // Health Check Message Tests
    - test_health_check_message_new
    - test_health_check_response_healthy
    - test_health_check_response_unhealthy
}
```

### 4.3 Worker Pool Tests (`src/worker.rs`)

```rust
mod worker_pool_tests {
    // WorkerPool Creation Tests
    - test_worker_pool_new
    - test_worker_pool_default
    - test_worker_pool_with_capacity
    - test_worker_pool_is_empty
    - test_worker_pool_len
    
    // Worker Management Tests
    - test_worker_pool_add_worker
    - test_worker_pool_add_worker_duplicate
    - test_worker_pool_remove_worker
    - test_worker_pool_get_worker
    - test_worker_pool_get_by_capability
    - test_worker_pool_clear
    
    // Worker Selection Tests
    - test_worker_pool_select_best
    - test_worker_pool_select_round_robin
    - test_worker_pool_select_any
    - test_worker_pool_select_empty_pool
    
    // BridgeWorker Tests
    - test_bridge_worker_new
    - test_bridge_worker_id
    - test_bridge_worker_capabilities
    - test_bridge_worker_execute
    - test_bridge_worker_health_check
    - test_bridge_worker_manifest
    
    // WorkerFactory Tests
    - test_worker_factory_new
    - test_worker_factory_create_worker
    - test_worker_factory_register_bridge
    - test_worker_factory_available_types
    
    // Concurrency Tests
    - test_worker_pool_concurrent_add
    - test_worker_pool_concurrent_select
    - test_worker_pool_concurrent_execute
}
```

### 4.4 Error Tests (`src/error.rs`)

```rust
mod bridge_error_tests {
    // BridgeError Creation Tests
    - test_bridge_error_timeout
    - test_bridge_error_connection_failed
    - test_bridge_error_worker_error
    - test_bridge_error_protocol_error
    - test_bridge_error_initialization_failed
    - test_bridge_error_shutdown_failed
    - test_bridge_error_not_initialized
    
    // Classification Tests
    - test_bridge_error_is_recoverable_timeout
    - test_bridge_error_is_recoverable_connection_failed
    - test_bridge_error_is_recoverable_worker_error
    - test_bridge_error_is_recoverable_protocol_error
    - test_bridge_error_is_recoverable_initialization_failed
    
    // Display Tests
    - test_bridge_error_display
}
```

---

## 5. x-adapter Testing Plan

### 5.1 Adapter Tests (`src/adapter.rs`)

```rust
mod x_adapter_tests {
    // XAdapter Creation Tests
    - test_x_adapter_new
    - test_x_adapter_default
    - test_x_adapter_platform
    
    // Parse Method Tests
    - test_x_adapter_parse_json_tweet
    - test_x_adapter_parse_json_user
    - test_x_adapter_parse_json_timeline
    - test_x_adapter_parse_html_unsupported
    - test_x_adapter_parse_unsupported_content_type
    
    // JSON Parsing Tests
    - test_x_adapter_parse_json_graphql_tweet
    - test_x_adapter_parse_json_graphql_user
    - test_x_adapter_parse_json_api_v2_tweet
    - test_x_adapter_parse_json_array
    - test_x_adapter_parse_json_single_tweet
    - test_x_adapter_parse_json_user_object
    - test_x_adapter_parse_json_unknown_format
    
    // Data Response Tests
    - test_x_adapter_parse_data_response_tweet_result
    - test_x_adapter_parse_data_response_user_result
    - test_x_adapter_parse_data_response_timeline
    - test_x_adapter_parse_data_response_direct_tweet
    - test_x_adapter_parse_data_response_direct_user
    - test_x_adapter_parse_data_response_unknown
    
    // Tweet Response Tests
    - test_x_adapter_parse_tweet_response
    - test_x_adapter_parse_tweet_response_with_includes
    - test_x_adapter_parse_tweet_response_with_errors
    - test_x_adapter_parse_tweet_response_no_data
    
    // Tweet Data Tests
    - test_x_adapter_parse_tweet_data_basic
    - test_x_adapter_parse_tweet_data_with_author
    - test_x_adapter_parse_tweet_data_with_metrics
    - test_x_adapter_parse_tweet_data_with_reply
    
    // User Data Tests
    - test_x_adapter_parse_user_data_basic
    - test_x_adapter_parse_user_data_with_metrics
    - test_x_adapter_parse_user_data_with_verified
    
    // Timeline Tests
    - test_x_adapter_parse_timeline_instructions
    - test_x_adapter_parse_timeline_with_cursors
    - test_x_adapter_parse_timeline_empty
    
    // Trending Tests
    - test_x_adapter_parse_trending_basic
    - test_x_adapter_parse_trending_empty
    
    // Search Tests
    - test_x_adapter_parse_search_basic
    - test_x_adapter_parse_search_empty
    
    // Error Detection Tests
    - test_x_adapter_detect_error_explicit
    - test_x_adapter_detect_error_rate_limit
    - test_x_adapter_detect_error_login_required
    - test_x_adapter_detect_error_not_found
    - test_x_adapter_detect_error_none
    
    // Error Mapping Tests
    - test_x_adapter_map_x_error_rate_limited
    - test_x_adapter_map_x_error_account_suspended
    - test_x_adapter_map_x_error_not_found
    - test_x_adapter_map_x_error_protected_account
    - test_x_adapter_map_x_error_login_required
    - test_x_adapter_map_x_error_suspicious_activity
    - test_x_adapter_map_x_error_parse_error
    
    // Integration Tests with Real JSON
    - test_x_adapter_parse_real_tweet_response
    - test_x_adapter_parse_real_user_response
    - test_x_adapter_parse_real_timeline_response
    - test_x_adapter_parse_real_search_response
}
```

### 5.2 Parser Tests (`src/parser.rs`)

```rust
mod x_parser_tests {
    // PostParser Tests
    - test_post_parser_new
    - test_post_parser_parse_basic_tweet
    - test_post_parser_parse_tweet_with_media
    - test_post_parser_parse_tweet_with_entities
    - test_post_parser_parse_tweet_with_reply
    - test_post_parser_parse_tweet_with_quote
    - test_post_parser_parse_tweet_with_metrics
    - test_post_parser_parse_legacy_format
    - test_post_parser_parse_graphql_format
    - test_post_parser_parse_invalid_data
    
    // UserParser Tests
    - test_user_parser_new
    - test_user_parser_parse_basic_user
    - test_user_parser_parse_user_with_metrics
    - test_user_parser_parse_user_with_description
    - test_user_parser_parse_user_with_location
    - test_user_parser_parse_user_with_url
    - test_user_parser_parse_user_verified
    - test_user_parser_parse_user_protected
    - test_user_parser_parse_legacy_format
    - test_user_parser_parse_graphql_format
    - test_user_parser_parse_invalid_data
}
```

### 5.3 Selector Tests (`src/selectors.rs`)

```rust
mod x_selector_tests {
    // XSelectors Tests
    - test_x_selectors_new
    - test_x_selectors_tweet_container
    - test_x_selectors_tweet_text
    - test_x_selectors_tweet_username
    - test_x_selectors_tweet_timestamp
    - test_x_selectors_tweet_metrics
    - test_x_selectors_user_container
    - test_x_selectors_user_username
    - test_x_selectors_user_display_name
    - test_x_selectors_user_description
    - test_x_selectors_user_metrics
    - test_x_selectors_timeline_item
    - test_x_selectors_search_result
}
```

### 5.4 GraphQL Tests (`src/graphql.rs`)

```rust
mod x_graphql_tests {
    // GraphQLQueries Tests
    - test_graphql_queries_new
    - test_graphql_queries_tweet_by_id
    - test_graphql_queries_user_by_screen_name
    - test_graphql_queries_user_tweets
    - test_graphql_queries_search
    - test_graphql_queries_trending
    - test_graphql_queries_build_query_params
    
    // GraphQLFeatures Tests
    - test_graphql_features_default
    - test_graphql_features_tweet_features
    - test_graphql_features_user_features
    - test_graphql_features_to_json
}
```

---

## 6. threads-adapter Testing Plan

### 6.1 Adapter Tests (`src/adapter.rs`)

```rust
mod threads_adapter_tests {
    // ThreadsAdapter Creation Tests
    - test_threads_adapter_new
    - test_threads_adapter_default
    - test_threads_adapter_platform
    
    // Parse Method Tests
    - test_threads_adapter_parse_json_post
    - test_threads_adapter_parse_json_user
    - test_threads_adapter_parse_json_thread
    - test_threads_adapter_parse_unsupported_content_type
    
    // JSON Parsing Tests
    - test_threads_adapter_parse_json_relay_post
    - test_threads_adapter_parse_json_relay_user
    - test_threads_adapter_parse_json_scraper_array
    - test_threads_adapter_parse_json_unknown_format
    
    // Data Parsing Tests
    - test_threads_adapter_parse_data_thread
    - test_threads_adapter_parse_data_user
    - test_threads_adapter_parse_data_timeline
    - test_threads_adapter_parse_data_container
    - test_threads_adapter_parse_data_single_post
    - test_threads_adapter_parse_data_single_user
    - test_threads_adapter_parse_data_array
    - test_threads_adapter_parse_data_unknown
    
    // Thread Parsing Tests
    - test_threads_adapter_parse_thread_internal
    - test_threads_adapter_parse_thread_with_items
    - test_threads_adapter_parse_thread_empty
    
    // Timeline Parsing Tests
    - test_threads_adapter_parse_timeline_internal
    - test_threads_adapter_parse_timeline_with_items
    - test_threads_adapter_parse_timeline_empty
    
    // Public API Tests
    - test_threads_adapter_parse_post
    - test_threads_adapter_parse_user
    - test_threads_adapter_parse_thread
    - test_threads_adapter_parse_search
    - test_threads_adapter_parse_timeline
    
    // Error Detection Tests
    - test_threads_adapter_detect_error_explicit
    - test_threads_adapter_detect_error_array
    - test_threads_adapter_detect_error_rate_limit
    - test_threads_adapter_detect_error_none
    
    // Error Mapping Tests
    - test_threads_adapter_map_threads_error_rate_limited
    - test_threads_adapter_map_threads_error_account_suspended
    - test_threads_adapter_map_threads_error_not_found
    - test_threads_adapter_map_threads_error_private_account
    - test_threads_adapter_map_threads_error_login_required
    - test_threads_adapter_map_threads_error_checkpoint
    - test_threads_adapter_map_threads_error_parse_error
    
    // Integration Tests with Real JSON
    - test_threads_adapter_parse_real_post_response
    - test_threads_adapter_parse_real_user_response
    - test_threads_adapter_parse_real_thread_response
    - test_threads_adapter_parse_real_timeline_response
}
```

### 6.2 Parser Tests (`src/parser.rs`)

```rust
mod threads_parser_tests {
    // PostParser Tests
    - test_post_parser_new
    - test_post_parser_parse_basic_post
    - test_post_parser_parse_post_with_media
    - test_post_parser_parse_post_with_reply
    - test_post_parser_parse_post_with_quote
    - test_post_parser_parse_post_with_metrics
    - test_post_parser_parse_relay_format
    - test_post_parser_parse_scraper_format
    - test_post_parser_parse_invalid_data
    
    // UserParser Tests
    - test_user_parser_new
    - test_user_parser_parse_basic_user
    - test_user_parser_parse_user_with_metrics
    - test_user_parser_parse_user_with_bio
    - test_user_parser_parse_user_verified
    - test_user_parser_parse_user_private
    - test_user_parser_parse_relay_format
    - test_user_parser_parse_scraper_format
    - test_user_parser_parse_invalid_data
}
```

### 6.3 Relay Tests (`src/relay.rs`)

```rust
mod relay_tests {
    // RelayResponse Tests
    - test_relay_response_from_json
    - test_relay_response_from_json_invalid
    - test_relay_response_has_errors_true
    - test_relay_response_has_errors_false
    - test_relay_response_extract_data
    - test_relay_response_extract_data_null
    
    // ThreadsQueries Tests
    - test_threads_queries_new
    - test_threads_queries_post_by_id
    - test_threads_queries_user_by_username
    - test_threads_queries_user_threads
    - test_threads_queries_build_query
    
    // ThreadsHeaders Tests
    - test_threads_headers_default
    - test_threads_headers_with_lsd
    - test_threads_headers_with_session
    - test_threads_headers_to_map
    
    // ThreadsRequestBuilder Tests
    - test_threads_request_builder_new
    - test_threads_request_builder_with_lsd
    - test_threads_request_builder_with_session
    - test_threads_request_builder_build
    
    // RelayError Tests
    - test_relay_error_parse
    - test_relay_error_message
}
```

### 6.4 Scraper Parser Tests (`src/scraper_parser.rs`)

```rust
mod scraper_parser_tests {
    // ScraperParser Tests
    - test_scraper_parser_new
    - test_scraper_parser_parse_basic
    - test_scraper_parser_parse_post
    - test_scraper_parser_parse_user
    - test_scraper_parser_parse_invalid
    
    // ScraperPost Tests
    - test_scraper_post_new
    - test_scraper_post_into_ghost_post
    - test_scraper_post_with_media
    
    // ScraperOutput Tests
    - test_scraper_output_new
    - test_scraper_output_parse_json
    - test_scraper_output_posts
    
    // WorkerResponse Tests
    - test_worker_response_success
    - test_worker_response_failure
    - test_worker_response_from_json
    
    // Helper Function Tests
    - test_parse_scraper_output
    - test_parse_scraper_blob
    - test_parse_worker_json
}
```

### 6.5 Official API Tests (`src/official.rs`)

```rust
mod threads_official_tests {
    // ThreadsOfficialClient Tests
    - test_threads_official_client_new
    - test_threads_official_client_with_token
    - test_threads_official_client_with_timeout
    - test_threads_official_client_base_url
    
    // Media Creation Tests
    - test_create_media_request_text_only
    - test_create_media_request_with_image
    - test_create_media_request_with_video
    - test_create_media_request_with_carousel
    - test_create_media_request_reply_control
    
    // API Response Tests
    - test_api_response_success
    - test_api_response_error
    - test_api_error_parse
    
    // Media Container Tests
    - test_media_container_status
    - test_media_container_pending
    - test_media_container_published
    - test_media_container_error
    
    // Insights Tests
    - test_insights_response_parse
    - test_insight_metric_parse
    - test_insight_value_parse
    
    // Token Tests
    - test_token_response_parse
    - test_long_lived_token_response_parse
    
    // Constants Tests
    - test_threads_api_base_url
    - test_default_api_version
    - test_max_posts_per_day
    - test_max_text_length
    - test_max_carousel_items
}
```

---

## 7. ghost-server Testing Plan

### 7.1 Handler Tests (`src/handlers.rs`)

```rust
mod handler_tests {
    // Health Handler Tests
    - test_health_handler_returns_ok
    - test_health_handler_includes_workers
    - test_health_handler_includes_platforms
    
    // Post Handler Tests
    - test_get_post_handler_missing_id
    - test_get_post_handler_invalid_platform
    - test_get_post_handler_success
    - test_get_post_handler_worker_error
    
    // User Handler Tests
    - test_get_user_handler_missing_id
    - test_get_user_handler_invalid_platform
    - test_get_user_handler_success
    - test_get_user_handler_worker_error
    
    // Search Handler Tests
    - test_search_handler_missing_query
    - test_search_handler_empty_query
    - test_search_handler_success
    - test_search_handler_worker_error
    
    // Trending Handler Tests
    - test_trending_handler_success
    - test_trending_handler_worker_error
    
    // Timeline Handler Tests
    - test_timeline_handler_missing_user_id
    - test_timeline_handler_success
    - test_timeline_handler_worker_error
    
    // Context Injection Tests
    - test_handler_context_injection_proxy_header
    - test_handler_context_injection_session_header
    - test_handler_context_injection_tenant_header
    
    // Error Handling Tests
    - test_handler_returns_404_for_unknown_route
    - test_handler_returns_500_for_internal_error
    - test_handler_returns_429_for_rate_limit
    - test_handler_returns_503_for_workers_exhausted
}
```

### 7.2 Route Tests (`src/routes.rs`)

```rust
mod route_tests {
    // Route Registration Tests
    - test_routes_register_health
    - test_routes_register_api_routes
    - test_routes_register_swagger
    - test_routes_create_router
    
    // Path Tests
    - test_route_path_health
    - test_route_path_get_post
    - test_route_path_get_user
    - test_route_path_search
    - test_route_path_trending
    - test_route_path_timeline
    
    // Middleware Tests
    - test_routes_cors_middleware
    - test_routes_trace_middleware
    - test_routes_limit_middleware
}
```

### 7.3 Error Tests (`src/error.rs`)

```rust
mod server_error_tests {
    // ServerError Creation Tests
    - test_server_error_from_ghost_error
    - test_server_error_from_io_error
    - test_server_error_not_found
    - test_server_error_bad_request
    - test_server_error_internal
    
    // Status Code Tests
    - test_server_error_status_code_404
    - test_server_error_status_code_400
    - test_server_error_status_code_500
    - test_server_error_status_code_429
    - test_server_error_status_code_503
    
    // Response Tests
    - test_server_error_into_response
    - test_server_error_json_body
}
```

### 7.4 Integration Tests

```rust
mod server_integration_tests {
    // Full Request Flow Tests
    - test_server_full_request_get_post
    - test_server_full_request_get_user
    - test_server_full_request_search
    - test_server_full_request_trending
    - test_server_full_request_timeline
    
    // Headers Tests
    - test_server_headers_cors
    - test_server_headers_trace_id
    - test_server_headers_provider
    
    // Swagger Tests
    - test_server_swagger_ui_accessible
    - test_server_openapi_json_accessible
    
    // Graceful Shutdown Tests
    - test_server_graceful_shutdown
    - test_server_shutdown_with_pending_requests
}
```

---

## 8. Test Utilities and Fixtures

### 8.1 Test Data Builders

```rust
// Location: tests/common/mod.rs

/// Builder for creating test GhostPost instances
pub struct GhostPostBuilder {
    id: String,
    platform: Platform,
    text: String,
    // ... other fields
}

impl GhostPostBuilder {
    pub fn new() -> Self { ... }
    pub fn with_id(mut self, id: &str) -> Self { ... }
    pub fn with_platform(mut self, platform: Platform) -> Self { ... }
    pub fn with_text(mut self, text: &str) -> Self { ... }
    pub fn build(self) -> GhostPost { ... }
}

/// Builder for creating test GhostUser instances
pub struct GhostUserBuilder { ... }

/// Builder for creating test GhostContext instances
pub struct GhostContextBuilder { ... }

/// Builder for creating test PayloadBlob instances
pub struct PayloadBlobBuilder { ... }
```

### 8.2 Test Fixtures

```rust
// Location: tests/common/fixtures.rs

/// Real X tweet JSON response fixture
pub fn fixture_x_tweet_response() -> &'static str { ... }

/// Real X user JSON response fixture
pub fn fixture_x_user_response() -> &'static str { ... }

/// Real Threads post JSON response fixture
pub fn fixture_threads_post_response() -> &'static str { ... }

/// Real Threads user JSON response fixture
pub fn fixture_threads_user_response() -> &'static str { ... }

/// Real timeline JSON response fixture
pub fn fixture_timeline_response() -> &'static str { ... }

/// Real search results JSON response fixture
pub fn fixture_search_response() -> &'static str { ... }
```

### 8.3 Test Helpers

```rust
// Location: tests/common/helpers.rs

/// Create a temporary file for testing file-based operations
pub fn temp_file() -> tempfile::NamedTempFile { ... }

/// Create a temporary directory for testing
pub fn temp_dir() -> tempfile::TempDir { ... }

/// Wait for a condition with timeout
pub async fn wait_for<F, T>(f: F, timeout: Duration) -> T
where
    F: Fn() -> Option<T>,
{ ... }

/// Assert that a result is an error of a specific type
pub fn assert_error_type<T, E>(result: Result<T, E>, expected_type: ErrorType) { ... }

/// Run a test with a timeout
pub async fn with_timeout<F, T>(future: F, timeout: Duration) -> T
where
    F: Future<Output = T>,
{ ... }
```

---

## 9. Test Execution Strategy

### 9.1 Test Categories

| Category | Command | Purpose |
|----------|---------|---------|
| Unit Tests | `cargo test --lib` | Fast, isolated tests |
| Integration Tests | `cargo test --test '*'` | Multi-component tests |
| Documentation Tests | `cargo test --doc` | Code example verification |
| All Tests | `cargo test --all` | Complete test suite |

### 9.2 Parallel Execution

```bash
# Run tests with maximum parallelism
cargo test --all -- --test-threads=$(nproc)

# Run tests for a specific crate
cargo test -p ghost-core

# Run a specific test
cargo test test_ghost_init

# Run tests matching a pattern
cargo test health
```

### 9.3 CI Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all --verbose
      - run: cargo test --all --doc
```

---

## 10. Test Quality Checklist

Each test must satisfy these criteria:

### Test Isolation
- [ ] Test creates its own test data
- [ ] Test cleans up resources (temp files, etc.)
- [ ] Test does not depend on execution order
- [ ] Test does not share mutable state with other tests

### Test Coverage
- [ ] Happy path is tested
- [ ] Error paths are tested
- [ ] Edge cases are tested (empty, null, max values)
- [ ] Boundary conditions are tested

### Test Verification
- [ ] Assertions verify actual behavior
- [ ] Assertions check return values
- [ ] Assertions check state changes
- [ ] Assertions check error types and messages
- [ ] No shallow assertions (e.g., `assert!(result.is_ok())` alone)

### Test Documentation
- [ ] Test name describes what is being tested
- [ ] Complex test logic has comments
- [ ] Test purpose is clear from reading

### Test Performance
- [ ] Test runs in reasonable time (< 1s for unit tests)
- [ ] Test uses appropriate async runtime
- [ ] Test does not have unnecessary sleeps

---

## 11. Implementation Priority

### Phase 1: Core Infrastructure (Week 1-2)
1. ghost-schema types and error tests
2. ghost-core health engine tests
3. ghost-vault vault and credential tests

### Phase 2: Adapters (Week 3-4)
1. x-adapter parser and adapter tests
2. threads-adapter parser and adapter tests
3. Integration tests with real JSON fixtures

### Phase 3: Bridge and Server (Week 5-6)
1. ghost-bridge protocol and pool tests
2. ghost-server handler and route tests
3. End-to-end integration tests

### Phase 4: Refinement (Week 7-8)
1. Fill coverage gaps
2. Add edge case tests
3. Performance and concurrency tests
4. Documentation and CI integration

---

## 12. Success Metrics

| Metric | Target |
|--------|--------|
| Line Coverage | >= 85% |
| Branch Coverage | >= 75% |
| Test Count | >= 730 |
| Zero `todo!()` in tests | 100% |
| All tests passing | 100% |
| No flaky tests | 0 failures in 100 runs |

---

## Conclusion

This comprehensive test plan provides a roadmap for achieving high-quality test coverage across all crates in the ghost-api workspace. The emphasis on real implementation testing, clean isolation, and thorough verification ensures that the tests provide meaningful confidence in the codebase's correctness and reliability.
