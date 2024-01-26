//! The handler core for the knowledge base, internally called [FindingDefinition] is defined
//! in this module and in its submodules

pub mod schema;

#[cfg(feature = "bin")]
pub(crate) mod handler;
