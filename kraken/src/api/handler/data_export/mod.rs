//! This module holds the data export of a workspace
//!
//! Data can be exported by an oauth application that was registered by an admin and has
//! access to a workspace granted by an user.

#[cfg(feature = "bin")]
pub(crate) mod handler;
pub mod schema;
