//! X-specific types and result structures

use ghost_schema::{GhostPost, GhostUser};

/// Result of parsing an X response
#[derive(Debug, Clone)]
pub enum XParseResult {
    /// Single user profile
    User(GhostUser),
    /// Single post/tweet
    Post(GhostPost),
    /// Multiple posts
    Posts(Vec<GhostPost>),
    /// Search results
    Search {
        posts: Vec<GhostPost>,
        cursor: Option<String>,
    },
    /// Timeline
    Timeline {
        posts: Vec<GhostPost>,
        cursor_top: Option<String>,
        cursor_bottom: Option<String>,
    },
    /// Trending topics
    Trending(Vec<TrendingTopic>),
    /// Error response
    Error(XError),
}

impl XParseResult {
    /// Returns the posts if this is a posts result
    pub fn into_posts(self) -> Option<Vec<GhostPost>> {
        // TODO: Implement posts extraction
        match self {
            XParseResult::Posts(posts) => Some(posts),
            XParseResult::Search { posts, .. } => Some(posts),
            XParseResult::Timeline { posts, .. } => Some(posts),
            _ => None,
        }
    }

    /// Returns the single post if this is a post result
    pub fn into_post(self) -> Option<GhostPost> {
        // TODO: Implement post extraction
        match self {
            XParseResult::Post(post) => Some(post),
            _ => None,
        }
    }

    /// Returns the user if this is a user result
    pub fn into_user(self) -> Option<GhostUser> {
        // TODO: Implement user extraction
        match self {
            XParseResult::User(user) => Some(user),
            _ => None,
        }
    }
}

/// X-specific error types
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
    /// Creates a rate limit error
    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        // TODO: Implement rate limit error construction
        Self::RateLimited { retry_after }
    }

    /// Returns whether this error is retryable
    pub fn is_retryable(&self) -> bool {
        // TODO: Implement retryability check
        matches!(self, XError::RateLimited { .. })
    }
}

/// Trending topic
#[derive(Debug, Clone)]
pub struct TrendingTopic {
    /// Topic name
    pub name: String,
    /// Topic URL
    pub url: String,
    /// Tweet volume
    pub tweet_count: Option<u64>,
    /// Description
    pub description: Option<String>,
}

impl TrendingTopic {
    /// Creates a new trending topic
    pub fn new(name: impl Into<String>) -> Self {
        // TODO: Implement trending topic construction
        Self {
            name: name.into(),
            url: String::new(),
            tweet_count: None,
            description: None,
        }
    }
}

/// X-specific user metadata
#[derive(Debug, Clone)]
pub struct XUserMetadata {
    /// Is blue verified
    pub is_blue_verified: bool,
    /// Is business verified
    pub is_business_verified: bool,
    /// Is government verified
    pub is_gov_verified: bool,
    /// Legacy verified
    pub is_legacy_verified: bool,
    /// Followers you follow
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
#[derive(Debug, Clone)]
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

/// Geographic coordinates
#[derive(Debug, Clone)]
pub struct Coordinates {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Type
    pub coord_type: String,
}

/// Place information
#[derive(Debug, Clone)]
pub struct Place {
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
}

/// User mention in a tweet
#[derive(Debug, Clone)]
pub struct UserMention {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Display name
    pub name: Option<String>,
    /// Indices in text
    pub indices: (usize, usize),
}

/// URL entity in a tweet
#[derive(Debug, Clone)]
pub struct UrlEntity {
    /// URL as displayed
    pub url: String,
    /// Expanded URL
    pub expanded_url: String,
    /// Display URL
    pub display_url: String,
    /// Indices in text
    pub indices: (usize, usize),
}
