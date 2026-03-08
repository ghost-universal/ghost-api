//! X GraphQL endpoint definitions
//!
//! GraphQL query IDs and endpoints for X internal API

/// X GraphQL query IDs
pub struct GraphQLQueries;

impl GraphQLQueries {
    /// Tweet detail query ID
    pub const TWEET_DETAIL: &'static str = "2ICDjqPd81riZzj2XVWmrw";

    /// User by screen name query ID
    pub const USER_BY_SCREEN_NAME: &'static str = "k5XeB6s5R6L8u0YwW3T3nA";

    /// User tweets query ID
    pub const USER_TWEETS: &'static str = "u5Yp5mX9L6X8H9wX7k9P2Q";

    /// Search adaptive query ID
    pub const SEARCH_ADAPTIVE: &'static str = "nK1kF0dWQ1L2x8Y3n9P4sA";

    /// Home timeline query ID
    pub const HOME_TIMELINE: &'static str = "3JqX0K8mP1xY2W4vL5z9R";

    /// Trending query ID
    pub const TRENDING: &'static str = "g5j5Y5f8p1L2x3W7k9V4zA";

    /// Followers query ID
    pub const FOLLOWERS: &'static str = "m5X9bQ2L1p8Y3w7K4v6R";

    /// Following query ID
    pub const FOLLOWING: &'static str = "k5Y6W1L9p2X8v3Q7R4mN";

    /// Returns the GraphQL base URL
    pub fn base_url() -> &'static str {
        "https://x.com/i/api/graphql"
    }

    /// Builds a GraphQL URL for a query
    pub fn build_url(query_id: &str, variables: &str) -> String {
        // TODO: Implement URL building with variables
        format!("{}/{}?variables={}", Self::base_url(), query_id, variables)
    }

    /// Creates default variables for tweet detail
    pub fn tweet_detail_vars(tweet_id: &str) -> String {
        // TODO: Implement tweet detail variables
        serde_json::to_string(&serde_json::json!({
            "tweetId": tweet_id,
            "includePromotedContent": true,
            "withCommunity": true
        }))
        .unwrap_or_default()
    }

    /// Creates default variables for user by screen name
    pub fn user_by_screen_name_vars(screen_name: &str) -> String {
        // TODO: Implement user variables
        serde_json::to_string(&serde_json::json!({
            "screen_name": screen_name,
            "withSafetyModeUserFields": true
        }))
        .unwrap_or_default()
    }
}

/// X GraphQL feature flags
pub struct GraphQLFeatures;

impl GraphQLFeatures {
    /// Default features for queries
    pub fn default_features() -> serde_json::Value {
        // TODO: Implement feature flags
        serde_json::json!({
            "blue_business_profile_label_shape": true,
            "creator_subscriptions_tweet_preview_api_enabled": true,
            "freedom_of_speech_not_reach_fetch_enabled": true,
            "graphql_is_translatable_rweb_tweet_is_translatable_enabled": true,
            "longform_notetweets_consumption_enabled": true,
            "responsive_web_edit_tweet_api_enabled": true,
            "responsive_web_enhance_cards_enabled": false,
            "responsive_web_graphql_exclude_directive_enabled": true,
            "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
            "responsive_web_graphql_timeline_navigation_enabled": true,
            "responsive_web_media_download_video_enabled": false,
            "responsive_web_text_conversations_enabled": false,
            "responsive_web_twitter_article_tweet_consumption_enabled": true,
            "responsive_web_twitter_blue_verified_badge_is_enabled": true,
            "rweb_tipjar_consumption_enabled": true,
            "standardized_nudges_toxicity": false,
            "tweet_awards_web_tipping_enabled": false,
            "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled": true,
            "tweetypie_unmention_optimization_enabled": true,
            "unified_cards_ad_metadata_container_dynamic_card_content_query_enabled": true,
            "verified_phone_label_enabled": false,
            "vibe_api_enabled": true,
            "view_counts_everywhere_api_enabled": true
        })
    }
}
