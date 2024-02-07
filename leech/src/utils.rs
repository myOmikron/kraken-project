//! Helper utilities

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::num::NonZeroU16;
use std::ops::RangeInclusive;
use std::os::fd::{FromRawFd, IntoRawFd};
use std::str::FromStr;

use get_if_addrs::IfAddr;
use once_cell::sync::Lazy;
use regex::{bytes, Regex};
use thiserror::Error;
use tokio::io::{self, stdin, AsyncBufReadExt, BufReader};
use tokio::net::{TcpSocket, UdpSocket};
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

/// Creates a TCP stream that you can read and write raw IP/TCP packets from/to.
///
/// When using this function you need to manually construct the three-way TCP handshake if you want to establish a
/// connection.
///
/// The return type is a UdpSocket, however it works with TCP streams as well, since TCP streams are made out of TCP
/// datagrams underneath, which need to be manually implemented.
///
/// Note that you need to specify IP headers as well to communicate with other devices.
///
/// The returned socket type is a tokio type and works with the standard async features.
///
/// Receiving on the socket will read all TCP network traffic, make sure you only process what you want to.
///
/// The kernel will still send TCP RST for unknown received SYN/ACKs - you will need to implement a whole lot more if
/// you want to establish a full TCP connection using raw sockets. See also:
/// https://stackoverflow.com/questions/48891727/using-socket-af-packet-sock-raw-but-tell-kernel-to-not-send-rst
pub fn raw_socket(domain: socket2::Domain, protocol: socket2::Protocol) -> io::Result<UdpSocket> {
    let socket = socket2::Socket::new(domain, socket2::Type::RAW, Some(protocol))?;
    socket.set_header_included(true)?;
    socket.set_nonblocking(true)?;

    UdpSocket::from_std(unsafe { std::net::UdpSocket::from_raw_fd(socket.into_raw_fd()) })
}

/// Returns the first IP from the system network interfaces that falls into the same network as destination.
/// If there is no matching, returns the first non-loopback interface's address.
/// If no non-loopback interfaces exist, returns `0.0.0.0` or `[::]` as last resort. (OS will fill it out when sending
/// an IP packet, however the TCP checksum will be wrong and might be dropped)
pub fn find_source_ip(destination: IpAddr) -> std::io::Result<IpAddr> {
    // see http://linux-ip.net/html/routing-saddr-selection.html
    let mut first = match destination {
        IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::from(0)),
        IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::from(0)),
    };
    let mut is_first = true;

    for iface in get_if_addrs::get_if_addrs()?.iter() {
        if iface.is_loopback() {
            continue;
        }

        match (destination, &iface.addr) {
            (IpAddr::V4(dst_v4), IfAddr::V4(src_v4)) => {
                if src_v4.ip & src_v4.netmask == dst_v4 & src_v4.netmask {
                    return Ok(IpAddr::V4(src_v4.ip));
                }
            }
            (IpAddr::V6(dst_v6), IfAddr::V6(src_v6)) => {
                if src_v6.ip & src_v6.netmask == dst_v6 & src_v6.netmask {
                    return Ok(IpAddr::V6(src_v6.ip));
                }
            }
            (_, _) => continue,
        }

        if is_first {
            is_first = false;
            first = iface.ip();
        }
    }

    Ok(first)
}

/// Wraps an OS struct holding a TCP port binding so we own the port while this is held.
pub struct AllocatedPort {
    owner: TcpSocket,
}

impl AllocatedPort {
    /// Returns the allocated port
    pub fn port(&self) -> u16 {
        self.owner
            .local_addr()
            .expect("should have been checked to work in allocate_tcp_port already")
            .port()
    }
}

/// Returns an open TCP port that will be kept available by the OS while it is being held.
pub fn allocate_tcp_port(address: SocketAddr) -> std::io::Result<AllocatedPort> {
    let tcp_port_socket = match address {
        SocketAddr::V4(_) => {
            let r = TcpSocket::new_v4()?;
            r.bind("0.0.0.0:0".parse().unwrap())?;
            r
        }
        SocketAddr::V6(_) => {
            let r = TcpSocket::new_v6()?;
            r.bind("[::]:0".parse().unwrap())?;
            r
        }
    };

    tcp_port_socket.local_addr()?;

    Ok(AllocatedPort {
        owner: tcp_port_socket,
    })
}
