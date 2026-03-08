//! Transformation and mapping functions for Ghost types
//!
//! This module provides functions for transforming platform-specific data
//! into unified Ghost types. All functions are scaffolded with TODO markers.

use crate::{GhostError, GhostResult, Platform, MediaType};
use serde::{Deserialize, Serialize};

// ============================================================================
// Timestamp Parsing
// ============================================================================

/// Parse X (Twitter) ISO 8601 timestamp to Unix timestamp
///
/// X API format: "2024-01-15T10:30:00.000Z"
pub fn parse_x_timestamp(ts: &str) -> GhostResult<i64> {
    // TODO: Implement X timestamp parsing
    // - Parse ISO 8601 format with milliseconds
    // - Handle timezone (Z suffix)
    // - Return Unix timestamp in seconds
    Err(GhostError::NotImplemented("parse_x_timestamp".into()))
}

/// Parse Threads timestamp to Unix timestamp
///
/// Threads format: "2024-01-15T10:30:00+0000"
pub fn parse_threads_timestamp(ts: &str) -> GhostResult<i64> {
    // TODO: Implement Threads timestamp parsing
    // - Parse non-standard ISO format with timezone offset
    // - Handle +0000 suffix instead of Z
    // - Return Unix timestamp in seconds
    Err(GhostError::NotImplemented("parse_threads_timestamp".into()))
}

/// Generic ISO 8601 timestamp parser
///
/// Attempts multiple common formats for flexibility
pub fn parse_iso8601(ts: &str) -> GhostResult<i64> {
    // TODO: Implement generic ISO 8601 parsing
    // - Try multiple format variations:
    //   - "%Y-%m-%dT%H:%M:%S%.3fZ"
    //   - "%Y-%m-%dT%H:%M:%SZ"
    //   - "%Y-%m-%dT%H:%M:%S%z"
    //   - "%Y-%m-%dT%H:%M:%S%.3f%z"
    // - Fallback to chrono's RFC 3339 parser
    // - Return Unix timestamp in seconds
    Err(GhostError::NotImplemented("parse_iso8601".into()))
}

/// Convert Unix timestamp to ISO 8601 string
pub fn unix_to_iso8601(unix: i64) -> String {
    // TODO: Implement Unix to ISO 8601 conversion
    // - Convert Unix timestamp to DateTime<Utc>
    // - Format as ISO 8601 with Z suffix
    format!("1970-01-01T00:00:00Z")
}

/// Convert Unix timestamp to Threads format
pub fn unix_to_threads_timestamp(unix: i64) -> String {
    // TODO: Implement Unix to Threads timestamp conversion
    // - Convert Unix timestamp to DateTime<Utc>
    // - Format with +0000 timezone suffix
    format!("1970-01-01T00:00:00+0000")
}

// ============================================================================
// Username Normalization
// ============================================================================

/// Normalize username by removing @ prefix and lowercasing
pub fn normalize_username(username: &str) -> String {
    // TODO: Implement username normalization
    // - Strip @ prefix if present
    // - Convert to lowercase
    // - Validate allowed characters (alphanumeric, underscore)
    // - Return normalized username
    username.trim_start_matches('@').to_lowercase()
}

/// Construct profile URL from platform and username
pub fn build_profile_url(platform: Platform, username: &str) -> String {
    // TODO: Implement profile URL construction
    // - Normalize username
    // - Use platform-specific URL patterns:
    //   - X: https://x.com/{username}
    //   - Threads: https://www.threads.net/@{username}
    let normalized = normalize_username(username);
    match platform {
        Platform::X => format!("https://x.com/{}", normalized),
        Platform::Threads => format!("https://www.threads.net/@{}", normalized),
        Platform::Unknown => String::new(),
    }
}

/// Construct post URL from platform and post ID
pub fn build_post_url(platform: Platform, username: &str, post_id: &str) -> String {
    // TODO: Implement post URL construction
    // - Use platform-specific URL patterns:
    //   - X: https://x.com/{username}/status/{post_id}
    //   - Threads: https://www.threads.net/@{username}/post/{post_id}
    let normalized = normalize_username(username);
    match platform {
        Platform::X => format!("https://x.com/{}/status/{}", normalized, post_id),
        Platform::Threads => format!("https://www.threads.net/@{}/post/{}", normalized, post_id),
        Platform::Unknown => String::new(),
    }
}

// ============================================================================
// Media Type Mapping
// ============================================================================

/// Map X media type string to MediaType enum
pub fn map_x_media_type(x_type: &str) -> MediaType {
    // TODO: Implement X media type mapping
    // - "photo" -> MediaType::Image
    // - "video" -> MediaType::Video
    // - "animated_gif" -> MediaType::Gif
    // - Unknown -> MediaType::Unknown
    match x_type {
        "photo" => MediaType::Image,
        "video" => MediaType::Video,
        "animated_gif" => MediaType::Gif,
        _ => MediaType::Unknown,
    }
}

/// Map Threads media type string to MediaType enum
pub fn map_threads_media_type(threads_type: &str) -> MediaType {
    // TODO: Implement Threads media type mapping
    // - "IMAGE" -> MediaType::Image
    // - "VIDEO" -> MediaType::Video
    // - "CAROUSEL" -> MediaType::Unknown (handled separately)
    // - "TEXT" -> MediaType::Unknown (no media)
    // - Unknown -> MediaType::Unknown
    match threads_type {
        "IMAGE" => MediaType::Image,
        "VIDEO" => MediaType::Video,
        _ => MediaType::Unknown,
    }
}

/// Convert MediaType to string for serialization
pub fn media_type_to_string(media_type: MediaType) -> &'static str {
    // TODO: Implement MediaType to string conversion
    match media_type {
        MediaType::Image => "image",
        MediaType::Video => "video",
        MediaType::Gif => "gif",
        MediaType::Audio => "audio",
        MediaType::Unknown => "unknown",
    }
}

// ============================================================================
// Verification Mapping
// ============================================================================

/// X verification type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XVerificationType {
    /// X Premium/Blue subscriber
    Blue,
    /// Verified business account
    Business,
    /// Government official account
    Government,
    /// Legacy verified (pre-Blue)
    Legacy,
    /// Not verified
    None,
}

impl XVerificationType {
    /// Parse from X API verified_type field
    pub fn from_x_api(verified_type: Option<&str>, legacy_verified: bool) -> Self {
        // TODO: Implement verification type parsing
        // - Check verified_type first
        // - If None but legacy_verified is true, return Legacy
        // - Otherwise return None
        match verified_type {
            Some("blue") => XVerificationType::Blue,
            Some("business") => XVerificationType::Business,
            Some("government") => XVerificationType::Government,
            Some(_) => XVerificationType::None,
            None if legacy_verified => XVerificationType::Legacy,
            None => XVerificationType::None,
        }
    }

    /// Check if this verification type is verified
    pub fn is_verified(&self) -> bool {
        // TODO: Implement verification check
        !matches!(self, XVerificationType::None)
    }
}

/// Determine if user is verified based on platform-specific fields
pub fn is_user_verified(platform: Platform, verified: Option<bool>, verified_type: Option<&str>) -> bool {
    // TODO: Implement cross-platform verification check
    // - For X: Check verified OR verified_type.is_some()
    // - For Threads: Check verified boolean directly
    match platform {
        Platform::X => {
            verified.unwrap_or(false) || verified_type.is_some()
        }
        Platform::Threads => verified.unwrap_or(false),
        Platform::Unknown => false,
    }
}

// ============================================================================
// Metric Aggregation
// ============================================================================

/// Aggregated metrics across platforms
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Total likes/reactions
    pub likes: u64,
    /// Total shares/reposts
    pub shares: u64,
    /// Total comments/replies
    pub comments: u64,
    /// Total views/impressions (if available)
    pub views: Option<u64>,
    /// Calculated engagement rate (percentage)
    pub engagement_rate: Option<f64>,
    /// Total engagement (likes + shares + comments)
    pub total_engagement: u64,
}

impl AggregatedMetrics {
    /// Create from individual metric values
    pub fn new(likes: u64, shares: u64, comments: u64, views: Option<u64>) -> Self {
        // TODO: Implement metrics aggregation
        // - Calculate total_engagement
        // - Calculate engagement_rate if views available
        let total_engagement = likes + shares + comments;
        let engagement_rate = views.map(|v| {
            if v > 0 {
                (total_engagement as f64 / v as f64) * 100.0
            } else {
                0.0
            }
        });

        Self {
            likes,
            shares,
            comments,
            views,
            engagement_rate,
            total_engagement,
        }
    }

    /// Check if metrics are empty (all zeros)
    pub fn is_empty(&self) -> bool {
        // TODO: Implement empty check
        self.likes == 0 && self.shares == 0 && self.comments == 0
    }

    /// Merge metrics from another source (for cross-platform aggregation)
    pub fn merge(&mut self, other: &AggregatedMetrics) {
        // TODO: Implement metrics merging
        // - Add likes, shares, comments
        // - Use max of views (or None if both None)
        // - Recalculate engagement_rate
        self.likes += other.likes;
        self.shares += other.shares;
        self.comments += other.comments;
        self.views = self.views.max(other.views);
        self.total_engagement += other.total_engagement;
        // Recalculate engagement rate
        if let Some(v) = self.views {
            self.engagement_rate = if v > 0 {
                Some((self.total_engagement as f64 / v as f64) * 100.0)
            } else {
                Some(0.0)
            };
        }
    }
}

// ============================================================================
// Content Type Inference
// ============================================================================

/// Infer MIME content type from URL and media type
pub fn infer_content_type(url: &str, media_type: &MediaType) -> Option<String> {
    // TODO: Implement content type inference
    // - Check URL extension for hints
    // - Use media type as fallback
    // - Return appropriate MIME type
    match media_type {
        MediaType::Image => {
            if url.contains(".png") {
                Some("image/png".to_string())
            } else if url.contains(".gif") {
                Some("image/gif".to_string())
            } else if url.contains(".webp") {
                Some("image/webp".to_string())
            } else {
                Some("image/jpeg".to_string())
            }
        }
        MediaType::Video => {
            if url.contains(".webm") {
                Some("video/webm".to_string())
            } else if url.contains(".mov") {
                Some("video/quicktime".to_string())
            } else {
                Some("video/mp4".to_string())
            }
        }
        MediaType::Gif => Some("image/gif".to_string()),
        MediaType::Audio => Some("audio/mpeg".to_string()),
        MediaType::Unknown => None,
    }
}

/// Get file extension from MIME type
pub fn mime_to_extension(mime: &str) -> Option<&'static str> {
    // TODO: Implement MIME to extension mapping
    match mime {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "video/mp4" => Some("mp4"),
        "video/webm" => Some("webm"),
        "video/quicktime" => Some("mov"),
        "audio/mpeg" => Some("mp3"),
        "audio/wav" => Some("wav"),
        _ => None,
    }
}

// ============================================================================
// Duration Conversion
// ============================================================================

/// Convert milliseconds to seconds
pub fn millis_to_secs(ms: u64) -> f64 {
    // TODO: Implement millisecond to second conversion
    ms as f64 / 1000.0
}

/// Convert seconds to milliseconds
pub fn secs_to_millis(secs: f64) -> u64 {
    // TODO: Implement second to millisecond conversion
    (secs * 1000.0) as u64
}

/// Format duration as human-readable string
pub fn format_duration(secs: f64) -> String {
    // TODO: Implement duration formatting
    // - Format as MM:SS or HH:MM:SS as appropriate
    let total_secs = secs as u64;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{}:{:02}", mins, secs)
    }
}

// ============================================================================
// Entity Extraction
// ============================================================================

/// Extract hashtags from text
pub fn extract_hashtags(text: &str) -> Vec<String> {
    // TODO: Implement hashtag extraction
    // - Find all #hashtag patterns
    // - Validate hashtag format (alphanumeric + underscore)
    // - Return list of hashtags without # prefix
    Vec::new()
}

/// Extract mentions from text
pub fn extract_mentions(text: &str) -> Vec<String> {
    // TODO: Implement mention extraction
    // - Find all @username patterns
    // - Validate username format
    // - Return list of usernames without @ prefix
    Vec::new()
}

/// Extract URLs from text
pub fn extract_urls(text: &str) -> Vec<String> {
    // TODO: Implement URL extraction
    // - Find all http(s):// patterns
    // - Handle t.co shortened URLs specially
    // - Return list of URLs
    Vec::new()
}

// ============================================================================
// Text Processing
// ============================================================================

/// Truncate text to platform-specific limit
pub fn truncate_text(text: &str, platform: Platform) -> String {
    // TODO: Implement text truncation
    // - X: 280 chars (basic) or 4000 (Blue)
    // - Threads: 500 chars
    // - Add ellipsis if truncated
    let limit = match platform {
        Platform::X => 280,
        Platform::Threads => 500,
        Platform::Unknown => 280,
    };

    if text.len() <= limit {
        text.to_string()
    } else {
        format!("{}...", &text[..limit.saturating_sub(3)])
    }
}

/// Count characters according to platform rules
pub fn count_characters(text: &str, platform: Platform) -> usize {
    // TODO: Implement character counting
    // - X: Count grapheme clusters (not bytes)
    // - Handle URL shortening (t.co URLs count as 23 chars on X)
    // - Handle mention/hashtag counting
    let _ = platform;
    text.chars().count()
}

/// Check if text exceeds platform limit
pub fn exceeds_limit(text: &str, platform: Platform) -> bool {
    // TODO: Implement limit check
    // - Use count_characters
    // - Compare against platform limits
    let limit = match platform {
        Platform::X => 280,
        Platform::Threads => 500,
        Platform::Unknown => 280,
    };
    count_characters(text, platform) > limit
}

// ============================================================================
// ID Validation
// ============================================================================

/// Validate X post ID format (Snowflake ID)
pub fn is_valid_x_post_id(id: &str) -> bool {
    // TODO: Implement X post ID validation
    // - X uses Snowflake IDs (numeric strings)
    // - Typically 18-19 digits
    // - All characters must be numeric
    id.chars().all(|c| c.is_ascii_digit()) && id.len() >= 16 && id.len() <= 20
}

/// Validate Threads post ID format
pub fn is_valid_threads_post_id(id: &str) -> bool {
    // TODO: Implement Threads post ID validation
    // - Threads uses Instagram-style IDs
    // - Typically numeric strings
    !id.is_empty() && id.chars().all(|c| c.is_ascii_digit())
}

/// Validate username format
pub fn is_valid_username(username: &str) -> bool {
    // TODO: Implement username validation
    // - Must be 1-15 characters
    // - Only alphanumeric and underscore
    // - Cannot start with number
    let normalized = normalize_username(username);
    !normalized.is_empty()
        && normalized.len() <= 15
        && normalized.chars().all(|c| c.is_alphanumeric() || c == '_')
}

// ============================================================================
// Reference Extraction (X)
// ============================================================================

/// Types of tweet references
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceType {
    /// Reply to another tweet
    RepliedTo,
    /// Quote tweet
    Quoted,
    /// Retweet
    Retweeted,
}

/// Extract reference type and ID from X referenced_tweets
pub fn extract_reference(referenced: &[serde_json::Value], ref_type: ReferenceType) -> Option<String> {
    // TODO: Implement reference extraction
    // - Filter referenced_tweets array by type
    // - Return the ID if found
    let type_str = match ref_type {
        ReferenceType::RepliedTo => "replied_to",
        ReferenceType::Quoted => "quoted",
        ReferenceType::Retweeted => "retweeted",
    };

    referenced
        .iter()
        .find(|r| r.get("type").and_then(|t| t.as_str()) == Some(type_str))
        .and_then(|r| r.get("id").and_then(|id| id.as_str()))
        .map(|s| s.to_string())
}

// ============================================================================
// Pagination Helpers
// ============================================================================

/// X pagination token wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPagination {
    /// Token for next page
    pub next_token: Option<String>,
    /// Token for previous page
    pub previous_token: Option<String>,
    /// Result count
    pub result_count: u32,
    /// Newest ID in results
    pub newest_id: Option<String>,
    /// Oldest ID in results
    pub oldest_id: Option<String>,
}

impl XPagination {
    /// Parse from X API meta object
    pub fn from_meta(meta: &serde_json::Value) -> Option<Self> {
        // TODO: Implement X pagination parsing
        // - Extract next_token, previous_token
        // - Extract result_count
        // - Extract newest_id, oldest_id
        let _ = meta;
        None
    }

    /// Check if there's a next page
    pub fn has_next(&self) -> bool {
        // TODO: Implement next page check
        self.next_token.is_some()
    }

    /// Check if there's a previous page
    pub fn has_previous(&self) -> bool {
        // TODO: Implement previous page check
        self.previous_token.is_some()
    }
}

/// Threads pagination cursor wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsPagination {
    /// Cursor for results before current
    pub before: Option<String>,
    /// Cursor for results after current
    pub after: Option<String>,
    /// Full URL for next page
    pub next: Option<String>,
    /// Full URL for previous page
    pub previous: Option<String>,
}

impl ThreadsPagination {
    /// Parse from Threads API paging object
    pub fn from_paging(paging: &serde_json::Value) -> Option<Self> {
        // TODO: Implement Threads pagination parsing
        // - Extract cursors.before, cursors.after
        // - Extract next, previous URLs
        let _ = paging;
        None
    }

    /// Check if there's a next page
    pub fn has_next(&self) -> bool {
        // TODO: Implement next page check
        self.after.is_some() || self.next.is_some()
    }

    /// Check if there's a previous page
    pub fn has_previous(&self) -> bool {
        // TODO: Implement previous page check
        self.before.is_some() || self.previous.is_some()
    }
}

// ============================================================================
// Error Code Mapping
// ============================================================================

/// Map X API error code to GhostError
pub fn map_x_error(code: u16, message: &str) -> GhostError {
    // TODO: Implement X error code mapping
    // - 429 -> RateLimited
    // - 401 -> AuthError
    // - 403 -> AccountSuspended or WafChallenge
    // - 404 -> PlatformError with NotFound
    // - etc.
    let _ = (code, message);
    GhostError::PlatformError {
        code,
        message: message.to_string(),
        platform: Platform::X,
    }
}

/// Map Threads API error code to GhostError
pub fn map_threads_error(code: u16, error_type: &str, message: &str) -> GhostError {
    // TODO: Implement Threads error code mapping
    // - 190 -> AuthError (OAuthException)
    // - 17 -> RateLimited (ApiException)
    // - 4 -> AuthError (OAuthException)
    // - 100 -> ValidationError (Invalid parameter)
    // - 200 -> AuthError (Permission denied)
    // etc.
    let _ = (code, error_type, message);
    GhostError::PlatformError {
        code,
        message: format!("{}: {}", error_type, message),
        platform: Platform::Threads,
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_username() {
        // TODO: Implement test
        assert_eq!(normalize_username("@test"), "test");
        assert_eq!(normalize_username("Test"), "test");
    }

    #[test]
    fn test_build_profile_url() {
        // TODO: Implement test
        assert_eq!(
            build_profile_url(Platform::X, "test"),
            "https://x.com/test"
        );
        assert_eq!(
            build_profile_url(Platform::Threads, "test"),
            "https://www.threads.net/@test"
        );
    }

    #[test]
    fn test_map_x_media_type() {
        // TODO: Implement test
        assert_eq!(map_x_media_type("photo"), MediaType::Image);
        assert_eq!(map_x_media_type("video"), MediaType::Video);
        assert_eq!(map_x_media_type("animated_gif"), MediaType::Gif);
    }

    #[test]
    fn test_aggregated_metrics() {
        // TODO: Implement test
        let metrics = AggregatedMetrics::new(100, 50, 25, Some(1000));
        assert_eq!(metrics.likes, 100);
        assert_eq!(metrics.total_engagement, 175);
    }

    #[test]
    fn test_format_duration() {
        // TODO: Implement test
        assert_eq!(format_duration(125.0), "2:05");
        assert_eq!(format_duration(3661.0), "1:01:01");
    }
}
