//! Handlers and schemas for finding categories

/// Handlers for finding categories
#[cfg(feature = "bin")]
pub(crate) mod handler;

/// Admin handlers for finding categories
#[cfg(feature = "bin")]
pub(crate) mod handler_admin;

/// Schemas for finding categories
pub mod schema;
#[cfg(feature = "bin")]
mod utils;
