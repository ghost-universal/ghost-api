//! Threads-specific types and result structures

use ghost_schema::{GhostPost, GhostUser};

/// Result of parsing a Threads response
#[derive(Debug, Clone)]
pub enum ThreadsParseResult {
    /// Single user profile
    User(GhostUser),
    /// Single post
    Post(GhostPost),
    /// Multiple posts
    Posts(Vec<GhostPost>),
    /// Thread (conversation)
    Thread {
        posts: Vec<GhostPost>,
        cursor: Option<String>,
    },
    /// Timeline feed
    Timeline {
        posts: Vec<GhostPost>,
        cursor: Option<String>,
    },
    /// Search results
    Search {
        posts: Vec<GhostPost>,
        cursor: Option<String>,
    },
    /// Error response
    Error(ThreadsError),
}

impl ThreadsParseResult {
    /// Returns the posts if this is a posts result
    pub fn into_posts(self) -> Option<Vec<GhostPost>> {
        // TODO: Implement posts extraction
        match self {
            ThreadsParseResult::Posts(posts) => Some(posts),
            ThreadsParseResult::Thread { posts, .. } => Some(posts),
            ThreadsParseResult::Timeline { posts, .. } => Some(posts),
            ThreadsParseResult::Search { posts, .. } => Some(posts),
            _ => None,
        }
    }

    /// Returns the single post if this is a post result
    pub fn into_post(self) -> Option<GhostPost> {
        // TODO: Implement post extraction
        match self {
            ThreadsParseResult::Post(post) => Some(post),
            _ => None,
        }
    }

    /// Returns the user if this is a user result
    pub fn into_user(self) -> Option<GhostUser> {
        // TODO: Implement user extraction
        match self {
            ThreadsParseResult::User(user) => Some(user),
            _ => None,
        }
    }
}

/// Threads-specific error types
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
    /// Creates a rate limit error
    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        // TODO: Implement rate limit error construction
        Self::RateLimited { retry_after }
    }

    /// Returns whether this error is retryable
    pub fn is_retryable(&self) -> bool {
        // TODO: Implement retryability check
        matches!(self, ThreadsError::RateLimited { .. })
    }
}

/// Threads-specific user metadata
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct BioLink {
    /// Link URL
    pub url: String,
    /// Link text
    pub text: Option<String>,
}

/// Threads-specific post metadata
#[derive(Debug, Clone)]
pub struct ThreadsPostMetadata {
    /// Post type
    pub post_type: PostType,
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
            post_type: PostType::Text,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostType {
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

/// User mention in a post
#[derive(Debug, Clone)]
pub struct UserMention {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Start position in text
    pub start: usize,
    /// End position in text
    pub end: usize,
}

/// Link entity in a post
#[derive(Debug, Clone)]
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

/// Threads API authentication tokens
#[derive(Debug, Clone)]
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
    pub fn from_cookies(cookies: &str) -> Self {
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
