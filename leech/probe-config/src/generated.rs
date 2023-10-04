//! Types used by the generated code

use std::ops::Deref;

use log::debug;
use regex::bytes::Regex;

pub struct AllProbes {
    pub empty_tcp_probes: [Vec<BaseProbe>; 3],
    pub payload_tcp_probes: [Vec<PayloadProbe>; 3],
    pub empty_tls_probes: [Vec<BaseProbe>; 3],
    pub payload_tls_probes: [Vec<TlsProbe>; 3],
    pub udp_probes: [Vec<PayloadProbe>; 3],
}

/// Base data shared by all probes
pub struct BaseProbe {
    /// The name of the service detected by this probe
    pub service: &'static str,

    /// The base regex this probe is tested against
    pub regex: Regex,

    /// Secondary regexes to match against (if any)
    pub sub_regex: Vec<Regex>,
}

/// A probe with payload
pub struct PayloadProbe {
    /// Base data shared by all probes
    pub base: BaseProbe,

    /// The payload to send upon connection
    pub payload: &'static [u8],
}

/// A probe with payload
pub struct TlsProbe {
    /// Base data shared by all probes
    pub base: BaseProbe,

    /// The payload to send upon connection
    pub payload: &'static [u8],

    /// The protocol to request during handshake via [ALPN (RFC 7301)](https://datatracker.ietf.org/doc/html/rfc7301)
    pub alpn: Option<&'static str>,
}

/// Extended `bool` returned by [`BaseProbe::is_match`] to state if and how much the probe matched the input
pub enum Match {
    No,
    Partial,
    Exact,
}

impl BaseProbe {
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
    type Target = BaseProbe;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Deref for TlsProbe {
    type Target = BaseProbe;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
