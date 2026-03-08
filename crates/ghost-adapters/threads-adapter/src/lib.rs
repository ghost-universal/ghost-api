//! Threads (Meta) Platform Adapter
//!
//! Maps Threads' internal relay-style JSON to Ghost AST.

mod adapter;
mod parser;
mod relay;
mod types;

pub use adapter::ThreadsAdapter;
pub use types::*;

use ghost_schema::{GhostError, PayloadBlob};

/// Parse Threads platform data into unified types
pub fn parse_threads_response(blob: &PayloadBlob) -> Result<ThreadsParseResult, GhostError> {
    // TODO: Implement Threads response parsing dispatcher
    let adapter = ThreadsAdapter::new();
    adapter.parse(blob)
}
