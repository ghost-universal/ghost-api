//! X (Twitter) Platform Adapter
//!
//! Maps X's 'data-testid' and GraphQL responses to Ghost AST.

mod adapter;
mod parser;
mod selectors;
mod graphql;
mod types;

pub use adapter::XAdapter;
pub use types::*;

use ghost_schema::{GhostError, PayloadBlob};

/// Parse X platform data into unified types
pub fn parse_x_response(blob: &PayloadBlob) -> Result<XParseResult, GhostError> {
    // TODO: Implement X response parsing dispatcher
    let adapter = XAdapter::new();
    adapter.parse(blob)
}
