//! Helper utilities

use std::num::NonZeroU16;
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::Regex;

pub(crate) struct Regexes {
    pub(crate) ports: Regex,
}

static RE: Lazy<Regexes> = Lazy::new(|| Regexes {
    ports: Regex::new(r#"^(?P<range>\d*-\d*)$|^(?P<single>\d+)$|^$"#).unwrap(),
});

/// Error while parsing ports
pub enum ParsePortError {
    /// Invalid port parsed
    InvalidPort(String),
    /// Invalid port range parsed
    InvalidPortRange(String),
}

impl From<ParsePortError> for String {
    fn from(value: ParsePortError) -> String {
        match value {
            ParsePortError::InvalidPort(err) => err,
            ParsePortError::InvalidPortRange(err) => err,
        }
    }
}

/// Parse ports retrieved via clap
pub fn parse_ports(ports: &[String], parsed_ports: &mut Vec<u16>) -> Result<(), ParsePortError> {
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

                    for port in start..=end {
                        parsed_ports.push(port);
                    }
                } else if let Some(m) = captures.name("single") {
                    if let Ok(v) = NonZeroU16::from_str(m.as_str()) {
                        parsed_ports.push(u16::from(v));
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

    parsed_ports.sort();
    parsed_ports.dedup();

    Ok(())
}
