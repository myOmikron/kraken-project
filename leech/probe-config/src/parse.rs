use std::path::Path;
use std::str::FromStr;

use base64::prelude::*;

#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub prevalence: Prevalence,
    pub probes: Vec<Probe>,
}

impl Service {
    pub fn from_file(file: &str, content: &str) -> Result<Self, ParseError> {
        parse_file(file, content)
    }
}

#[derive(Debug)]
pub struct Probe {
    pub protocol: Protocol,
    pub alpn: Option<String>,
    pub payload: Payload,
    pub regex: String,
    pub sub_regex: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum Payload {
    Empty,
    String(String),
    Binary(Vec<u8>),
}

#[derive(Debug, Copy, Clone)]
pub enum Protocol {
    Tcp,
    Udp,
    Tls,
}

#[derive(Debug, Copy, Clone)]
pub enum Prevalence {
    Often,
    Average,
    Obscure,
}

/// The directory name a probe file must be in.
#[derive(Debug, PartialEq)]
pub enum ProbeFileDirectory {
    /// Allows TCP and TLS - directory name: `"tcp"`
    Tcp,
    /// Allows UDP only - directory name: `"udp"`
    Udp,
}

#[derive(Debug)]
pub enum ParseError {
    /// The initial `service: <name>` line is missing
    MissingService,
    /// The second line `prevalence:` is missing
    MissingPrevalence,
    /// The third line `probes:` is missing or has no lines following
    MissingProbes,

    /// A probe's value has been passed twice
    DuplicateValue(&'static str, usize),
    /// A probe's value is missing
    MissingValue(&'static str, usize),
    /// An unknown probe
    UnknownValue(usize),

    /// More than one `payload_str`, `payload_b64` or `payload_hex` are specified
    ConflictingPayload { probe_line: usize },
    /// Format errors for `payload_b64` or `payload_hex`
    InvalidPayload { probe_line: usize },

    /// The sub regex must be the last key in any probe
    ValueAfterSubRegex(usize),
    /// A sub regex item before `sub_regex:`
    UnexpectedSubRegex(usize),
    /// The sub regex is specified but empty,
    MissingSubRegex { probe_line: usize },

    /// Invalid value for `protocol: `
    InvalidProtocol(usize),
    /// Invalid value for `prevalence: `
    InvalidPrevalence(usize),
    /// Value for `protocol: ` doesn't match folder name it's in.
    ProtocolMismatch {
        expected: ProbeFileDirectory,
        actual: ProbeFileDirectory,
    },
    /// Probe file not in a valid folder name
    UnimplementedFolder(String),
    /// Some other error occured from trying to parse the path (very unlikely error)
    FilenameError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::MissingService => {
                write!(f, "The file should start with `service: <name>`")
            }
            ParseError::MissingPrevalence => {
                write!(f, "The `service: <name>` line should be followed by `prevalence: <often|average|obscure>`")
            }
            ParseError::MissingProbes => {
                write!(
                    f,
                    "Missing list of probes `probes:\\n  - <probe declaration>"
                )
            }
            ParseError::DuplicateValue(value, line) => {
                write!(f, "The value {value} in line {line} has already been set")
            }
            ParseError::MissingValue(value, probe) => {
                write!(
                    f,
                    "The probe started in line {probe} is missing the value {value}"
                )
            }
            ParseError::UnknownValue(line) => {
                write!(f, "Unknown value in line {line}")
            }
            ParseError::ConflictingPayload { probe_line } => {
                write!(
                    f,
                    "The probe started in line {probe_line} has two conflicting payloads"
                )
            }
            ParseError::InvalidPayload { probe_line } => {
                write!(f, "Invalid payload format in line {probe_line}")
            }
            ParseError::ValueAfterSubRegex(line) => {
                write!(
                    f,
                    "There is a value after a `sub_regex` list in line {line}"
                )
            }
            ParseError::UnexpectedSubRegex(line) => {
                write!(f, "`sub_regex` item outside of list in line {line}")
            }
            ParseError::MissingSubRegex { probe_line } => {
                write!(
                    f,
                    "The probe started in line {probe_line} has an empty `sub_regex` list \
                    (If you don't want any, then remove the list completely)"
                )
            }
            ParseError::InvalidProtocol(line) => {
                write!(f, "Invalid protocol in line {line}")
            }
            ParseError::InvalidPrevalence(line) => {
                write!(f, "Invalid prevalence in line {line}")
            }
            ParseError::FilenameError(filename) => {
                write!(
                    f,
                    "Unable to resolve information from filename `{filename}`."
                )
            }
            ParseError::UnimplementedFolder(folder) => {
                write!(f, "Unrecognized probe folder `{folder}`.")
            }
            ParseError::ProtocolMismatch { actual, expected } => {
                write!(f, "File specified protocol {actual:?} but is expected to be {expected:?}, since it's in that folder.")
            }
        }
    }
}
impl std::error::Error for ParseError {}

fn parse_file(filename: &str, content: &str) -> Result<Service, ParseError> {
    // bunch of unwraps, since this is a build script and errors here would mean misconfiguration of the glob / pattern matcher
    let actual_dir = match Path::new(&filename)
        .parent()
        .ok_or(ParseError::FilenameError(String::from(filename)))?
        .file_name()
        .ok_or(ParseError::FilenameError(String::from(filename)))?
        .to_str()
        .ok_or(ParseError::FilenameError(String::from(filename)))?
    {
        "tcp" => ProbeFileDirectory::Tcp,
        "udp" => ProbeFileDirectory::Udp,
        v => return Err(ParseError::UnimplementedFolder(String::from(v))),
    };

    // Iterator over lines with their numbers excluding empty lines and comment lines
    let mut lines = content
        .lines()
        .enumerate()
        .filter(|(_, line)| !(line.is_empty() || line.trim_start().starts_with('#')))
        .map(|(index, line)| (index + 1, line));

    // First line
    let name = lines
        .next()
        .ok_or(ParseError::MissingService)?
        .1
        .strip_prefix("service: ")
        .ok_or(ParseError::MissingService)?
        .to_string();

    // Second line
    let snd = lines.next().ok_or(ParseError::MissingPrevalence)?;
    let prevalence: Prevalence = snd
        .1
        .strip_prefix("prevalence: ")
        .ok_or(ParseError::MissingPrevalence)?
        .parse()
        .map_err(|_| ParseError::InvalidPrevalence(snd.0))?;

    // Third line
    lines.next().ok_or(ParseError::MissingProbes)?;

    // Start parsing all probes
    let mut probes = Vec::new();

    // First line need special treatment
    let (first_number, first_line) = lines.next().ok_or(ParseError::MissingProbes)?;
    let first_trimmed = first_line
        .strip_prefix("  - ")
        .ok_or(ParseError::MissingProbes)?;
    let mut builder = ProbeBuilder {
        start_line: first_number,
        ..Default::default()
    };
    builder.handle_trimmed_line(first_number, first_trimmed)?;

    for (number, line) in lines {
        if let Some(trimmed) = line.strip_prefix("    ") {
            builder.handle_trimmed_line(number, trimmed)?;
            continue;
        }
        if let Some(trimmed) = line.strip_prefix("  - ") {
            probes.push(builder.finish()?);
            builder = ProbeBuilder {
                start_line: number,
                ..Default::default()
            };

            builder.handle_trimmed_line(number, trimmed)?;
            continue;
        }
    }

    let probe = builder.finish()?;

    let expected_dir = match probe.protocol {
        Protocol::Udp => ProbeFileDirectory::Udp,
        Protocol::Tcp => ProbeFileDirectory::Tcp,
        Protocol::Tls => ProbeFileDirectory::Tcp,
    };

    if actual_dir != expected_dir {
        return Err(ParseError::ProtocolMismatch {
            actual: actual_dir,
            expected: expected_dir,
        });
    }

    probes.push(probe);

    Ok(Service {
        name,
        prevalence,
        probes,
    })
}

fn set_or_err<T>(
    option: &mut Option<T>,
    value: Result<T, ParseError>,
    or_err: ParseError,
) -> Result<(), ParseError> {
    if option.is_some() {
        Err(or_err)
    } else {
        *option = Some(value?);
        Ok(())
    }
}

#[derive(Default)]
struct ProbeBuilder {
    start_line: usize,
    protocol: Option<Protocol>,
    alpn: Option<String>,
    payload_str: Option<String>,
    payload_b64: Option<String>,
    payload_hex: Option<String>,
    regex: Option<String>,
    sub_regex: Option<Vec<String>>,
}
impl ProbeBuilder {
    fn handle_trimmed_line(&mut self, number: usize, line: &str) -> Result<(), ParseError> {
        if self.sub_regex.is_some() && !line.starts_with("  - ") {
            return Err(ParseError::ValueAfterSubRegex(number));
        }

        if let Some(value) = line.strip_prefix("protocol: ") {
            return set_or_err(
                &mut self.protocol,
                Protocol::from_str(value).map_err(|_| ParseError::InvalidProtocol(number)),
                ParseError::DuplicateValue("protocol", number),
            );
        }
        if let Some(value) = line.strip_prefix("alpn: ") {
            return set_or_err(
                &mut self.alpn,
                Ok(value.to_string()),
                ParseError::DuplicateValue("alpn", number),
            );
        }
        if let Some(value) = line.strip_prefix("payload_str: ") {
            return set_or_err(
                &mut self.payload_str,
                Ok(value.to_string()),
                ParseError::DuplicateValue("payload_str", number),
            );
        }
        if let Some(value) = line.strip_prefix("payload_b64: ") {
            return set_or_err(
                &mut self.payload_b64,
                Ok(value.to_string()),
                ParseError::DuplicateValue("payload_b64", number),
            );
        }
        if let Some(value) = line.strip_prefix("payload_hex: ") {
            return set_or_err(
                &mut self.payload_hex,
                Ok(value.to_string()),
                ParseError::DuplicateValue("payload_hex", number),
            );
        }
        if let Some(value) = line.strip_prefix("regex: ") {
            return set_or_err(
                &mut self.regex,
                Ok(value.to_string()),
                ParseError::DuplicateValue("regex", number),
            );
        }
        if line == "sub_regex:" {
            return set_or_err(
                &mut self.sub_regex,
                Ok(Vec::new()),
                ParseError::DuplicateValue("sub_regex", number),
            );
        }
        if let Some(value) = line.strip_prefix("  - ") {
            let sub_regex = self
                .sub_regex
                .as_mut()
                .ok_or(ParseError::UnexpectedSubRegex(number))?;
            sub_regex.push(value.to_string());
            return Ok(());
        }
        Err(ParseError::UnknownValue(number))
    }

    fn finish(self) -> Result<Probe, ParseError> {
        if let Some(sub_regex) = self.sub_regex.as_ref() {
            if sub_regex.is_empty() {
                return Err(ParseError::MissingSubRegex {
                    probe_line: self.start_line,
                });
            }
        }
        let payload = match (self.payload_str, self.payload_b64, self.payload_hex) {
            (None, None, None) => Payload::Empty,
            (Some(string), None, None) => Payload::String(string),
            (None, Some(base64), None) => {
                Payload::Binary(BASE64_STANDARD.decode(base64).map_err(|_| {
                    ParseError::InvalidPayload {
                        probe_line: self.start_line,
                    }
                })?)
            }
            (None, None, Some(hex_string)) => {
                Payload::Binary(hex::decode(hex_string).map_err(|_| {
                    ParseError::InvalidPayload {
                        probe_line: self.start_line,
                    }
                })?)
            }
            (_, _, _) => {
                return Err(ParseError::ConflictingPayload {
                    probe_line: self.start_line,
                })
            }
        };
        Ok(Probe {
            protocol: self
                .protocol
                .ok_or(ParseError::MissingValue("protocol", self.start_line))?,
            alpn: self.alpn,
            payload,
            regex: self
                .regex
                .ok_or(ParseError::MissingValue("regex", self.start_line))?,
            sub_regex: self.sub_regex,
        })
    }
}

impl FromStr for Protocol {
    type Err = ();
    fn from_str(protocol: &str) -> Result<Self, Self::Err> {
        match protocol {
            "TCP" => Ok(Self::Tcp),
            "UDP" => Ok(Self::Udp),
            "TLS" => Ok(Self::Tls),
            _ => Err(()),
        }
    }
}

impl FromStr for Prevalence {
    type Err = ();
    fn from_str(protocol: &str) -> Result<Self, Self::Err> {
        match protocol {
            "often" => Ok(Self::Often),
            "average" => Ok(Self::Average),
            "obscure" => Ok(Self::Obscure),
            _ => Err(()),
        }
    }
}
