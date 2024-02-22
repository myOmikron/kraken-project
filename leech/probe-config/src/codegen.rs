use std::{fmt, io};

use crate::schema::ProbeFile;

pub fn generate_code(writer: &mut impl io::Write, services: &[ProbeFile]) -> io::Result<()> {
    writer.write_fmt(format_args!("{}", AllProbes::from(services)))
}

#[derive(Default)]
struct AllProbes<'a> {
    empty_tcp_probes: [Vec<RegexProbe<'a>>; 3],
    payload_tcp_probes: [Vec<PayloadProbe<'a>>; 3],
    rust_tcp_probes: [Vec<RustProbe<'a>>; 3],
    empty_tls_probes: [Vec<RegexProbe<'a>>; 3],
    payload_tls_probes: [Vec<TlsProbe<'a>>; 3],
    rust_tls_probes: [Vec<RustProbe<'a>>; 3],
    udp_probes: [Vec<PayloadProbe<'a>>; 3],
    rust_udp_probes: [Vec<RustProbe<'a>>; 3],
}
struct RegexProbe<'a> {
    service: &'a str,
    regex: &'a str,
    sub_regex: Option<&'a [String]>,
}
struct PayloadProbe<'a> {
    base: RegexProbe<'a>,
    payload: Payload<'a>,
}
struct TlsProbe<'a> {
    base: RegexProbe<'a>,
    payload: Payload<'a>,
    alpn: Option<&'a String>,
}
struct RustProbe<'a> {
    service: &'a str,
    module: &'a str,
    protocol: RustProbeProtocol<'a>,
}
enum RustProbeProtocol<'a> {
    Tcp,
    Tls(Option<&'a str>),
    Udp,
}

#[derive(Copy, Clone)]
pub enum Payload<'a> {
    Empty,
    String(&'a String),
    Binary(&'a Vec<u8>),
}

impl<'a> Extend<&'a ProbeFile> for AllProbes<'a> {
    fn extend<T: IntoIterator<Item = &'a ProbeFile>>(&mut self, iter: T) {
        for service in iter {
            let empty_tcp_probes = &mut self.empty_tcp_probes[service.prevalence as usize];
            let payload_tcp_probes = &mut self.payload_tcp_probes[service.prevalence as usize];
            let rust_tcp_probes = &mut self.rust_tcp_probes[service.prevalence as usize];
            let empty_tls_probes = &mut self.empty_tls_probes[service.prevalence as usize];
            let payload_tls_probes = &mut self.payload_tls_probes[service.prevalence as usize];
            let rust_tls_probes = &mut self.rust_tls_probes[service.prevalence as usize];
            let udp_probes = &mut self.udp_probes[service.prevalence as usize];
            let rust_udp_probes = &mut self.rust_udp_probes[service.prevalence as usize];

            for probe in &service.probes {
                // RustProbe
                if let Some(module) = probe.rust.as_deref() {
                    if probe.tcp {
                        rust_tcp_probes.push(RustProbe {
                            service: &service.service,
                            module,
                            protocol: RustProbeProtocol::Tcp,
                        });
                    }
                    if probe.tls {
                        rust_tls_probes.push(RustProbe {
                            service: &service.service,
                            module,
                            protocol: RustProbeProtocol::Tls(probe.alpn.as_deref()),
                        });
                    }
                    if probe.udp {
                        rust_udp_probes.push(RustProbe {
                            service: &service.service,
                            module,
                            protocol: RustProbeProtocol::Udp,
                        });
                    }
                }
                // RegexProbe
                if let Some(regex) = probe.regex.as_deref() {
                    let payload = None
                        .or(probe.payload_str.as_ref().map(Payload::String))
                        .or(probe.payload_b64.as_ref().map(Payload::Binary))
                        .or(probe.payload_hex.as_ref().map(Payload::Binary))
                        .unwrap_or(Payload::Empty);

                    if probe.tcp {
                        match payload {
                            Payload::Empty => empty_tcp_probes.push(RegexProbe {
                                service: &service.service,
                                regex,
                                sub_regex: probe.sub_regex.as_deref(),
                            }),
                            payload => payload_tcp_probes.push(PayloadProbe {
                                base: RegexProbe {
                                    service: &service.service,
                                    regex,
                                    sub_regex: probe.sub_regex.as_deref(),
                                },
                                payload,
                            }),
                        }
                    }
                    if probe.tls {
                        match payload {
                            Payload::Empty => empty_tls_probes.push(RegexProbe {
                                service: &service.service,
                                regex,
                                sub_regex: probe.sub_regex.as_deref(),
                            }),
                            payload => payload_tls_probes.push(TlsProbe {
                                base: RegexProbe {
                                    service: &service.service,
                                    regex,
                                    sub_regex: probe.sub_regex.as_deref(),
                                },
                                payload,
                                alpn: probe.alpn.as_ref(),
                            }),
                        }
                    }
                    if probe.udp {
                        udp_probes.push(PayloadProbe {
                            base: RegexProbe {
                                service: &service.service,
                                regex,
                                sub_regex: probe.sub_regex.as_deref(),
                            },
                            payload,
                        });
                    }
                }
            }
        }
    }
}
impl<'a> From<&'a [ProbeFile]> for AllProbes<'a> {
    fn from(services: &'a [ProbeFile]) -> Self {
        let mut probes = Self::default();
        probes.extend(services);
        probes
    }
}

impl<'a> fmt::Display for AllProbes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const HEADER: &str = r#"
use once_cell::sync::Lazy;
#[allow(unused_imports)] // Only used by rust probes
use futures::future::FutureExt;

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

        writeln!(f, "    rust_tcp_probes: [")?;
        for group in self.rust_tcp_probes.iter().map(ProbeGroup) {
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

        writeln!(f, "    rust_tls_probes: [")?;
        for group in self.rust_tls_probes.iter().map(ProbeGroup) {
            writeln!(f, "        {group},")?;
        }
        writeln!(f, "    ],")?;

        writeln!(f, "    udp_probes: [")?;
        for group in self.udp_probes.iter().map(ProbeGroup) {
            writeln!(f, "        {group},")?;
        }
        writeln!(f, "    ],")?;

        writeln!(f, "    rust_udp_probes: [")?;
        for group in self.rust_udp_probes.iter().map(ProbeGroup) {
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

impl<'a> fmt::Display for RegexProbe<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            service,
            regex,
            sub_regex,
        } = self;
        write!(f, "RegexProbe {{ service: \"{service}\", regex: Regex::new(r\"(?i-u){regex}\").unwrap(), sub_regex: vec![")?;
        for sub in sub_regex.unwrap_or(&[]) {
            write!(f, "Regex::new(r\"(?i-u){sub}\").unwrap(),")?;
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
            Payload::Binary(b) => write!(f, "{}", encode_binary_string_literal(b))?,
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
            Payload::Binary(b) => write!(f, "{}", encode_binary_string_literal(b))?,
        }
        write!(f, ", alpn: ")?;
        match alpn {
            None => write!(f, "None")?,
            Some(alpn) => write!(f, "Some(\"{alpn}\")")?,
        }
        write!(f, " }}")
    }
}

impl<'a> fmt::Display for RustProbe<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            service,
            module,
            protocol,
        } = self;
        let function = match protocol {
            RustProbeProtocol::Tcp => "probe_tcp",
            RustProbeProtocol::Tls(_) => "probe_tls",
            RustProbeProtocol::Udp => "probe_udp",
        };
        let args = Args(match protocol {
            RustProbeProtocol::Tcp => 1,
            RustProbeProtocol::Tls(_) => 2,
            RustProbeProtocol::Udp => 1,
        });
        write!(
            f,
            "RustProbe {{ service: \"{service}\", function: |{args}| super::probe_impls::{module}::{function}({args}).boxed(), alpn: ")?;
        if let RustProbeProtocol::Tls(Some(alpn)) = protocol {
            write!(f, "Some(\"{alpn}\")")?;
        } else {
            write!(f, "None")?;
        }
        return write!(f, " }}");

        struct Args(usize);
        impl fmt::Display for Args {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                for i in 0..self.0 {
                    write!(f, "arg{i},")?;
                }
                Ok(())
            }
        }
    }
}

fn encode_binary_string_literal(bytes: &[u8]) -> String {
    const ALPHA_HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut ret = String::from("b\"");
    for byte in bytes {
        if *byte == b'"' {
            ret.push_str("\\\"");
        } else if *byte == b'\\' {
            ret.push_str("\\\\");
        } else if *byte >= 0x21 && *byte <= 0x7E {
            ret.push(*byte as char);
        } else {
            ret.push_str("\\x");
            ret.push(ALPHA_HEX[(byte >> 4) as usize] as char);
            ret.push(ALPHA_HEX[(byte & 0xF) as usize] as char);
        }
    }
    ret.push('"');
    ret
}
