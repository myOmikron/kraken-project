//! Types used by the generated code

use std::ops::Deref;

use futures::future::BoxFuture;
use log::debug;
use regex::bytes::Regex;
use tokio::net::UdpSocket;

use crate::modules::service_detection::tcp::OneShotTcpSettings;

include!(concat!(env!("OUT_DIR"), "/generated_probes.rs"));

pub struct AllProbes {
    pub empty_tcp_probes: [Vec<RegexProbe>; 3],
    pub payload_tcp_probes: [Vec<PayloadProbe>; 3],
    pub rust_tcp_probes: [Vec<RustProbe<TcpFn>>; 3],
    pub empty_tls_probes: [Vec<RegexProbe>; 3],
    pub payload_tls_probes: [Vec<TlsProbe>; 3],
    pub rust_tls_probes: [Vec<RustProbe<TlsFn>>; 3],
    pub udp_probes: [Vec<PayloadProbe>; 3],
    pub rust_udp_probes: [Vec<RustProbe<UdpFn>>; 3],
}

pub type TcpFn = for<'a> fn(&'a OneShotTcpSettings) -> BoxFuture<'a, Match>;
pub type TlsFn = for<'a> fn(&'a OneShotTcpSettings, Option<&'static str>) -> BoxFuture<'a, Match>;
pub type UdpFn = for<'a> fn(&'a mut UdpSocket) -> BoxFuture<'a, Match>;

/// A probe implemented in rust
pub struct RustProbe<F> {
    /// The name of the service detected by this probe
    pub service: &'static str,

    /// The function for running the service
    pub function: F,

    /// The protocol to use during ALPN
    ///
    /// This is always `None` for non-tls probes, but not enforced in type-system
    pub alpn: Option<&'static str>,
}

/// Base data shared by all probes
pub struct RegexProbe {
    /// The name of the service detected by this probe
    pub service: &'static str,

    /// The base regex this probe is tested against
    pub regex: Regex,

    /// Secondary regexes to match against (if any)
    pub sub_regex: Vec<Regex>,
}

/// A regex probe with payload
pub struct PayloadProbe {
    /// Base data shared by all probes
    pub base: RegexProbe,

    /// The payload to send upon connection
    pub payload: &'static [u8],
}

/// A regex probe with payload and tls config
pub struct TlsProbe {
    /// Base data shared by all probes
    pub base: RegexProbe,

    /// The payload to send upon connection
    pub payload: &'static [u8],

    /// The protocol to request during handshake via [ALPN (RFC 7301)](https://datatracker.ietf.org/doc/html/rfc7301)
    pub alpn: Option<&'static str>,
}

/// Extended `bool` returned by [`RegexProbe::is_match`] to state if and how much the probe matched the input
pub enum Match {
    No,
    Partial,
    Exact,
}

impl RegexProbe {
    /// Match the probe's regex against the received data
    pub fn is_match(&self, data: &[u8]) -> Match {
        if self.regex.is_match(data) {
            if self.sub_regex.is_empty() {
                Match::Exact
            } else {
                debug!(target: "regex", "Initial regex matched for service: {}", self.service);
                for sub in &self.sub_regex {
                    if sub.is_match(data) {
                        return Match::Exact;
                    }
                }
                Match::Partial
            }
        } else {
            Match::No
        }
    }
}

impl Deref for PayloadProbe {
    type Target = RegexProbe;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Deref for TlsProbe {
    type Target = RegexProbe;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
