//! This module provides utilities for generating findings in code

#[cfg(feature = "bin")]
pub use factory::FindingFactory;

#[cfg(feature = "bin")]
mod factory;
pub mod schema;
