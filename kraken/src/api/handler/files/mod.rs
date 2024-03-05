//! The handlers and schemas for files uploaded to a workspace

#[cfg(feature = "bin")]
pub(crate) mod handler;
#[cfg(feature = "bin")]
pub(crate) mod handler_admin;
/// The schemas for files uploaded to a workspace
pub mod schema;
#[cfg(feature = "bin")]
pub(crate) mod utils;
