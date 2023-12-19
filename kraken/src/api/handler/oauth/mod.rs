//! OAuth related code lives here

#[cfg(feature = "bin")]
pub(crate) mod handler;

/// OAuth related schemas lives here
pub mod schema;

#[cfg(feature = "bin")]
pub(crate) mod utils;
