//! This is the library that can be used to retrieve the models that are used within kraken

pub use chan::WsMessage;

#[allow(dead_code)]
pub mod api;
#[allow(dead_code)]
pub(crate) mod chan;
#[allow(dead_code)]
pub mod config;
#[allow(dead_code)]
pub mod models;
#[allow(dead_code)]
pub mod modules;
#[allow(dead_code)]
pub(crate) mod rpc;
