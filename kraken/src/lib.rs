//! This is the library that can be used to retrieve the models that are used within kraken

pub mod api;
pub mod chan;
#[cfg(feature = "bin")]
pub mod config;
pub mod models;

#[cfg(feature = "bin")]
pub mod modules;
#[cfg(feature = "bin")]
pub mod rpc;
