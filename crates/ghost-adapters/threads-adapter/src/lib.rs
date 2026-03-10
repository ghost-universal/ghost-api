//! Threads (Meta) Platform Adapter
//!
//! Maps Threads' internal relay-style JSON to Ghost AST.
//!
//! All types are imported from `ghost-schema` - the single source of truth.

mod adapter;
mod parser;
mod relay;
mod scraper_parser;

pub use adapter::ThreadsAdapter;
pub use parser::{PostParser, UserParser};
pub use relay::{RelayResponse, RelayError, ThreadsQueries, ThreadsHeaders, ThreadsRequestBuilder};

// Re-export adapter types from ghost-schema
pub use ghost_schema::{
    GhostPost, GhostUser, GhostMedia, MediaType,
    Platform, GhostError, PayloadBlob,
    // Threads-specific types
    ThreadsError, ThreadsUserMetadata, ThreadsPostMetadata, ThreadsPostType, ThreadsAuth, BioLink,
    ThreadsMediaContainer, ThreadsUserResponse, ThreadsInsightsResponse,
    ThreadsListResponse, ThreadsPagination, ThreadsErrorResponse,
    ThreadsMention, LinkEntity, ReplyAudience, HideStatus,
    // Common adapter types
    AdapterParseResult, AdapterError, TrendingTopic,
};

// Re-export scraper parser
pub use scraper_parser::{
    ScraperParser, ScraperPost, ScraperOutput, WorkerResponse,
    parse_scraper_output, parse_scraper_blob, parse_worker_json,
};

/// Parse Threads platform data into unified types
pub fn parse_threads_response(blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
    let adapter = ThreadsAdapter::new();
    adapter.parse(blob)
}

/// Parse threads-scraper output into unified types
pub fn parse_scraper_response(blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
    parse_scraper_blob(blob)
}

/// Parse Threads post from JSON value
pub fn parse_threads_post(data: &serde_json::Value) -> Result<GhostPost, GhostError> {
    let adapter = ThreadsAdapter::new();
    adapter.parse_post(data)
}

/// Parse Threads user from JSON value
pub fn parse_threads_user(data: &serde_json::Value) -> Result<GhostUser, GhostError> {
    let adapter = ThreadsAdapter::new();
    adapter.parse_user(data)
}

/// Parse Threads timeline from JSON value
pub fn parse_threads_timeline(data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
    let adapter = ThreadsAdapter::new();
    adapter.parse_timeline(data)
}

/// Parse Threads search results from JSON value
pub fn parse_threads_search(data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
    let adapter = ThreadsAdapter::new();
    adapter.parse_search(data)
}

// Local types module
mod types {
    //! Threads-specific types not in ghost-schema
    
    use ghost_schema::{GhostPost, GhostUser, Platform};

    /// Threads parse result type
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
            pagination: Option<ghost_schema::ThreadsPagination>,
        },
        /// Timeline
        Timeline {
            posts: Vec<GhostPost>,
            pagination: Option<ghost_schema::ThreadsPagination>,
        },
        /// Error
        Error(ghost_schema::ThreadsError),
    }

    impl ThreadsParseResult {
        /// Get posts if present
        pub fn into_posts(self) -> Option<Vec<GhostPost>> {
            match self {
                ThreadsParseResult::Posts(posts) => Some(posts),
                ThreadsParseResult::Thread { posts, .. } => Some(posts),
                ThreadsParseResult::Timeline { posts, .. } => Some(posts),
                _ => None,
            }
        }

        /// Get single post if present
        pub fn into_post(self) -> Option<GhostPost> {
            match self {
                ThreadsParseResult::Post(post) => Some(post),
                _ => None,
            }
        }

        /// Get user if present
        pub fn into_user(self) -> Option<GhostUser> {
            match self {
                ThreadsParseResult::User(user) => Some(user),
                _ => None,
            }
        }

        /// Check if error
        pub fn is_error(&self) -> bool {
            matches!(self, ThreadsParseResult::Error(_))
        }
    }

    /// Threads insights data
    #[derive(Debug, Clone, Default)]
    pub struct ThreadsInsightsData {
        /// Total views
        pub views: Option<u64>,
        /// Total likes
        pub likes: Option<u64>,
        /// Total replies
        pub replies: Option<u64>,
        /// Total reposts
        pub reposts: Option<u64>,
        /// Total quotes
        pub quotes: Option<u64>,
        /// Total engagement
        pub engagement: Option<u64>,
        /// Follower count at post time
        pub follower_count: Option<u64>,
        /// Engagement rate
        pub engagement_rate: Option<f64>,
    }

    impl ThreadsInsightsData {
        /// Calculate engagement rate
        pub fn calculate_engagement_rate(&mut self) {
            if let (Some(engagement), Some(views)) = (self.engagement, self.views) {
                if views > 0 {
                    self.engagement_rate = Some((engagement as f64 / views as f64) * 100.0);
                }
            }
        }
    }
}
