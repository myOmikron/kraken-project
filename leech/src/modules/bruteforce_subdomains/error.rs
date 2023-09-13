//! The errors of a bruteforce of a subdomain enumeration

use std::fmt::{Display, Formatter};
use std::io;

/// The errors that can be thrown when brute-forcing subdomains
#[derive(Debug)]
pub enum BruteforceSubdomainError {
    /// Error while reading the wordlist
    WordlistRead(io::Error),
}

impl Display for BruteforceSubdomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BruteforceSubdomainError::WordlistRead(err) => {
                write!(f, "Could not read wordlist: {err}")
            }
        }
    }
}
