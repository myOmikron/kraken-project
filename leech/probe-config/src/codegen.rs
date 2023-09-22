use std::{fmt, io};

use crate::{Payload, Protocol, Service};

pub fn generate_code(writer: &mut impl io::Write, services: &[Service]) -> io::Result<()> {
    writer.write_fmt(format_args!("{}", AllProbes::from(services)))
}

#[derive(Default)]
struct AllProbes<'a> {
    empty_tcp_probes: [Vec<EmptyProbe<'a>>; 3],
    payload_tcp_probes: [Vec<PayloadProbe<'a>>; 3],
    empty_tls_probes: [Vec<EmptyProbe<'a>>; 3],
    payload_tls_probes: [Vec<PayloadProbe<'a>>; 3],
    udp_probes: [Vec<PayloadProbe<'a>>; 3],
}
struct EmptyProbe<'a> {
    service: &'a str,
    regex: &'a str,
    sub_regex: Option<&'a [String]>,
}
struct PayloadProbe<'a> {
    service: &'a str,
    regex: &'a str,
    sub_regex: Option<&'a [String]>,
    payload: &'a Payload,
}

impl<'a> Extend<&'a Service> for AllProbes<'a> {
    fn extend<T: IntoIterator<Item = &'a Service>>(&mut self, iter: T) {
        for service in iter {
            let empty_tcp_probes = &mut self.empty_tcp_probes[service.prevalence as usize];
            let payload_tcp_probes = &mut self.payload_tcp_probes[service.prevalence as usize];
            let empty_tls_probes = &mut self.empty_tls_probes[service.prevalence as usize];
            let payload_tls_probes = &mut self.payload_tls_probes[service.prevalence as usize];
            let udp_probes = &mut self.udp_probes[service.prevalence as usize];

            for probe in &service.probes {
                match (&probe.protocol, &probe.payload) {
                    (Protocol::Tcp, Payload::Empty) => empty_tcp_probes.push(EmptyProbe {
                        service: &service.name,
                        regex: &probe.regex,
                        sub_regex: probe.sub_regex.as_deref(),
                    }),
                    (Protocol::Tcp, payload) => payload_tcp_probes.push(PayloadProbe {
                        service: &service.name,
                        regex: &probe.regex,
                        sub_regex: probe.sub_regex.as_deref(),
                        payload,
                    }),
                    (Protocol::Tls, Payload::Empty) => empty_tls_probes.push(EmptyProbe {
                        service: &service.name,
                        regex: &probe.regex,
                        sub_regex: probe.sub_regex.as_deref(),
                    }),
                    (Protocol::Tls, payload) => payload_tls_probes.push(PayloadProbe {
                        service: &service.name,
                        regex: &probe.regex,
                        sub_regex: probe.sub_regex.as_deref(),
                        payload,
                    }),
                    (Protocol::Udp, payload) => udp_probes.push(PayloadProbe {
                        service: &service.name,
                        regex: &probe.regex,
                        sub_regex: probe.sub_regex.as_deref(),
                        payload,
                    }),
                }
            }
        }
    }
}
impl<'a> From<&'a [Service]> for AllProbes<'a> {
    fn from(services: &'a [Service]) -> Self {
        let mut probes = Self::default();
        probes.extend(services);
        probes
    }
}

impl<'a> fmt::Display for AllProbes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const HEADER: &'static str = r#"
use once_cell::sync::Lazy;
use regex::bytes::Regex;
use log::debug;

pub struct AllProbes {
    pub empty_tcp_probes: [Vec<EmptyProbe>; 3],
    pub payload_tcp_probes: [Vec<PayloadProbe>; 3],
    pub empty_tls_probes: [Vec<EmptyProbe>; 3],
    pub payload_tls_probes: [Vec<PayloadProbe>; 3],
    pub udp_probes: [Vec<PayloadProbe>; 3],
}

/// A probe without payload
pub struct EmptyProbe {
    /// The name of the service detected by this probe
    pub service: &'static str,
    
    /// The base regex this probe is tested against
    pub regex: Regex,
    
    /// Secondary regexes to match against (if any)
    pub sub_regex: Vec<Regex>,
}

impl EmptyProbe {
    pub fn is_match(&self, data: &[u8]) -> bool {
        if self.regex.is_match(data) {
            if self.sub_regex.is_empty() {
                true
            } else {
                debug!(target: "regex", "Initial regex matched for service: {}", self.service);
                for sub in &self.sub_regex {
                    if sub.is_match(data) {
                        return true;
                    }
                }
                false
            }
        } else {
            false
        }
    }
}

/// A probe with payload
pub struct PayloadProbe {
    /// The name of the service detected by this probe
    pub service: &'static str,
    
    /// The payload to send upon connection
    pub payload: &'static [u8],
    
    /// The base regex this probe is tested against
    pub regex: Regex,
    
    /// Secondary regexes to match against (if any)
    pub sub_regex: Vec<Regex>,
}

impl PayloadProbe {
    pub fn is_match(&self, data: &[u8]) -> bool {
        if self.regex.is_match(data) {
            if self.sub_regex.is_empty() {
                true
            } else {
                debug!(target: "regex", "Initial regex matched for service: {}", self.service);
                for sub in &self.sub_regex {
                    if sub.is_match(data) {
                        return true;
                    }
                }
                false
            }
        } else {
            false
        }
    }
}

/// Lists of all probes
pub static PROBES: Lazy<AllProbes> = Lazy::new(|| AllProbes {"#;
        writeln!(f, "{HEADER}")?;

        writeln!(f, "    empty_tcp_probes: [")?;
        for group in self.empty_tcp_probes.iter().map(ProbeGroup) {
            writeln!(f, "        {group},")?;
        }
        writeln!(f, "    ],")?;

        writeln!(f, "    payload_tcp_probes: [")?;
        for group in self.payload_tcp_probes.iter().map(ProbeGroup) {
            writeln!(f, "        {group},")?;
        }
        writeln!(f, "    ],")?;

        writeln!(f, "    empty_tls_probes: [")?;
        for group in self.empty_tls_probes.iter().map(ProbeGroup) {
            writeln!(f, "        {group},")?;
        }
        writeln!(f, "    ],")?;

        writeln!(f, "    payload_tls_probes: [")?;
        for group in self.payload_tls_probes.iter().map(ProbeGroup) {
            writeln!(f, "        {group},")?;
        }
        writeln!(f, "    ],")?;

        writeln!(f, "    udp_probes: [")?;
        for group in self.udp_probes.iter().map(ProbeGroup) {
            writeln!(f, "        {group},")?;
        }
        writeln!(f, "    ],")?;

        writeln!(f, "}});")
    }
}

struct ProbeGroup<'a, T>(pub &'a Vec<T>);
impl<'a, T: fmt::Display> fmt::Display for ProbeGroup<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vec![")?;
        for probe in self.0 {
            write!(f, "{probe},")?;
        }
        write!(f, "]")
    }
}

impl<'a> fmt::Display for EmptyProbe<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            service,
            regex,
            sub_regex,
        } = self;
        write!(f, "EmptyProbe {{ service: \"{service}\", regex: Regex::new(r\"{regex}\").unwrap(), sub_regex: vec![")?;
        for sub in sub_regex.unwrap_or(&[]) {
            write!(f, "Regex::new(r\"{sub}\").unwrap(),")?;
        }
        write!(f, "] }}")
    }
}

impl<'a> fmt::Display for PayloadProbe<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            service,
            regex,
            sub_regex,
            payload,
        } = self;
        write!(f, "PayloadProbe {{ service: \"{service}\", regex: Regex::new(r\"{regex}\").unwrap(), sub_regex: vec![")?;
        for sub in sub_regex.unwrap_or(&[]) {
            write!(f, "Regex::new(r\"{sub}\").unwrap(),")?;
        }
        write!(f, "], payload: ")?;
        match payload {
            Payload::Empty => write!(f, "&[]")?,
            Payload::String(string) => write!(f, "b\"{string}\"")?,
            Payload::Base64(_) => write!(f, "compile_error!(\"TODO\")")?,
        }
        write!(f, " }}")
    }
}
