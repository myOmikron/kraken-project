//! User management is defined here

#[cfg(feature = "bin")]
pub(crate) mod handler;
#[cfg(feature = "bin")]
pub(crate) mod handler_admin;

/// Schemas for user management are defined here
pub mod schema;
