use std::str::FromStr;

#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub prevalence: Prevalence,
    pub probes: Vec<Probe>,
}

impl Service {
    pub fn from_file(file: &str) -> Result<Self, ParseError> {
        parse_file(file)
    }
}

#[derive(Debug)]
pub struct Probe {
    pub protocol: Protocol,
    pub payload: Payload,
    pub regex: String,
    pub sub_regex: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum Payload {
    Empty,
    String(String),
    Base64(String),
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

    /// Both `payload_str` and `payload_b64` are specified
    ConflictingPayload { probe_line: usize },

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
}

fn parse_file(file: &str) -> Result<Service, ParseError> {
    // Iterator over lines with their numbers excluding empty lines and comment lines
    let mut lines = file
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
    probes.push(builder.finish()?);

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
    payload_str: Option<String>,
    payload_b64: Option<String>,
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
        let payload = match (self.payload_str, self.payload_b64) {
            (None, None) => Payload::Empty,
            (Some(string), None) => Payload::String(string),
            (None, Some(base64)) => Payload::Base64(base64),
            (Some(_), Some(_)) => {
                return Err(ParseError::ConflictingPayload {
                    probe_line: self.start_line,
                })
            }
        };
        Ok(Probe {
            protocol: self
                .protocol
                .ok_or(ParseError::MissingValue("protocol", self.start_line))?,
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
