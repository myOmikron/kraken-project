//! Handlers and schemas for object affected by findings

/// Handlers for object affected by findings
#[cfg(feature = "bin")]
pub(crate) mod handler;

/// Schemas for object affected by findings
pub mod schema;
#[cfg(feature = "bin")]
pub(crate) mod utils;
