//! parser and data structures for SPF TXT entries

use std::fmt::Display;
use std::str::FromStr;

use ipnetwork::IpNetwork;
use log::debug;
use thiserror::Error;

/*
parses SPF strings like this:

a:mail03.viwork.net a:mail04.viwork.net mx:mailrelay.viwork.net include:spf.protection.outlook.com include:spf-de.emailsignatures365.com -all
include:_spf-a.microsoft.com include:_spf-b.microsoft.com include:_spf-c.microsoft.com include:_spf-ssg-a.msft.net include:spf-a.hotmail.com include:_spf1-meo.microsoft.com -all
include:_spf.google.com ~all
include:_spf.google.com ~all
include:_spf.protonmail.ch mx ip4:202.61.254.194 ~all
include:mailer.duckduckgo.com include:amazonses.com include:spf.protection.outlook.com -all
ip4:212.224.101.32 ip4:159.100.31.138 ip6:2a01:7e0:0:259::32 include:amazonses.com -all
mx ?all
mx ip4:87.139.193.6 ip4:212.227.181.119 ~all
*/
use crate::utils::RE;

/// In the (whitespace delimited) SPF string this is one part of it such as `~all` or `include:_spf.example.org`
#[derive(Debug, Clone)]
pub enum SPFPart {
    /// SPF Directive as defined per RFC, if this matches a given sender this controls what happens to the SPF result.
    Directive {
        /// This is the return value if the mechanism matches (pass or fail)
        qualifier: SPFQualifier,
        /// The mechanism to match the sender on
        mechanism: SPFMechanism,
    },
    /// Can affect what the SPF result does if no previous directives match by asking the client to lookup the given domain.
    RedirectModifier {
        /// domain to query, as per RFC
        domain: String,
    },
    /// Refers to another domain that has a TXT entry explaining why the SPF lookup would fail in a more human readable format (possibly with macros).
    ExplanationModifier {
        /// domain to query for explanation, as per RFC
        domain: String,
    },
    /// Other unknown modifiers, parsed anyway.
    UnknownModifier {
        /// key (before equals sign)
        name: String,
        /// value (after equals sign)
        value: String,
    },
}

impl SPFPart {
    /// Converts this SPF mechanism to a string that could be used in the DNS
    /// txt entry. Has no spaces within, since those are escaped, so multiple
    /// parts can be joined and split with spaces.
    pub fn encode_spf(&self) -> String {
        match self {
            SPFPart::Directive {
                qualifier,
                mechanism,
            } => format!("{qualifier}{}", mechanism.encode_spf()),
            SPFPart::RedirectModifier { domain } => {
                format!("redirect={}", encode_spf_domain_spec(domain))
            }
            SPFPart::ExplanationModifier { domain } => {
                format!("exp={}", encode_spf_domain_spec(domain))
            }
            SPFPart::UnknownModifier { name, value } => {
                format!("{name}={value}")
            }
        }
    }
}

impl Display for SPFPart {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SPFPart::Directive {
                qualifier,
                mechanism,
            } => write!(f, "{}{:?}", qualifier, mechanism),
            SPFPart::RedirectModifier { domain } => write!(f, "redirect={}", domain),
            SPFPart::ExplanationModifier { domain } => write!(f, "exp={}", domain),
            SPFPart::UnknownModifier { name, value } => write!(f, "{}={}", name, value),
        }
    }
}

/// Represents is the return value if a mechanism matches (pass or fail)
#[derive(Debug, Clone)]
pub enum SPFQualifier {
    /// equal to spf '+' (default value)
    Pass,
    /// equal to spf '-'
    Fail,
    /// equal to spf '~'
    SoftFail,
    /// equal to spf '?'
    Neutral,
}

impl Display for SPFQualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SPFQualifier::Pass => write!(f, "+"),
            SPFQualifier::Fail => write!(f, "-"),
            SPFQualifier::SoftFail => write!(f, "~"),
            SPFQualifier::Neutral => write!(f, "?"),
        }
    }
}

/// Error in case of SPFQualifier char -> enum conversion
#[derive(Error, Debug)]
#[error("Invalid qualifier: {0}")]
pub struct SPFQualifierError(u8);

impl TryFrom<u8> for SPFQualifier {
    type Error = SPFQualifierError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'+' => Ok(SPFQualifier::Pass),
            b'-' => Ok(SPFQualifier::Fail),
            b'~' => Ok(SPFQualifier::SoftFail),
            b'?' => Ok(SPFQualifier::Neutral),
            _ => Err(SPFQualifierError(value)),
        }
    }
}

/// Describes a SPF mechanism that can match a given sender, or in our case,
/// includes IPs and domains that are likely in the owners control or otherwise
/// related to the scanned domain.
#[derive(Debug, Clone)]
pub enum SPFMechanism {
    /// spf 'all'
    All,
    /// spf 'include:DOMAIN'
    Include {
        /// Domain from the 'include' directive
        domain: String,
    },
    /// spf 'a[:DOMAIN][/32][//128]'
    A {
        /// Domain from the 'a' directive or empty if omitted
        domain: String,
        /// If specified, IPv4 CIDR (prefix length)
        ipv4_cidr: Option<u8>,
        /// If specified, IPv6 CIDR (prefix length)
        ipv6_cidr: Option<u8>,
    },
    /// spf 'mx[:DOMAIN][/32][//128]'
    MX {
        /// Domain from the 'mx' directive or empty if omitted
        domain: String,
        /// If specified, IPv4 CIDR (prefix length)
        ipv4_cidr: Option<u8>,
        /// If specified, IPv6 CIDR (prefix length)
        ipv6_cidr: Option<u8>,
    },
    /// spf 'ptr[:DOMAIN]'
    PTR {
        /// Domain from the 'ptr' directive or empty if omitted
        domain: String,
    },
    /// spf 'ip4:IP' and 'ip6:IP'
    IP {
        /// IPv4 or IPv6 with embedded CIDR
        ipnet: IpNetwork,
    },
    /// spf 'exists:DOMAIN'
    Exists {
        /// Domain from the 'exists' directive
        domain: String,
    },
}

impl SPFMechanism {
    fn encode_spf(&self) -> String {
        match self {
            SPFMechanism::All => "all".to_owned(),
            SPFMechanism::Include { domain } => {
                format!("include:{}", encode_spf_domain_spec(domain))
            }
            SPFMechanism::A {
                domain,
                ipv4_cidr,
                ipv6_cidr,
            } => {
                let mut ret = String::with_capacity(10 + domain.len());
                if domain.is_empty() {
                    ret.push('a');
                } else {
                    ret.push_str("a:");
                    ret.push_str(encode_spf_domain_spec(domain).as_str());
                }
                if let Some(cidr) = ipv4_cidr {
                    ret.push('/');
                    ret.push_str((*cidr).to_string().as_str());
                }
                if let Some(cidr) = ipv6_cidr {
                    ret.push_str("//");
                    ret.push_str((*cidr).to_string().as_str());
                }
                ret
            }
            SPFMechanism::MX {
                domain,
                ipv4_cidr,
                ipv6_cidr,
            } => {
                let mut ret = String::with_capacity(10 + domain.len());
                if domain.is_empty() {
                    ret.push_str("mx");
                } else {
                    ret.push_str("mx:");
                    ret.push_str(encode_spf_domain_spec(domain).as_str());
                }
                if let Some(cidr) = ipv4_cidr {
                    ret.push('/');
                    ret.push_str((*cidr).to_string().as_str());
                }
                if let Some(cidr) = ipv6_cidr {
                    ret.push_str("//");
                    ret.push_str((*cidr).to_string().as_str());
                }
                ret
            }
            SPFMechanism::PTR { domain } => {
                if domain.is_empty() {
                    "ptr".to_owned()
                } else {
                    format!("ptr:{}", encode_spf_domain_spec(domain))
                }
            }
            SPFMechanism::IP { ipnet } => match ipnet {
                IpNetwork::V4(v4) => format!("ip4:{}", v4),
                IpNetwork::V6(v6) => format!("ip6:{}", v6),
            },
            SPFMechanism::Exists { domain } => format!("exists:{}", encode_spf_domain_spec(domain)),
        }
    }
}

/// Parses an SPF record, skips invalid parts and only returns valid ones.
pub fn parse_spf(spf_in: &[u8]) -> Vec<SPFPart> {
    let mut spf = spf_in;
    let mut ret = Vec::new();

    while !spf.is_empty() {
        if let Some(start) = spf.iter().position(|&x| !x.is_ascii_whitespace()) {
            spf = &spf[start..];
        } else {
            break;
        }

        let end = spf
            .iter()
            .position(|&x| x.is_ascii_whitespace())
            .unwrap_or(spf.len());

        if let Some(part) = parse_spf_part(&spf[0..end]) {
            ret.push(part)
        } else {
            debug!(
                "could not parse SPF part '{}'",
                std::str::from_utf8(&spf[0..end]).unwrap_or("<invalid UTF8>")
            );
        }

        spf = &spf[end..];
    }

    ret
}

fn extract_mechanism<'a>(part_ascii: &'a [u8], expected: &[u8]) -> Option<&'a [u8]> {
    if starts_with_ci(part_ascii, expected) {
        if part_ascii.len() == expected.len() {
            return Some(&part_ascii[expected.len()..]);
        } else if part_ascii[expected.len()] == b':' {
            return Some(&part_ascii[expected.len() + 1..]); // skip over leading colon
        } else if part_ascii[expected.len()] == b'/' {
            return Some(&part_ascii[expected.len()..]);
        }
    }
    None
}

fn equals_ci(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        false
    } else {
        for i in 0..a.len() {
            if !a[i].eq_ignore_ascii_case(&b[i]) {
                return false;
            }
        }
        true
    }
}

fn starts_with_ci(does_this: &[u8], start_with: &[u8]) -> bool {
    if start_with.len() > does_this.len() {
        false
    } else {
        for i in 0..start_with.len() {
            if !does_this[i].eq_ignore_ascii_case(&start_with[i]) {
                return false;
            }
        }
        true
    }
}

fn parse_spf_part(part_ascii: &[u8]) -> Option<SPFPart> {
    if part_ascii.is_empty() {
        None
    } else if let Ok(qualifier) = SPFQualifier::try_from(part_ascii[0]) {
        Some(SPFPart::Directive {
            qualifier,
            mechanism: parse_spf_mechanism(&part_ascii[1..])?,
        })
    } else if equals_ci(part_ascii, b"all")
        || starts_with_ci(part_ascii, b"include:")
        || extract_mechanism(part_ascii, b"a").is_some()
        || extract_mechanism(part_ascii, b"mx").is_some()
        || extract_mechanism(part_ascii, b"ptr").is_some()
        || starts_with_ci(part_ascii, b"ip4:")
        || starts_with_ci(part_ascii, b"ip6:")
        || starts_with_ci(part_ascii, b"exists:")
    {
        Some(SPFPart::Directive {
            qualifier: SPFQualifier::Pass,
            mechanism: parse_spf_mechanism(part_ascii)?,
        })
    } else {
        parse_spf_modifier(part_ascii)
    }
}

fn parse_spf_mechanism(part_ascii: &[u8]) -> Option<SPFMechanism> {
    if equals_ci(part_ascii, b"all") {
        Some(SPFMechanism::All)
    } else if starts_with_ci(part_ascii, b"include:") {
        Some(SPFMechanism::Include {
            domain: parse_spf_domain_spec(&part_ascii[8..], false)?,
        })
    } else if let Some(remaining) = extract_mechanism(part_ascii, b"a") {
        let (domain, ipv4_cidr, ipv6_cidr) = parse_spf_domain_spec_and_cidr(remaining, true)?;
        Some(SPFMechanism::A {
            domain,
            ipv4_cidr,
            ipv6_cidr,
        })
    } else if let Some(remaining) = extract_mechanism(part_ascii, b"mx") {
        let (domain, ipv4_cidr, ipv6_cidr) = parse_spf_domain_spec_and_cidr(remaining, true)?;
        Some(SPFMechanism::MX {
            domain,
            ipv4_cidr,
            ipv6_cidr,
        })
    } else if let Some(remaining) = extract_mechanism(part_ascii, b"ptr") {
        Some(SPFMechanism::PTR {
            domain: parse_spf_domain_spec(remaining, true)?,
        })
    } else if starts_with_ci(part_ascii, b"ip4:") || starts_with_ci(part_ascii, b"ip6:") {
        Some(SPFMechanism::IP {
            ipnet: IpNetwork::from_str(std::str::from_utf8(&part_ascii[4..]).ok()?).ok()?,
        })
    } else if starts_with_ci(part_ascii, b"exists:") {
        Some(SPFMechanism::Exists {
            domain: parse_spf_domain_spec(&part_ascii[7..], false)?,
        })
    } else {
        None
    }
}

fn parse_spf_domain_spec_and_cidr(
    part_ascii: &[u8],
    optional: bool,
) -> Option<(String, Option<u8>, Option<u8>)> {
    let mut ipv4_cidr: Option<u8> = None;
    let mut ipv6_cidr: Option<u8> = None;
    let mut end = part_ascii.len();
    if let Some(slash) = part_ascii.iter().rposition(|&x| x == b'/') {
        if slash > 0 && part_ascii[slash - 1] == b'/' {
            ipv6_cidr =
                Some(u8::from_str(std::str::from_utf8(&part_ascii[slash + 1..]).ok()?).ok()?);
            if let Some(slash) = part_ascii[..slash - 1].iter().rposition(|&x| x == b'/') {
                ipv4_cidr =
                    Some(u8::from_str(std::str::from_utf8(&part_ascii[slash + 1..]).ok()?).ok()?);
                end = slash - 1;
            } else {
                end = slash - 2;
            }
        } else {
            ipv4_cidr =
                Some(u8::from_str(std::str::from_utf8(&part_ascii[slash + 1..]).ok()?).ok()?);
            end = slash - 1;
        }
    }

    Some((
        parse_spf_domain_spec(&part_ascii[..end], optional)?,
        ipv4_cidr,
        ipv6_cidr,
    ))
}

fn parse_spf_domain_spec(part_ascii: &[u8], optional: bool) -> Option<String> {
    if part_ascii.is_empty() && optional {
        return Some("".to_owned());
    }

    if !RE.spf_domain_spec.is_match(part_ascii) {
        return None;
    }

    let mut ret = String::new();
    let mut escape = false;

    for c in part_ascii {
        if escape {
            match *c {
                b'%' => ret.push('%'),
                b'_' => ret.push(' '),
                b'-' => ret.push_str("%20"),
                _ => return None, // for now we just skip over domains with macros or unknown parts
            }
            escape = false;
        } else if *c == b'%' {
            escape = true;
        } else {
            ret.push(*c as char);
        }
    }

    Some(ret)
}

fn encode_spf_domain_spec(domain_spec: &str) -> String {
    domain_spec.replace('%', "%%").replace(' ', "%_")
}

fn parse_spf_modifier(part_ascii: &[u8]) -> Option<SPFPart> {
    if let Some(eq) = part_ascii.iter().position(|&x| x == b'=') {
        let lhs = &part_ascii[..eq];
        let rhs = &part_ascii[eq + 1..];
        if equals_ci(lhs, b"redirect") {
            Some(SPFPart::RedirectModifier {
                domain: parse_spf_domain_spec(rhs, false)?,
            })
        } else if equals_ci(lhs, b"exp") {
            Some(SPFPart::ExplanationModifier {
                domain: parse_spf_domain_spec(rhs, false)?,
            })
        } else {
            Some(SPFPart::UnknownModifier {
                name: String::from_utf8(lhs.to_vec()).ok()?,
                value: String::from_utf8(rhs.to_vec()).ok()?,
            })
        }
    } else {
        None
    }
}
