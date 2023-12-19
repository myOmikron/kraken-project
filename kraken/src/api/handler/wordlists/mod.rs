//! Endpoints for listing and managing wordlists

#[cfg(feature = "bin")]
pub(crate) mod handler;

#[cfg(feature = "bin")]
pub(crate) mod handler_admin;

/// Schemas for listing and managing wordlists
pub mod schema;
