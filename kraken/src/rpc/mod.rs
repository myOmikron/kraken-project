//! This module holds all rpc related definitions.
//!
//! In the default configuration, kraken has a rpc server and a client per leech running.
//! The leech has a server running.
//! If you want to use the leech as cli utility and push results to kraken, it will start a
//! rpc client and connect to the rpc server of kraken

pub use definitions::*;

mod definitions;
pub mod server;
