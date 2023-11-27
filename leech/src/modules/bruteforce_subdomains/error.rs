//! The errors of a bruteforce of a subdomain enumeration

use std::io;

use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use crate::modules::bruteforce_subdomains::BruteforceSubdomainResult;

/// The errors that can be thrown when brute-forcing subdomains
#[derive(Debug, Error)]
pub enum BruteforceSubdomainError {
    /// Error while reading the wordlist
    #[error("Could not read wordlist: {0}")]
    WordlistRead(#[from] io::Error),

    /// Error while sending a result
    #[error("The result channel has been closed")]
    ChannelClosed(#[from] SendError<BruteforceSubdomainResult>),
}
