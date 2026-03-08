//! Ghost Bridge - Polyglot FFI Integration
//!
//! This crate provides FFI bridges for integrating Python, Node.js, and Go
//! scrapers with the Rust core.

mod bridge;
mod worker;
mod protocol;
mod error;

#[cfg(feature = "pyo3")]
mod python;

#[cfg(feature = "napi")]
mod nodejs;

pub use bridge::*;
pub use worker::*;
pub use protocol::*;
pub use error::*;

// Re-export GhostWorker trait
pub use ghost_core::GhostWorker;
