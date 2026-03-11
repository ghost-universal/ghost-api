//! X (Twitter) Platform Adapter
//!
//! Maps X's 'data-testid' and GraphQL responses to Ghost AST.
//!
//! All types are imported from `ghost-schema` - the single source of truth.

#![allow(clippy::large_enum_variant)]

mod adapter;
mod parser;
mod selectors;
mod graphql;

pub use adapter::XAdapter;
pub use parser::{PostParser, UserParser};
pub use selectors::XSelectors;
pub use graphql::{GraphQLQueries, GraphQLFeatures};

// Re-export types from ghost-schema
pub use ghost_schema::{
    GhostPost, GhostUser, GhostMedia, MediaType,
    Platform, GhostError, PayloadBlob,
    // X-specific types
    XError, XUserMetadata, XPostMetadata,
    XTweetResponse, XTweetData, XUserData, XMediaData,
    XTweetMetrics, XUserMetrics, XIncludes,
    // Common adapter types
    AdapterParseResult, AdapterError, TrendingTopic,
    Coordinates, Place, UserMention, UrlEntity, HashtagEntity, CashtagEntity,
};

/// Parse X platform data into unified types
pub fn parse_x_response(blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
    let adapter = XAdapter::new();
    adapter.parse(blob)
}

/// Parse X post from JSON value
pub fn parse_x_post(data: &serde_json::Value) -> Result<GhostPost, GhostError> {
    let adapter = XAdapter::new();
    adapter.parse_post(data)
}

/// Parse X user from JSON value  
pub fn parse_x_user(data: &serde_json::Value) -> Result<GhostUser, GhostError> {
    let adapter = XAdapter::new();
    adapter.parse_user(data)
}

/// Parse X timeline from JSON value
pub fn parse_x_timeline(data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
    let adapter = XAdapter::new();
    adapter.parse_timeline(data)
}

/// Parse X search results from JSON value
pub fn parse_x_search(data: &serde_json::Value) -> Result<Vec<GhostPost>, GhostError> {
    let adapter = XAdapter::new();
    adapter.parse_search(data)
}

/// Parse X trending topics from JSON value
pub fn parse_x_trending(data: &serde_json::Value) -> Result<Vec<TrendingTopic>, GhostError> {
    let adapter = XAdapter::new();
    adapter.parse_trending(data)
}

// Local types module
#[allow(dead_code)]
mod types {
    //! X-specific types not in ghost-schema
    
    use ghost_schema::{GhostPost, GhostUser};

    /// X parse result type
    #[derive(Debug, Clone)]
    pub enum XParseResult {
        /// Single user profile
        User(GhostUser),
        /// Single post/tweet
        Post(Box<GhostPost>),
        /// Multiple posts
        Posts(Vec<GhostPost>),
        /// Timeline with bidirectional pagination
        Timeline {
            posts: Vec<GhostPost>,
            cursor_top: Option<String>,
            cursor_bottom: Option<String>,
        },
        /// Trending topics
        Trending(Vec<ghost_schema::TrendingTopic>),
        /// Error
        Error(ghost_schema::XError),
    }

    impl XParseResult {
        /// Get posts if present
        pub fn into_posts(self) -> Option<Vec<GhostPost>> {
            match self {
                XParseResult::Posts(posts) => Some(posts),
                XParseResult::Timeline { posts, .. } => Some(posts),
                _ => None,
            }
        }

        /// Get single post if present
        pub fn into_post(self) -> Option<GhostPost> {
            match self {
                XParseResult::Post(post) => Some(*post),
                _ => None,
            }
        }

        /// Get user if present
        pub fn into_user(self) -> Option<GhostUser> {
            match self {
                XParseResult::User(user) => Some(user),
                _ => None,
            }
        }

        /// Check if error
        pub fn is_error(&self) -> bool {
            matches!(self, XParseResult::Error(_))
        }
    }

    /// X pagination cursor
    #[derive(Debug, Clone)]
    pub struct XPagination {
        /// Next token
        pub next_token: Option<String>,
        /// Previous token
        pub previous_token: Option<String>,
    }
}
