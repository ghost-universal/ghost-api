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

// Re-export types from local module
pub use types::*;

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
    // TODO: Implement Threads response parsing dispatcher
    let adapter = ThreadsAdapter::new();
    adapter.parse(blob)
}

/// Parse threads-scraper output into unified types
pub fn parse_scraper_response(blob: &PayloadBlob) -> Result<AdapterParseResult, GhostError> {
    parse_scraper_blob(blob)
}
