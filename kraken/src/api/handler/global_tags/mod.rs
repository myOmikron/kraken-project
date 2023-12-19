//! The handlers and schemas for global tags are defined in this module

#[cfg(feature = "bin")]
pub(crate) mod handler;

#[cfg(feature = "bin")]
pub(crate) mod handler_admin;

/// The schemas for global tags are defined in this module
pub mod schema;
