//! Adapter types for Ghost API
//!
//! This module contains all types shared between platform adapters,
//! including X (Twitter) and Threads adapters.

use serde::{Deserialize, Serialize};

use crate::{GhostPost, GhostUser, Platform};

// ============================================================================
// Common Adapter Types
// ============================================================================

/// Generic result from parsing platform responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdapterParseResult {
    /// Single user profile
    User(GhostUser),
    /// Single post
    Post(GhostPost),
    /// Multiple posts
    Posts(Vec<GhostPost>),
    /// Search results with pagination
    Search {
        posts: Vec<GhostPost>,
        cursor: Option<String>,
    },
    /// Timeline/feed
    Timeline {
        posts: Vec<GhostPost>,
        cursor_top: Option<String>,
        cursor_bottom: Option<String>,
    },
    /// Trending topics
    Trending(Vec<TrendingTopic>),
    /// Thread/conversation
    Thread {
        posts: Vec<GhostPost>,
        cursor: Option<String>,
    },
    /// Error response
    Error(AdapterError),
}

impl AdapterParseResult {
    /// Returns the posts if this is a posts result
    pub fn into_posts(self) -> Option<Vec<GhostPost>> {
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
    pub fn into_post(self) -> Option<GhostPost> {
        // TODO: Implement post extraction
        match self {
            AdapterParseResult::Post(post) => Some(post),
            _ => None,
        }
    }

    /// Returns the user if this is a user result
    pub fn into_user(self) -> Option<GhostUser> {
        // TODO: Implement user extraction
        match self {
            AdapterParseResult::User(user) => Some(user),
            _ => None,
        }
    }

    /// Returns true if this is an error
    pub fn is_error(&self) -> bool {
        // TODO: Implement error check
        matches!(self, AdapterParseResult::Error(_))
    }
}

/// Generic adapter error
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum AdapterError {
    /// Rate limited
    RateLimited {
        retry_after: Option<u64>,
    },
    /// Account suspended
    AccountSuspended {
        user_id: String,
    },
    /// Resource not found
    NotFound {
        resource_type: String,
        resource_id: String,
    },
    /// Protected/private account
    ProtectedAccount {
        user_id: String,
    },
    /// Login required
    LoginRequired,
    /// Challenge/checkpoint required
    ChallengeRequired {
        challenge_type: String,
        challenge_url: Option<String>,
    },
    /// Suspicious activity detected
    SuspiciousActivity {
        challenge_url: Option<String>,
    },
    /// Parsing error
    ParseError {
        message: String,
    },
}

impl AdapterError {
    /// Creates a rate limit error
    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        // TODO: Implement rate limit error construction
        Self::RateLimited { retry_after }
    }

    /// Creates a not found error
    pub fn not_found(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        // TODO: Implement not found error construction
        Self::NotFound {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
        }
    }

    /// Creates a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        // TODO: Implement parse error construction
        Self::ParseError {
            message: message.into(),
        }
    }

    /// Returns whether this error is retryable
    pub fn is_retryable(&self) -> bool {
        // TODO: Implement retryability check
        matches!(self, AdapterError::RateLimited { .. })
    }

    /// Returns whether this requires authentication
    pub fn requires_auth(&self) -> bool {
        // TODO: Implement auth requirement check
        matches!(
            self,
            AdapterError::LoginRequired | AdapterError::ChallengeRequired { .. }
        )
    }
}

/// Trending topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingTopic {
    /// Topic name/hashtag
    pub name: String,
    /// Topic URL
    pub url: String,
    /// Tweet/post volume
    pub post_count: Option<u64>,
    /// Description
    pub description: Option<String>,
    /// Trending rank
    pub rank: Option<u32>,
}

impl TrendingTopic {
    /// Creates a new trending topic
    pub fn new(name: impl Into<String>) -> Self {
        // TODO: Implement trending topic construction
        Self {
            name: name.into(),
            url: String::new(),
            post_count: None,
            description: None,
            rank: None,
        }
    }

    /// Sets the URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        // TODO: Implement URL setter
        self.url = url.into();
        self
    }

    /// Sets the post count
    pub fn with_post_count(mut self, count: u64) -> Self {
        // TODO: Implement post count setter
        self.post_count = Some(count);
        self
    }

    /// Sets the rank
    pub fn with_rank(mut self, rank: u32) -> Self {
        // TODO: Implement rank setter
        self.rank = Some(rank);
        self
    }
}

// ============================================================================
// Entity Types (shared between platforms)
// ============================================================================

/// Geographic coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Coordinate type (Point, Polygon, etc.)
    pub coord_type: String,
}

impl Coordinates {
    /// Creates new coordinates
    pub fn new(latitude: f64, longitude: f64) -> Self {
        // TODO: Implement coordinates construction
        Self {
            latitude,
            longitude,
            coord_type: "Point".to_string(),
        }
    }
}

/// Place information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    /// Place ID
    pub id: String,
    /// Place name
    pub name: String,
    /// Full name with country
    pub full_name: String,
    /// Country name
    pub country: String,
    /// Country code (ISO)
    pub country_code: String,
    /// Place type (city, admin, country)
    pub place_type: String,
}

impl Place {
    /// Creates a new place
    pub fn new(name: impl Into<String>) -> Self {
        // TODO: Implement place construction
        Self {
            id: String::new(),
            name: name.into(),
            full_name: String::new(),
            country: String::new(),
            country_code: String::new(),
            place_type: String::new(),
        }
    }
}

/// User mention in content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMention {
    /// User ID
    pub id: String,
    /// Username/handle
    pub username: String,
    /// Display name
    pub display_name: Option<String>,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
}

impl UserMention {
    /// Creates a new user mention
    pub fn new(id: impl Into<String>, username: impl Into<String>) -> Self {
        // TODO: Implement user mention construction
        Self {
            id: id.into(),
            username: username.into(),
            display_name: None,
            start: 0,
            end: 0,
        }
    }

    /// Sets the positions
    pub fn with_positions(mut self, start: usize, end: usize) -> Self {
        // TODO: Implement position setter
        self.start = start;
        self.end = end;
        self
    }
}

/// URL entity in content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlEntity {
    /// Shortened URL
    pub url: String,
    /// Expanded URL
    pub expanded_url: String,
    /// Display URL
    pub display_url: String,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
}

impl UrlEntity {
    /// Creates a new URL entity
    pub fn new(url: impl Into<String>) -> Self {
        // TODO: Implement URL entity construction
        Self {
            url: url.into(),
            expanded_url: String::new(),
            display_url: String::new(),
            start: 0,
            end: 0,
        }
    }
}

/// Hashtag entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashtagEntity {
    /// Hashtag text (without #)
    pub text: String,
    /// Start position
    pub start: usize,
    /// End position
    pub end: usize,
}

impl HashtagEntity {
    /// Creates a new hashtag entity
    pub fn new(text: impl Into<String>) -> Self {
        // TODO: Implement hashtag construction
        Self {
            text: text.into(),
            start: 0,
            end: 0,
        }
    }
}

/// Link entity (generic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntity {
    /// URL
    pub url: String,
    /// Display text
    pub display_text: Option<String>,
    /// Start position
    pub start: usize,
    /// End position
    pub end: usize,
}

impl LinkEntity {
    /// Creates a new link entity
    pub fn new(url: impl Into<String>) -> Self {
        // TODO: Implement link entity construction
        Self {
            url: url.into(),
            display_text: None,
            start: 0,
            end: 0,
        }
    }
}

// ============================================================================
// X (Twitter) Specific Types
// ============================================================================

/// X-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
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
    /// Creates a rate limit error
    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        // TODO: Implement rate limit error construction
        Self::RateLimited { retry_after }
    }
}

/// X-specific user metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUserMetadata {
    /// Is blue verified
    pub is_blue_verified: bool,
    /// Is business verified
    pub is_business_verified: bool,
    /// Is government verified
    pub is_gov_verified: bool,
    /// Legacy verified
    pub is_legacy_verified: bool,
    /// Followers you follow count
    pub followers_you_follow: Option<u64>,
    /// Can DM
    pub can_dm: bool,
    /// Can media tag
    pub can_media_tag: bool,
    /// Profile created at
    pub created_at: Option<i64>,
    /// Location
    pub location: Option<String>,
    /// Pinned tweet ID
    pub pinned_tweet_id: Option<String>,
}

impl XUserMetadata {
    /// Creates new user metadata
    pub fn new() -> Self {
        // TODO: Implement user metadata construction
        Self {
            is_blue_verified: false,
            is_business_verified: false,
            is_gov_verified: false,
            is_legacy_verified: false,
            followers_you_follow: None,
            can_dm: true,
            can_media_tag: true,
            created_at: None,
            location: None,
            pinned_tweet_id: None,
        }
    }

    /// Returns whether user has any verification
    pub fn is_verified_any(&self) -> bool {
        // TODO: Implement verification check
        self.is_blue_verified || self.is_business_verified || self.is_gov_verified || self.is_legacy_verified
    }
}

impl Default for XUserMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// X-specific post metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPostMetadata {
    /// Source client
    pub source: Option<String>,
    /// Language
    pub lang: Option<String>,
    /// Is quote status
    pub is_quote_status: bool,
    /// Is possibly sensitive
    pub possibly_sensitive: Option<bool>,
    /// Coordinates
    pub coordinates: Option<Coordinates>,
    /// Place
    pub place: Option<Place>,
    /// Hashtags
    pub hashtags: Vec<String>,
    /// User mentions
    pub mentions: Vec<UserMention>,
    /// URLs
    pub urls: Vec<UrlEntity>,
    /// Conversation ID
    pub conversation_id: Option<String>,
}

impl XPostMetadata {
    /// Creates new post metadata
    pub fn new() -> Self {
        // TODO: Implement post metadata construction
        Self {
            source: None,
            lang: None,
            is_quote_status: false,
            possibly_sensitive: None,
            coordinates: None,
            place: None,
            hashtags: Vec::new(),
            mentions: Vec::new(),
            urls: Vec::new(),
            conversation_id: None,
        }
    }
}

impl Default for XPostMetadata {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Threads Specific Types
// ============================================================================

/// Threads-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
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
    /// Creates a rate limit error
    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        // TODO: Implement rate limit error construction
        Self::RateLimited { retry_after }
    }
}

/// Threads-specific user metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsUserMetadata {
    /// Is verified (blue check)
    pub is_verified: bool,
    /// Is business account
    pub is_business_account: bool,
    /// Is creator account
    pub is_creator_account: bool,
    /// Has linked Instagram
    pub has_linked_instagram: bool,
    /// Profile deep link
    pub profile_deep_link: Option<String>,
    /// Bio links
    pub bio_links: Vec<BioLink>,
}

impl ThreadsUserMetadata {
    /// Creates new user metadata
    pub fn new() -> Self {
        // TODO: Implement user metadata construction
        Self {
            is_verified: false,
            is_business_account: false,
            is_creator_account: false,
            has_linked_instagram: false,
            profile_deep_link: None,
            bio_links: Vec::new(),
        }
    }
}

impl Default for ThreadsUserMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Link in user bio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioLink {
    /// Link URL
    pub url: String,
    /// Link text
    pub text: Option<String>,
}

impl BioLink {
    /// Creates a new bio link
    pub fn new(url: impl Into<String>) -> Self {
        // TODO: Implement bio link construction
        Self {
            url: url.into(),
            text: None,
        }
    }
}

/// Threads-specific post metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsPostMetadata {
    /// Post type
    pub post_type: ThreadsPostType,
    /// Has audio
    pub has_audio: bool,
    /// Is reel
    pub is_reel: bool,
    /// Language code
    pub lang: Option<String>,
    /// Hashtags
    pub hashtags: Vec<String>,
    /// Mentions
    pub mentions: Vec<UserMention>,
    /// Links
    pub links: Vec<LinkEntity>,
}

impl ThreadsPostMetadata {
    /// Creates new post metadata
    pub fn new() -> Self {
        // TODO: Implement post metadata construction
        Self {
            post_type: ThreadsPostType::Text,
            has_audio: false,
            is_reel: false,
            lang: None,
            hashtags: Vec::new(),
            mentions: Vec::new(),
            links: Vec::new(),
        }
    }
}

impl Default for ThreadsPostMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of Threads post
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadsPostType {
    /// Text-only post
    Text,
    /// Image post
    Image,
    /// Video post
    Video,
    /// Carousel post
    Carousel,
    /// Reel
    Reel,
}

/// Threads API authentication tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadsAuth {
    /// LSD token
    pub lsd_token: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Bearer token
    pub bearer_token: Option<String>,
    /// Device ID
    pub device_id: Option<String>,
}

impl ThreadsAuth {
    /// Creates new auth info
    pub fn new() -> Self {
        // TODO: Implement auth construction
        Self {
            lsd_token: None,
            session_id: None,
            bearer_token: None,
            device_id: None,
        }
    }

    /// Creates auth from cookie string
    pub fn from_cookies(_cookies: &str) -> Self {
        // TODO: Implement cookie parsing
        Self::new()
    }

    /// Validates auth is complete
    pub fn is_valid(&self) -> bool {
        // TODO: Implement auth validation
        self.session_id.is_some() || self.bearer_token.is_some()
    }
}

impl Default for ThreadsAuth {
    fn default() -> Self {
        Self::new()
    }
}
