//! All API related code lives here

#[cfg(feature = "bin")]
pub mod extractors;
pub mod handler;
#[cfg(feature = "bin")]
pub mod middleware;
#[cfg(feature = "bin")]
pub mod server;
#[cfg(feature = "bin")]
pub mod swagger;
