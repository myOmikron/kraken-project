//! Endpoints to inspect and modify registered oauth applications

#[cfg(feature = "bin")]
pub(crate) mod handler_admin;

/// The schemas for the endpoints to inspect and modify registered oauth applications
pub mod schema;
