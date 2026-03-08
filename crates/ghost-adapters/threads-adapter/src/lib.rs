//! Threads (Meta) Platform Adapter
//!
//! Maps Threads' internal relay-style JSON to Ghost AST.
//!
//! All types are imported from `ghost-schema` - the single source of truth.

mod adapter;
mod parser;
mod relay;

pub use adapter::ThreadsAdapter;

// Re-export adapter types from ghost-schema
pub use ghost_schema::{
    GhostPost, GhostUser, GhostMedia, MediaType,
    Platform, GhostError, PayloadBlob,
    // Threads-specific types
    ThreadsError, ThreadsUserMetadata, ThreadsPostMetadata, ThreadsPostType, ThreadsAuth, BioLink,
    // Common adapter types
    AdapterParseResult, AdapterError, TrendingTopic,
    UserMention, LinkEntity,
};

/// Parse Threads platform data into unified types
pub fn parse_threads_response(blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
    // TODO: Implement Threads response parsing dispatcher
    let adapter = ThreadsAdapter::new();
    adapter.parse(blob)
}
