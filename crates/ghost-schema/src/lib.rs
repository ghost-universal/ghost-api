//! Ghost Schema - Unified Social AST (Abstract Syntax Tree)
//!
//! This crate provides the unified data structures for representing social media
//! content across different platforms (X/Twitter, Threads, etc.).

mod types;
mod error;
mod capability;
mod platform;

pub use types::*;
pub use error::*;
pub use capability::*;
pub use platform::*;

// Re-export commonly used types
pub mod prelude {
    pub use crate::{GhostPost, GhostUser, GhostMedia, GhostContext, Platform, Capability};
}
