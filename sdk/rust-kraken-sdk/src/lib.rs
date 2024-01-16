//! # kraken-sdk
//!
//! The SDK to [Kraken](https://github.com/myOmikron/kraken-project).

#![warn(clippy::unwrap_used, clippy::expect_used, missing_docs)]
#![forbid(unsafe_code)]

pub use crate::export::*;
pub use crate::sdk::*;

pub mod error;
mod export;
mod sdk;
