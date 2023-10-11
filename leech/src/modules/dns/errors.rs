//! Holds all the errors from dns resolution

use std::fmt::{Display, Formatter};

use trust_dns_resolver::error::ResolveError;

/// DNS Resolution error types
pub enum DnsResolutionError {
    /// Error creating the system resolver
    CreateSystemResolver(ResolveError),
}

impl Display for DnsResolutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsResolutionError::CreateSystemResolver(err) => {
                write!(f, "Could not create system resolver: {err}")
            }
        }
    }
}
