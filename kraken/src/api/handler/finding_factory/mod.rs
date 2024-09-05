//! The handler and schemas for inspecting and updating the finding factory's entries

/// The schemas for inspecting and updating the finding factory's entries
pub mod schema;

/// The handler for inspecting and updating the finding factory's entries
#[cfg(feature = "bin")]
pub(crate) mod handler_admin;
