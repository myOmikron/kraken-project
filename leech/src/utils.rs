//! Helper utilities

use std::num::NonZeroU16;
use std::ops::RangeInclusive;
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::{bytes, Regex};
use thiserror::Error;
use tokio::io::{self, stdin, AsyncBufReadExt, BufReader};
use tonic::transport::{Certificate, ClientTlsConfig, Endpoint};

use crate::config::KrakenConfig;

pub(crate) struct Regexes {
    pub(crate) ports: Regex,
    pub(crate) spf_domain_spec: bytes::Regex,
}

pub(crate) static RE: Lazy<Regexes> = Lazy::new(|| Regexes {
    ports: Regex::new(r"^(?P<range>\d*-\d*)$|^(?P<single>\d+)$|^$").unwrap(),
    spf_domain_spec: bytes::Regex::new(
        r"[\x21-\x7e]*(?:\.(?:\w*[^\W\d]\w*|\w+-[\w-]*\w)\.?|%[\x21-\x7e]+)",
    )
    .unwrap(),
});

/// Error while parsing ports
#[derive(Debug, Error)]
pub enum ParsePortError {
    /// Invalid port parsed
    #[error("{0}")]
    InvalidPort(String),

    /// Invalid port range parsed
    #[error("{0}")]
    InvalidPortRange(String),
}

/// Parse ports retrieved via clap
pub fn parse_ports(
    ports: &[String],
    parsed_ports: &mut Vec<RangeInclusive<u16>>,
) -> Result<(), ParsePortError> {
    for port in ports {
        let port_parts = port.split(',');
        for part in port_parts {
            if let Some(captures) = RE.ports.captures(part) {
                if let Some(c) = captures.get(0) {
                    if c.as_str().is_empty() {
                        continue;
                    }
                }
                if let Some(m) = captures.name("range") {
                    let mut start = 1;
                    let mut end = u16::MAX;
                    for (idx, content) in m.as_str().split('-').enumerate() {
                        match idx {
                            0 => {
                                if content.is_empty() {
                                    start = 1;
                                } else if let Ok(v) = NonZeroU16::from_str(content) {
                                    start = u16::from(v);
                                } else {
                                    return Err(ParsePortError::InvalidPort(content.to_string()));
                                }
                            }
                            1 => {
                                if content.is_empty() {
                                    end = u16::MAX;
                                } else if let Ok(v) = NonZeroU16::from_str(content) {
                                    end = u16::from(v);
                                } else {
                                    return Err(ParsePortError::InvalidPort(content.to_string()));
                                }
                            }
                            _ => unreachable!(""),
                        }
                    }

                    if end < start {
                        return Err(ParsePortError::InvalidPortRange(format!(
                            "Invalid port range: {end} < {start}"
                        )));
                    }

                    parsed_ports.push(start..=end);
                } else if let Some(m) = captures.name("single") {
                    if let Ok(v) = NonZeroU16::from_str(m.as_str()) {
                        let port = u16::from(v);
                        parsed_ports.push(port..=port);
                    } else {
                        return Err(ParsePortError::InvalidPort(m.as_str().to_string()));
                    }
                }
            } else {
                return Err(ParsePortError::InvalidPort(format!(
                    "Invalid port declaration found: {part}"
                )));
            }
        }
    }

    Ok(())
}

/// Read a line from stdin
pub async fn input() -> io::Result<Option<String>> {
    BufReader::new(stdin()).lines().next_line().await
}

/// Build an endpoint for connecting to kraken.
pub fn kraken_endpoint(config: &KrakenConfig) -> Result<Endpoint, tonic::transport::Error> {
    Endpoint::from_str(config.kraken_uri.as_str())?.tls_config(
        ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(&config.kraken_ca))
            .domain_name(&config.kraken_sni),
    )
}
