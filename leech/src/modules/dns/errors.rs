//! Holds all the errors from dns resolution

use thiserror::Error;

/// DNS Resolution error types
#[derive(Debug, Error)]
pub enum DnsResolutionError {
    /// Error creating the system resolver
    #[error("Could not create system resolver: {0}")]
    CreateSystemResolver(#[from] trust_dns_resolver::error::ResolveError),
}
