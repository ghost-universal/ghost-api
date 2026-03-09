//! Platform-specific adapter types for Ghost API
//!
//! This module contains types used by platform adapters for parsing
//! and transforming platform-specific data into unified Ghost types.

use serde::{Deserialize, Serialize};
use crate::{Platform, GhostError};

// ============================================================================
// Common Adapter Types
// ============================================================================

/// Result of parsing a platform response
#[derive(Debug, Clone)]
pub enum AdapterParseResult {
    /// Single user profile
    User(crate::GhostUser),
    /// Single post
    Post(crate::GhostPost),
    /// Multiple posts
    Posts(Vec<crate::GhostPost>),
    /// Search results with pagination
    Search {
        posts: Vec<crate::GhostPost>,
        cursor: Option<String>,
    },
    /// Timeline with bidirectional pagination
    Timeline {
        posts: Vec<crate::GhostPost>,
        cursor_top: Option<String>,
        cursor_bottom: Option<String>,
    },
    /// Thread/conversation
    Thread {
        posts: Vec<crate::GhostPost>,
        cursor: Option<String>,
    },
    /// Trending topics
    Trending(Vec<TrendingTopic>),
    /// Error response
    Error(AdapterError),
}

impl AdapterParseResult {
    /// Returns the posts if this is a posts result
    pub fn into_posts(self) -> Option<Vec<crate::GhostPost>> {
        // TODO: Implement posts extraction
        match self {
            AdapterParseResult::Posts(posts) => Some(posts),
            AdapterParseResult::Search { posts, .. } => Some(posts),
            AdapterParseResult::Timeline { posts, .. } => Some(posts),
            AdapterParseResult::Thread { posts, .. } => Some(posts),
            _ => None,
        }
    }

    /// Returns the single post if this is a post result
    pub fn into_post(self) -> Option<crate::GhostPost> {
        // TODO: Implement post extraction
        match self {
            AdapterParseResult::Post(post) => Some(post),
            _ => None,
        }
    }

    /// Returns the user if this is a user result
    pub fn into_user(self) -> Option<crate::GhostUser> {
        // TODO: Implement user extraction
        match self {
            AdapterParseResult::User(user) => Some(user),
            _ => None,
        }
    }

    /// Check if this is an error result
    pub fn is_error(&self) -> bool {
        // TODO: Implement error check
        matches!(self, AdapterParseResult::Error(_))
    }
}

/// Adapter error types
#[derive(Debug, Clone)]
pub enum AdapterError {
    /// Rate limited
    RateLimited {
        retry_after: Option<u64>,
        platform: Platform,
    },
    /// Account suspended
    AccountSuspended {
        user_id: String,
        platform: Platform,
    },
    /// Resource not found
    NotFound {
        resource_type: String,
        resource_id: String,
        platform: Platform,
    },
    /// Protected/private account
    ProtectedAccount {
        user_id: String,
        platform: Platform,
    },
    /// Login required
    LoginRequired {
        platform: Platform,
    },
    /// Suspicious activity / challenge
    SuspiciousActivity {
        challenge_url: Option<String>,
        platform: Platform,
    },
    /// Parsing error
    ParseError {
        message: String,
        platform: Platform,
    },
    /// Network error
    NetworkError {
        message: String,
        platform: Platform,
    },
}

impl AdapterError {
    /// Create a rate limit error
    pub fn rate_limited(platform: Platform, retry_after: Option<u64>) -> Self {
        // TODO: Implement rate limit error construction
        Self::RateLimited { retry_after, platform }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        // TODO: Implement retryability check
        matches!(self, AdapterError::RateLimited { .. } | AdapterError::NetworkError { .. })
    }

    /// Get the platform for this error
    pub fn platform(&self) -> Platform {
        // TODO: Implement platform extraction
        match self {
            AdapterError::RateLimited { platform, .. } => *platform,
            AdapterError::AccountSuspended { platform, .. } => *platform,
            AdapterError::NotFound { platform, .. } => *platform,
            AdapterError::ProtectedAccount { platform, .. } => *platform,
            AdapterError::LoginRequired { platform } => *platform,
            AdapterError::SuspiciousActivity { platform, .. } => *platform,
            AdapterError::ParseError { platform, .. } => *platform,
            AdapterError::NetworkError { platform, .. } => *platform,
        }
    }
}

/// Trending topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingTopic {
    /// Topic name/hashtag
    pub name: String,
    /// URL to topic page
    pub url: Option<String>,
    /// Tweet/post volume
    pub post_count: Option<u64>,
    /// Topic description
    pub description: Option<String>,
    /// Trending rank position
    pub rank: Option<u32>,
    /// Platform this trend is from
    pub platform: Platform,
}

impl TrendingTopic {
    /// Create a new trending topic
    pub fn new(name: impl Into<String>, platform: Platform) -> Self {
        // TODO: Implement trending topic construction
        Self {
            name: name.into(),
            url: None,
            post_count: None,
            description: None,
            rank: None,
            platform,
        }
    }
}

// ============================================================================
// X (Twitter) Specific Types
// ============================================================================

/// X API error types
#[derive(Debug, Clone)]
pub enum XError {
    /// Rate limited
    RateLimited {
        retry_after: Option<u64>,
    },
    /// Account suspended
    AccountSuspended {
        user_id: String,
    },
    /// Not found
    NotFound {
        resource_type: String,
        resource_id: String,
    },
    /// Protected account
    ProtectedAccount {
        user_id: String,
    },
    /// Login required
    LoginRequired,
    /// Suspicious activity detected
    SuspiciousActivity {
        challenge_url: Option<String>,
    },
    /// Parsing error
    ParseError {
        message: String,
    },
}

impl XError {
    /// Create a rate limit error
    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        // TODO: Implement rate limit error construction
        Self::RateLimited { retry_after }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        // TODO: Implement retryability check
        matches!(self, XError::RateLimited { .. })
    }
}

/// X-specific user metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XUserMetadata {
    /// X Premium/Blue subscriber
    pub is_blue_verified: bool,
    /// Verified business account
    pub is_business_verified: bool,
    /// Government verified
    pub is_gov_verified: bool,
    /// Legacy verified (pre-Blue)
    pub is_legacy_verified: bool,
    /// Followers you also follow
    pub followers_you_follow: Option<u64>,
    /// Can receive DMs
    pub can_dm: bool,
    /// Can be tagged in media
    pub can_media_tag: bool,
    /// Account creation timestamp
    pub created_at: Option<i64>,
    /// User-provided location
    pub location: Option<String>,
    /// Pinned tweet ID
    pub pinned_tweet_id: Option<String>,
    /// Listed count
    pub listed_count: Option<u64>,
    /// Profile URL
    pub url: Option<String>,
    /// Has custom profile banner
    pub has_banner: bool,
    /// Account type (normal, business, creator)
    pub account_type: Option<String>,
}

impl XUserMetadata {
    /// Create new user metadata with defaults
    pub fn new() -> Self {
        // TODO: Implement user metadata construction
        Self::default()
    }

    /// Check if user has any verification
    pub fn is_verified_any(&self) -> bool {
        // TODO: Implement verification check
        self.is_blue_verified
            || self.is_business_verified
            || self.is_gov_verified
            || self.is_legacy_verified
    }

    /// Get verification type as string
    pub fn verification_type(&self) -> Option<&'static str> {
        // TODO: Implement verification type determination
        if self.is_blue_verified {
            Some("blue")
        } else if self.is_business_verified {
            Some("business")
        } else if self.is_gov_verified {
            Some("government")
        } else if self.is_legacy_verified {
            Some("legacy")
        } else {
            None
        }
    }
}

/// X-specific post metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XPostMetadata {
    /// Source client (e.g., "Twitter Web App")
    pub source: Option<String>,
    /// Language code (BCP 47)
    pub lang: Option<String>,
    /// Is quote status
    pub is_quote_status: bool,
    /// Sensitive content flag
    pub possibly_sensitive: Option<bool>,
    /// Geographic coordinates
    pub coordinates: Option<Coordinates>,
    /// Place information
    pub place: Option<Place>,
    /// Extracted hashtags
    pub hashtags: Vec<HashtagEntity>,
    /// User mentions
    pub mentions: Vec<UserMention>,
    /// URL entities
    pub urls: Vec<UrlEntity>,
    /// Cashtag entities ($TICKER)
    pub cashtags: Vec<CashtagEntity>,
    /// Conversation/thread ID
    pub conversation_id: Option<String>,
    /// Bookmark count
    pub bookmark_count: Option<u64>,
    /// Edit history tweet IDs
    pub edit_history: Vec<String>,
    /// Edit window remaining (seconds)
    pub edit_remaining: Option<u64>,
    /// Is a note tweet (long form)
    pub is_note_tweet: bool,
    /// Reply settings
    pub reply_settings: Option<String>,
}

impl XPostMetadata {
    /// Create new post metadata
    pub fn new() -> Self {
        // TODO: Implement post metadata construction
        Self::default()
    }

    /// Check if this post has location data
    pub fn has_location(&self) -> bool {
        // TODO: Implement location check
        self.coordinates.is_some() || self.place.is_some()
    }

    /// Check if this post has entities
    pub fn has_entities(&self) -> bool {
        // TODO: Implement entities check
        !self.hashtags.is_empty()
            || !self.mentions.is_empty()
            || !self.urls.is_empty()
            || !self.cashtags.is_empty()
    }
}

/// Geographic coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Coordinate type (Point, Polygon)
    pub coord_type: String,
}

impl Coordinates {
    /// Create new coordinates
    pub fn new(latitude: f64, longitude: f64) -> Self {
        // TODO: Implement coordinates construction
        Self {
            latitude,
            longitude,
            coord_type: "Point".to_string(),
        }
    }

    /// Validate coordinate ranges
    pub fn validate(&self) -> Result<(), GhostError> {
        // TODO: Implement coordinate validation
        // - Latitude: -90 to 90
        // - Longitude: -180 to 180
        if self.latitude < -90.0 || self.latitude > 90.0 {
            return Err(GhostError::ValidationError("Invalid latitude".into()));
        }
        if self.longitude < -180.0 || self.longitude > 180.0 {
            return Err(GhostError::ValidationError("Invalid longitude".into()));
        }
        Ok(())
    }
}

/// Place/location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    /// Place ID
    pub id: String,
    /// Short name
    pub name: String,
    /// Full name with country
    pub full_name: String,
    /// Country name
    pub country: String,
    /// ISO country code
    pub country_code: String,
    /// Place type (poi, neighborhood, city, admin)
    pub place_type: String,
    /// Bounding box coordinates
    pub bounding_box: Option<Vec<Vec<Vec<f64>>>>,
}

impl Place {
    /// Create new place
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        // TODO: Implement place construction
        Self {
            id: id.into(),
            name: name.into(),
            full_name: String::new(),
            country: String::new(),
            country_code: String::new(),
            place_type: String::new(),
            bounding_box: None,
        }
    }
}

/// Hashtag entity with position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashtagEntity {
    /// Hashtag text (without #)
    pub tag: String,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
}

impl HashtagEntity {
    /// Create new hashtag entity
    pub fn new(tag: impl Into<String>, start: usize, end: usize) -> Self {
        // TODO: Implement hashtag entity construction
        Self {
            tag: tag.into(),
            start,
            end,
        }
    }
}

/// User mention entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMention {
    /// User ID
    pub id: String,
    /// Username (without @)
    pub username: String,
    /// Display name
    pub display_name: Option<String>,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
}

impl UserMention {
    /// Create new user mention
    pub fn new(id: impl Into<String>, username: impl Into<String>, start: usize, end: usize) -> Self {
        // TODO: Implement user mention construction
        Self {
            id: id.into(),
            username: username.into(),
            display_name: None,
            start,
            end,
        }
    }
}

/// URL entity with expanded URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlEntity {
    /// Shortened URL (t.co)
    pub url: String,
    /// Expanded/full URL
    pub expanded_url: String,
    /// Display-friendly URL
    pub display_url: String,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
    /// Unwound/final URL after redirects
    pub unwound_url: Option<String>,
    /// Link title (Open Graph)
    pub title: Option<String>,
    /// Link description (Open Graph)
    pub description: Option<String>,
    /// Link images (Open Graph)
    pub images: Option<Vec<UrlImage>>,
}

impl UrlEntity {
    /// Create new URL entity
    pub fn new(url: impl Into<String>, expanded_url: impl Into<String>, start: usize, end: usize) -> Self {
        // TODO: Implement URL entity construction
        Self {
            url: url.into(),
            expanded_url: expanded_url.into(),
            display_url: String::new(),
            start,
            end,
            unwound_url: None,
            title: None,
            description: None,
            images: None,
        }
    }
}

/// URL image from Open Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlImage {
    /// Image URL
    pub url: String,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
}

/// Cashtag entity ($TICKER)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashtagEntity {
    /// Ticker symbol (without $)
    pub tag: String,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
}

impl CashtagEntity {
    /// Create new cashtag entity
    pub fn new(tag: impl Into<String>, start: usize, end: usize) -> Self {
        // TODO: Implement cashtag entity construction
        Self {
            tag: tag.into(),
            start,
            end,
        }
    }
}

// ============================================================================
// Threads Specific Types
// ============================================================================

/// Threads API error types
#[derive(Debug, Clone)]
pub enum ThreadsError {
    /// Rate limited
    RateLimited {
        retry_after: Option<u64>,
    },
    /// Account suspended
    AccountSuspended {
        user_id: String,
    },
    /// Not found
    NotFound {
        resource_type: String,
        resource_id: String,
    },
    /// Private account
    PrivateAccount {
        user_id: String,
    },
    /// Login required
    LoginRequired,
    /// Checkpoint required
    Checkpoint {
        url: Option<String>,
    },
    /// Parsing error
    ParseError {
        message: String,
    },
}

impl ThreadsError {
    /// Create a rate limit error
    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        // TODO: Implement rate limit error construction
        Self::RateLimited { retry_after }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        // TODO: Implement retryability check
        matches!(self, ThreadsError::RateLimited { .. })
    }
}

/// Threads-specific user metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadsUserMetadata {
    /// Meta Verified subscription
    pub is_verified: bool,
    /// Is business account
    pub is_business_account: bool,
    /// Is creator account
    pub is_creator_account: bool,
    /// Has linked Instagram account
    pub has_linked_instagram: bool,
    /// Profile deep link URL
    pub profile_deep_link: Option<String>,
    /// Links in bio
    pub bio_links: Vec<BioLink>,
    /// Follower count (from insights)
    pub followers_count: Option<u64>,
    /// Following count (from insights)
    pub following_count: Option<u64>,
}

impl ThreadsUserMetadata {
    /// Create new user metadata
    pub fn new() -> Self {
        // TODO: Implement user metadata construction
        Self::default()
    }

    /// Check if this is a professional account
    pub fn is_professional(&self) -> bool {
        // TODO: Implement professional account check
        self.is_business_account || self.is_creator_account
    }
}

/// Link in user bio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioLink {
    /// Link URL
    pub url: String,
    /// Display text
    pub text: Option<String>,
    /// Link type (external, internal)
    pub link_type: Option<String>,
}

impl BioLink {
    /// Create new bio link
    pub fn new(url: impl Into<String>) -> Self {
        // TODO: Implement bio link construction
        Self {
            url: url.into(),
            text: None,
            link_type: None,
        }
    }
}

/// Threads-specific post metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadsPostMetadata {
    /// Post type classification
    pub post_type: ThreadsPostType,
    /// Video has audio track
    pub has_audio: bool,
    /// Is a reel video
    pub is_reel: bool,
    /// Language code
    pub lang: Option<String>,
    /// Reply audience restriction
    pub reply_audience: Option<ReplyAudience>,
    /// Hide status
    pub hide_status: Option<HideStatus>,
    /// Short URL identifier
    pub shortcode: Option<String>,
    /// Product type (THREADS, THREADS_REEL)
    pub media_product_type: Option<String>,
    /// Instagram crosspost flag
    pub is_shared_to_feed: Option<bool>,
    /// Hashtags extracted from text
    pub hashtags: Vec<String>,
    /// User mentions
    pub mentions: Vec<ThreadsMention>,
    /// Links in text
    pub links: Vec<LinkEntity>,
}

impl ThreadsPostMetadata {
    /// Create new post metadata
    pub fn new() -> Self {
        // TODO: Implement post metadata construction
        Self::default()
    }

    /// Check if this is a media post
    pub fn is_media_post(&self) -> bool {
        // TODO: Implement media post check
        !matches!(self.post_type, ThreadsPostType::Text)
    }

    /// Check if replies are restricted
    pub fn has_restricted_replies(&self) -> bool {
        // TODO: Implement reply restriction check
        self.reply_audience
            .as_ref()
            .map(|a| !matches!(a, ReplyAudience::Everyone))
            .unwrap_or(false)
    }
}

/// Type of Threads post
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThreadsPostType {
    /// Text-only post
    #[default]
    Text,
    /// Single image
    Image,
    /// Single video
    Video,
    /// Carousel (multiple media)
    Carousel,
    /// Reel video
    Reel,
}

impl ThreadsPostType {
    /// Parse from Threads API media_type field
    pub fn from_api(media_type: &str) -> Self {
        // TODO: Implement post type parsing
        match media_type {
            "TEXT" => ThreadsPostType::Text,
            "IMAGE" => ThreadsPostType::Image,
            "VIDEO" => ThreadsPostType::Video,
            "CAROUSEL" => ThreadsPostType::Carousel,
            "THREADS_REEL" => ThreadsPostType::Reel,
            _ => ThreadsPostType::Text,
        }
    }
}

/// Reply audience restriction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplyAudience {
    /// Anyone can reply
    Everyone,
    /// Only mentioned users can reply
    Mentions,
    /// Only followers can reply
    Followers,
}

/// Post visibility status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HideStatus {
    /// Post is hidden
    Hidden,
    /// Post is visible
    Shown,
}

/// User mention in Threads post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsMention {
    /// User ID
    pub id: String,
    /// Username (without @)
    pub username: String,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
}

impl ThreadsMention {
    /// Create new mention
    pub fn new(id: impl Into<String>, username: impl Into<String>, start: usize, end: usize) -> Self {
        // TODO: Implement mention construction
        Self {
            id: id.into(),
            username: username.into(),
            start,
            end,
        }
    }
}

/// Link entity in Threads post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntity {
    /// URL
    pub url: String,
    /// Display text
    pub text: Option<String>,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
}

impl LinkEntity {
    /// Create new link entity
    pub fn new(url: impl Into<String>, start: usize, end: usize) -> Self {
        // TODO: Implement link entity construction
        Self {
            url: url.into(),
            text: None,
            start,
            end,
        }
    }
}

/// Threads authentication tokens
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadsAuth {
    /// LSD token (Meta's CSRF token)
    pub lsd_token: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Bearer token
    pub bearer_token: Option<String>,
    /// Device ID
    pub device_id: Option<String>,
    /// Access token (OAuth)
    pub access_token: Option<String>,
    /// Refresh token
    pub refresh_token: Option<String>,
    /// Token expiration timestamp
    pub expires_at: Option<i64>,
}

impl ThreadsAuth {
    /// Create new auth info
    pub fn new() -> Self {
        // TODO: Implement auth construction
        Self::default()
    }

    /// Create auth from cookie string
    pub fn from_cookies(cookies: &str) -> Self {
        // TODO: Implement cookie parsing
        // - Extract session_id from cookie string
        // - Extract lsd_token if present
        let _ = cookies;
        Self::default()
    }

    /// Create auth from OAuth access token
    pub fn from_access_token(token: impl Into<String>) -> Self {
        // TODO: Implement OAuth auth construction
        Self {
            access_token: Some(token.into()),
            ..Default::default()
        }
    }

    /// Validate auth is complete
    pub fn is_valid(&self) -> bool {
        // TODO: Implement auth validation
        // - Must have either session_id or access_token
        self.session_id.is_some() || self.access_token.is_some()
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        // TODO: Implement expiration check
        self.expires_at
            .map(|exp| exp < chrono::Utc::now().timestamp())
            .unwrap_or(false)
    }

    /// Check if auth can be refreshed
    pub fn can_refresh(&self) -> bool {
        // TODO: Implement refresh capability check
        self.refresh_token.is_some()
    }
}

// ============================================================================
// API Response Types - X (Twitter)
// ============================================================================

/// X API v2 tweet response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XTweetResponse {
    /// Primary tweet data
    pub data: Option<XTweetData>,
    /// Expanded objects
    pub includes: Option<XIncludes>,
    /// Error objects
    pub errors: Option<Vec<XErrorObject>>,
    /// Pagination metadata
    pub meta: Option<XMeta>,
}

/// X tweet data object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XTweetData {
    /// Tweet ID
    pub id: String,
    /// Tweet text
    pub text: String,
    /// Author user ID
    pub author_id: Option<String>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Public metrics
    pub public_metrics: Option<XTweetMetrics>,
    /// Referenced tweets
    pub referenced_tweets: Option<Vec<XReferencedTweet>>,
    /// Attachments
    pub attachments: Option<XAttachments>,
    /// Entities
    pub entities: Option<XEntities>,
    /// Language code
    pub lang: Option<String>,
    /// Source client
    pub source: Option<String>,
    /// Conversation ID
    pub conversation_id: Option<String>,
    /// Sensitive content flag
    pub possibly_sensitive: Option<bool>,
    /// Reply settings
    pub reply_settings: Option<String>,
    /// Geographic data
    pub geo: Option<XGeo>,
    /// Withheld content info
    pub withheld: Option<XWithheld>,
    /// Context annotations
    pub context_annotations: Option<Vec<XContextAnnotation>>,
    /// Edit history
    pub edit_history_tweet_ids: Option<Vec<String>>,
    /// Edit controls
    pub edit_controls: Option<XEditControls>,
}

/// X tweet metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XTweetMetrics {
    /// Like count
    pub like_count: u64,
    /// Retweet count
    pub retweet_count: u64,
    /// Reply count
    pub reply_count: u64,
    /// Quote count
    pub quote_count: u64,
    /// Impression/view count
    pub impression_count: u64,
    /// Bookmark count
    pub bookmark_count: Option<u64>,
}

impl Default for XTweetMetrics {
    fn default() -> Self {
        Self {
            like_count: 0,
            retweet_count: 0,
            reply_count: 0,
            quote_count: 0,
            impression_count: 0,
            bookmark_count: None,
        }
    }
}

/// X referenced tweet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XReferencedTweet {
    /// Reference type (replied_to, quoted, retweeted)
    #[serde(rename = "type")]
    pub reference_type: String,
    /// Referenced tweet ID
    pub id: String,
}

/// X tweet attachments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XAttachments {
    /// Media keys
    pub media_keys: Option<Vec<String>>,
    /// Poll IDs
    pub poll_ids: Option<Vec<String>>,
}

/// X entities (parsed text elements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XEntities {
    /// Hashtags
    pub hashtags: Option<Vec<XHashtag>>,
    /// User mentions
    pub mentions: Option<Vec<XMention>>,
    /// URLs
    pub urls: Option<Vec<XUrl>>,
    /// Cashtags
    pub cashtags: Option<Vec<XCashtag>>,
    /// Annotations (NLP entities)
    pub annotations: Option<Vec<XAnnotation>>,
}

/// X hashtag entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XHashtag {
    /// Tag text
    pub tag: String,
    /// Start position
    pub start: usize,
    /// End position
    pub end: usize,
}

/// X mention entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMention {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Start position
    pub start: usize,
    /// End position
    pub end: usize,
}

/// X URL entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUrl {
    /// t.co URL
    pub url: String,
    /// Expanded URL
    pub expanded_url: String,
    /// Display URL
    pub display_url: String,
    /// Start position
    pub start: usize,
    /// End position
    pub end: usize,
    /// Unwound URL info
    pub unwound_url: Option<String>,
    /// Images from Open Graph
    pub images: Option<Vec<XUrlImage>>,
    /// Title from Open Graph
    pub title: Option<String>,
    /// Description from Open Graph
    pub description: Option<String>,
}

/// X URL image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUrlImage {
    /// Image URL
    pub url: String,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

/// X cashtag entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XCashtag {
    /// Ticker symbol
    pub tag: String,
    /// Start position
    pub start: usize,
    /// End position
    pub end: usize,
}

/// X annotation entity (NLP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XAnnotation {
    /// Probability score
    pub probability: f64,
    /// Entity type
    #[serde(rename = "type")]
    pub annotation_type: String,
    /// Normalized text
    pub normalized_text: String,
}

/// X geographic data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XGeo {
    /// Place ID
    pub place_id: Option<String>,
    /// Coordinates
    pub coordinates: Option<XCoordinates>,
}

/// X coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XCoordinates {
    /// Coordinate type
    #[serde(rename = "type")]
    pub coord_type: String,
    /// [longitude, latitude]
    pub coordinates: Vec<f64>,
}

/// X withheld info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XWithheld {
    /// Country codes where withheld
    pub country_codes: Vec<String>,
    /// Scope (tweet, user)
    pub scope: Option<String>,
}

/// X context annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XContextAnnotation {
    /// Domain info
    pub domain: XContextDomain,
    /// Entity info
    pub entity: XContextEntity,
}

/// X context domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XContextDomain {
    /// Domain ID
    pub id: String,
    /// Domain name
    pub name: String,
    /// Domain description
    pub description: Option<String>,
}

/// X context entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XContextEntity {
    /// Entity ID
    pub id: String,
    /// Entity name
    pub name: String,
    /// Entity description
    pub description: Option<String>,
}

/// X edit controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XEditControls {
    /// Edit window remaining (seconds)
    pub edits_remaining: u64,
    /// Whether edit is eligible
    pub is_edit_eligible: bool,
}

/// X includes (expanded objects)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XIncludes {
    /// Expanded users
    pub users: Option<Vec<XUserData>>,
    /// Expanded media
    pub media: Option<Vec<XMediaData>>,
    /// Expanded tweets
    pub tweets: Option<Vec<XTweetData>>,
    /// Expanded places
    pub places: Option<Vec<XPlaceData>>,
    /// Expanded polls
    pub polls: Option<Vec<XPollData>>,
}

/// X user data object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUserData {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Display name
    pub name: String,
    /// Profile description
    pub description: Option<String>,
    /// Profile image URL
    pub profile_image_url: Option<String>,
    /// Profile URL
    pub url: Option<String>,
    /// Location
    pub location: Option<String>,
    /// Protected/private flag
    pub protected: Option<bool>,
    /// Verified flag
    pub verified: Option<bool>,
    /// Verification type
    pub verified_type: Option<String>,
    /// Public metrics
    pub public_metrics: Option<XUserMetrics>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Pinned tweet ID
    pub pinned_tweet_id: Option<String>,
    /// Entities in profile
    pub entities: Option<XUserEntities>,
    /// Withheld info
    pub withheld: Option<XWithheld>,
}

/// X user metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUserMetrics {
    /// Followers count
    pub followers_count: u64,
    /// Following count
    pub following_count: u64,
    /// Tweets count
    pub tweet_count: u64,
    /// Listed count
    pub listed_count: u64,
}

impl Default for XUserMetrics {
    fn default() -> Self {
        Self {
            followers_count: 0,
            following_count: 0,
            tweet_count: 0,
            listed_count: 0,
        }
    }
}

/// X user entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUserEntities {
    /// URL entities
    pub url: Option<XUserUrl>,
    /// Description entities
    pub description: Option<XEntities>,
}

/// X user URL entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUserUrl {
    /// URLs in profile
    pub urls: Vec<XUrl>,
}

/// X media data object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMediaData {
    /// Media key
    pub media_key: String,
    /// Media type
    #[serde(rename = "type")]
    pub media_type: String,
    /// Media URL (for images)
    pub url: Option<String>,
    /// Preview URL (for videos)
    pub preview_image_url: Option<String>,
    /// Alt text
    pub alt_text: Option<String>,
    /// Width
    pub width: Option<u32>,
    /// Height
    pub height: Option<u32>,
    /// Duration (ms)
    pub duration_ms: Option<u64>,
    /// Public metrics
    pub public_metrics: Option<XMediaMetrics>,
    /// Video variants
    pub variants: Option<Vec<XVideoVariant>>,
}

/// X media metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMediaMetrics {
    /// View count
    pub view_count: u64,
}

/// X video variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XVideoVariant {
    /// Bit rate
    pub bit_rate: Option<u32>,
    /// Content type
    pub content_type: String,
    /// URL
    pub url: String,
}

/// X place data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPlaceData {
    /// Place ID
    pub id: String,
    /// Place name
    pub name: String,
    /// Full name
    pub full_name: String,
    /// Country
    pub country: String,
    /// Country code
    pub country_code: String,
    /// Place type
    pub place_type: String,
    /// Geo (bounding box)
    pub geo: Option<XPlaceGeo>,
}

/// X place geo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPlaceGeo {
    /// Type
    #[serde(rename = "type")]
    pub geo_type: String,
    /// Bounding box
    pub bbox: Option<Vec<f64>>,
}

/// X poll data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPollData {
    /// Poll ID
    pub id: String,
    /// Poll options
    pub options: Vec<XPollOption>,
    /// Voting status
    pub voting_status: String,
    /// End datetime
    pub end_datetime: Option<String>,
    /// Duration minutes
    pub duration_minutes: Option<u32>,
}

/// X poll option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPollOption {
    /// Position
    pub position: u32,
    /// Label text
    pub label: String,
    /// Vote count
    pub votes: Option<u64>,
}

/// X pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMeta {
    /// Result count
    pub result_count: Option<u32>,
    /// Next page token
    pub next_token: Option<String>,
    /// Previous page token
    pub previous_token: Option<String>,
    /// Newest ID in results
    pub newest_id: Option<String>,
    /// Oldest ID in results
    pub oldest_id: Option<String>,
}

/// X error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XErrorObject {
    /// Error detail
    pub detail: Option<String>,
    /// Error title
    pub title: Option<String>,
    /// Error type URL
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    /// Resource type
    pub resource_type: Option<String>,
    /// Parameter
    pub parameter: Option<String>,
    /// Resource ID
    pub resource_id: Option<String>,
    /// Error value
    pub value: Option<String>,
}

// ============================================================================
// API Response Types - Threads
// ============================================================================

/// Threads media container response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsMediaContainer {
    /// Container ID
    pub id: String,
    /// Media type
    pub media_type: Option<String>,
    /// Media product type
    pub media_product_type: Option<String>,
    /// Text content
    pub text: Option<String>,
    /// Media URL
    pub media_url: Option<String>,
    /// Thumbnail URL
    pub thumbnail_url: Option<String>,
    /// Timestamp
    pub timestamp: Option<String>,
    /// Owner user
    pub owner: Option<ThreadsOwner>,
    /// Children (for carousel)
    pub children: Option<Vec<ThreadsMediaChild>>,
    /// Like count
    pub likes_count: Option<u64>,
    /// Repost count
    pub reposts_count: Option<u64>,
    /// Reply count
    pub replies_count: Option<u64>,
    /// Quote count
    pub quotes_count: Option<u64>,
    /// Is reply
    pub is_reply: Option<bool>,
    /// Is quote post
    pub is_quote_post: Option<bool>,
    /// Quoted post
    pub quoted_post: Option<Box<ThreadsMediaContainer>>,
    /// Reposted post
    pub reposted_post: Option<Box<ThreadsMediaContainer>>,
    /// Reply audience
    pub reply_audience: Option<String>,
    /// Hide status
    pub hide_status: Option<String>,
    /// Permalink
    pub permalink: Option<String>,
    /// Shortcode
    pub shortcode: Option<String>,
    /// Has audio
    pub has_audio: Option<bool>,
    /// Is shared to feed
    pub is_shared_to_feed: Option<bool>,
}

/// Threads owner user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsOwner {
    /// User ID
    pub id: String,
    /// Username
    pub username: Option<String>,
}

/// Threads media child (for carousel)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsMediaChild {
    /// Child ID
    pub id: String,
    /// Media type
    pub media_type: Option<String>,
    /// Media URL
    pub media_url: Option<String>,
    /// Thumbnail URL
    pub thumbnail_url: Option<String>,
}

/// Threads user response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsUserResponse {
    /// User ID
    pub id: String,
    /// Username
    pub username: Option<String>,
    /// Display name
    pub name: Option<String>,
    /// Profile picture URL
    pub threads_profile_picture_url: Option<String>,
    /// Bio text
    pub threads_biography: Option<String>,
    /// Profile URL
    pub profile_url: Option<String>,
    /// Verified status
    pub is_verified: Option<bool>,
    /// Followers count
    pub followers_count: Option<u64>,
}

/// Threads insights response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsInsightsResponse {
    /// Insight data points
    pub data: Vec<ThreadsInsight>,
}

/// Threads insight data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsInsight {
    /// Metric name
    pub name: String,
    /// Time period
    pub period: String,
    /// Values
    pub values: Vec<ThreadsInsightValue>,
    /// Title
    pub title: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// Threads insight value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsInsightValue {
    /// Value
    pub value: u64,
    /// End time (for time-series)
    pub end_time: Option<String>,
}

/// Threads list response (paginated)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsListResponse {
    /// Data items
    pub data: Vec<ThreadsMediaContainer>,
    /// Pagination info
    pub paging: Option<ThreadsPaging>,
}

/// Threads pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsPaging {
    /// Cursors
    pub cursors: Option<ThreadsCursors>,
    /// Next page URL
    pub next: Option<String>,
    /// Previous page URL
    pub previous: Option<String>,
}

/// Threads cursors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsCursors {
    /// Before cursor
    pub before: Option<String>,
    /// After cursor
    pub after: Option<String>,
}

/// Threads error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsErrorResponse {
    /// Error object
    pub error: ThreadsErrorObject,
}

/// Threads error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsErrorObject {
    /// Error message
    pub message: String,
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,
    /// Error code
    pub code: u16,
    /// User-friendly message
    pub error_user_msg: Option<String>,
    /// FB trace ID
    pub fbtrace_id: Option<String>,
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x_user_metadata_verification() {
        // TODO: Implement test
        let mut meta = XUserMetadata::new();
        meta.is_blue_verified = true;
        assert!(meta.is_verified_any());
    }

    #[test]
    fn test_threads_post_type_parsing() {
        // TODO: Implement test
        assert_eq!(ThreadsPostType::from_api("IMAGE"), ThreadsPostType::Image);
        assert_eq!(ThreadsPostType::from_api("VIDEO"), ThreadsPostType::Video);
    }

    #[test]
    fn test_adapter_error_retryable() {
        // TODO: Implement test
        let error = AdapterError::rate_limited(Platform::X, Some(60));
        assert!(error.is_retryable());
    }

    #[test]
    fn test_threads_auth_validation() {
        // TODO: Implement test
        let auth = ThreadsAuth::from_access_token("test_token");
        assert!(auth.is_valid());
    }
}
