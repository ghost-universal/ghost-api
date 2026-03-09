//! X (Twitter) Platform Adapter
//!
//! Maps X's 'data-testid' and GraphQL responses to Ghost AST.
//!
//! All types are imported from `ghost-schema` - the single source of truth.

mod adapter;
mod parser;
mod selectors;
mod graphql;

pub use adapter::XAdapter;

// Re-export types from local module
pub use types::*;

// Re-export adapter types from ghost-schema
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
    // TODO: Implement X response parsing dispatcher
    let adapter = XAdapter::new();
    adapter.parse(blob)
}
