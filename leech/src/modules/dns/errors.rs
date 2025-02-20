//! Holds all the errors from dns resolution

use std::fmt::Display;
use std::fmt::Formatter;

use hickory_resolver::error::ResolveError;
use itertools::Itertools;
use thiserror::Error;

/// DNS Resolution error types
#[derive(Debug, Error)]
pub enum DnsResolutionError {
    /// Error creating the system resolver
    #[error("Could not create system resolver: {0}")]
    CreateSystemResolver(#[from] ResolveError),
    /// Failed at least parts of the input
    #[error("Failed resolving the following domains:\n{}", format_list(.0))]
    SomeFailed(Vec<ResolutionStatus>),
}

/// Resolution status for a single domain in a DNS resolution attack.
#[derive(Debug)]
pub struct ResolutionStatus {
    /// The input domain
    pub domain: String,
    /// A/AAAA/CNAME lookup status
    pub ip: LookupResultStatus,
    /// MX lookup status
    pub mx: LookupResultStatus,
    /// TLSA lookup status
    pub tlsa: LookupResultStatus,
    /// TXT lookup status
    pub txt: LookupResultStatus,
    /// CAA lookup status
    pub caa: LookupResultStatus,
}

impl ResolutionStatus {
    /// Returns true if any field is error
    pub fn has_error(&self) -> bool {
        self.ip.is_error()
            || self.mx.is_error()
            || self.tlsa.is_error()
            || self.txt.is_error()
            || self.caa.is_error()
    }

    /// Returns true if any field is Success
    pub fn has_records(&self) -> bool {
        self.ip.is_success()
            || self.mx.is_success()
            || self.tlsa.is_success()
            || self.txt.is_success()
            || self.caa.is_success()
    }
}

/// Custom result for lookups
#[derive(Debug)]
pub enum LookupResultStatus {
    /// No error and got records
    Success,
    /// Got no records
    NoRecords,
    /// Got some I/O error or similar
    Error(ResolveError),
}

impl Display for LookupResultStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LookupResultStatus::Success => write!(f, "ok"),
            LookupResultStatus::NoRecords => write!(f, "no records"),
            LookupResultStatus::Error(e) => write!(f, "FAIL: {e}"),
        }
    }
}

impl LookupResultStatus {
    fn is_success(&self) -> bool {
        matches!(self, LookupResultStatus::Success)
    }

    fn is_error(&self) -> bool {
        matches!(self, LookupResultStatus::Error(_))
    }
}

fn format_list<T>(v: &[T]) -> String
where
    T: Display,
{
    return v.iter().map(|v| format!("- {}", v)).join("\n");
}

impl Display for ResolutionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ResolutionStatus {
            domain,
            ip,
            mx,
            tlsa,
            txt,
            caa,
        } = self;
        if !self.has_error() {
            if self.has_records() {
                write!(f, "{domain}: ok")
            } else {
                write!(f, "{domain}: no records")
            }
        } else {
            write!(
                f,
                "{domain}: IP {ip}, MX {mx}, TLSA {tlsa}, TXT {txt}, CAA {caa}",
            )
        }
    }
}
