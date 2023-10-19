//! The errors of a bruteforce of a subdomain enumeration

use std::io;

use thiserror::Error;

/// The errors that can be thrown when brute-forcing subdomains
#[derive(Debug, Error)]
pub enum BruteforceSubdomainError {
    /// Error while reading the wordlist
    #[error("Could not read wordlist: {0}")]
    WordlistRead(#[from] io::Error),
}
