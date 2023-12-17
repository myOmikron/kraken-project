//! Everything regarding workspace management is located in this module

#[cfg(feature = "bin")]
pub(crate) mod handler;
#[cfg(feature = "bin")]
pub(crate) mod handler_admin;
pub mod schema;
#[cfg(feature = "bin")]
pub(crate) mod utils;
