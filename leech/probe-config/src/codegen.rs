use std::{fmt, io};

use crate::parse::{Payload, Protocol, Service};

pub fn generate_code(writer: &mut impl io::Write, services: &[Service]) -> io::Result<()> {
    writer.write_fmt(format_args!("{}", AllProbes::from(services)))
}

#[derive(Default)]
struct AllProbes<'a> {
    empty_tcp_probes: [Vec<BaseProbe<'a>>; 3],
    payload_tcp_probes: [Vec<PayloadProbe<'a>>; 3],
    empty_tls_probes: [Vec<BaseProbe<'a>>; 3],
    payload_tls_probes: [Vec<TlsProbe<'a>>; 3],
    udp_probes: [Vec<PayloadProbe<'a>>; 3],
}
struct BaseProbe<'a> {
    service: &'a str,
    regex: &'a str,
    sub_regex: Option<&'a [String]>,
}
struct PayloadProbe<'a> {
    base: BaseProbe<'a>,
    payload: &'a Payload,
}
struct TlsProbe<'a> {
    base: BaseProbe<'a>,
    payload: &'a Payload,
    alpn: Option<&'a String>,
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
                    (Protocol::Tcp, Payload::Empty) => empty_tcp_probes.push(BaseProbe {
                        service: &service.name,
                        regex: &probe.regex,
                        sub_regex: probe.sub_regex.as_deref(),
                    }),
                    (Protocol::Tcp, payload) => payload_tcp_probes.push(PayloadProbe {
                        base: BaseProbe {
                            service: &service.name,
                            regex: &probe.regex,
                            sub_regex: probe.sub_regex.as_deref(),
                        },
                        payload,
                    }),
                    (Protocol::Tls, Payload::Empty) => empty_tls_probes.push(BaseProbe {
                        service: &service.name,
                        regex: &probe.regex,
                        sub_regex: probe.sub_regex.as_deref(),
                    }),
                    (Protocol::Tls, payload) => payload_tls_probes.push(TlsProbe {
                        base: BaseProbe {
                            service: &service.name,
                            regex: &probe.regex,
                            sub_regex: probe.sub_regex.as_deref(),
                        },
                        payload,
                        alpn: probe.alpn.as_ref(),
                    }),
                    (Protocol::Udp, payload) => udp_probes.push(PayloadProbe {
                        base: BaseProbe {
                            service: &service.name,
                            regex: &probe.regex,
                            sub_regex: probe.sub_regex.as_deref(),
                        },
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
        const HEADER: &str = r#"
use once_cell::sync::Lazy;
use probe_config::generated::*;
use regex::bytes::Regex;

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

impl<'a> fmt::Display for BaseProbe<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            service,
            regex,
            sub_regex,
        } = self;
        write!(f, "BaseProbe {{ service: \"{service}\", regex: Regex::new(r\"{regex}\").unwrap(), sub_regex: vec![")?;
        for sub in sub_regex.unwrap_or(&[]) {
            write!(f, "Regex::new(r\"{sub}\").unwrap(),")?;
        }
        write!(f, "] }}")
    }
}

impl<'a> fmt::Display for PayloadProbe<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { base, payload } = self;
        write!(f, "PayloadProbe {{ base: {base}, payload: ")?;
        match payload {
            Payload::Empty => write!(f, "&[]")?,
            Payload::String(string) => write!(f, "b\"{string}\"")?,
            Payload::Base64(_) => write!(f, "compile_error!(\"TODO\")")?,
        }
        write!(f, " }}")
    }
}

impl<'a> fmt::Display for TlsProbe<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            base,
            payload,
            alpn,
        } = self;
        write!(f, "TlsProbe {{ base: {base}, payload: ")?;
        match payload {
            Payload::Empty => write!(f, "&[]")?,
            Payload::String(string) => write!(f, "b\"{string}\"")?,
            Payload::Base64(_) => write!(f, "compile_error!(\"TODO\")")?,
        }
        write!(f, ", alpn: ")?;
        match alpn {
            None => write!(f, "None")?,
            Some(alpn) => write!(f, "Some(\"{alpn}\")")?,
        }
        write!(f, " }}")
    }
}
