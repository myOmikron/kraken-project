//! This is the library that can be used to retrieve the models that are used within kraken
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

pub mod api;
pub mod chan;
#[cfg(feature = "bin")]
pub mod config;
#[cfg(feature = "bin")]
pub mod models;

pub mod modules;
#[cfg(feature = "bin")]
pub mod rpc;
