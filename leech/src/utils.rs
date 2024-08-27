//! Helper utilities

use std::future::Future;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::num::NonZeroU16;
use std::num::NonZeroUsize;
use std::ops::RangeInclusive;
use std::os::fd::FromRawFd;
use std::os::fd::IntoRawFd;
use std::panic;
use std::str::FromStr;

use nix::ifaddrs::getifaddrs;
use nix::sys::socket::AddressFamily;
use nix::sys::socket::SockaddrLike;
use once_cell::sync::Lazy;
use regex::bytes;
use regex::Regex;
use thiserror::Error;
use tokio::io::stdin;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::io::{self};
use tokio::net::TcpSocket;
use tokio::net::UdpSocket;
use tokio::task::JoinError;
use tokio::task::JoinSet;
use tonic::transport::Certificate;
use tonic::transport::ClientTlsConfig;
use tonic::transport::Endpoint;

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
///
/// If `ports` is empty, an optional `default` can be used to populate the `Vec`.
pub fn parse_ports(
    ports: &[String],
    default: Option<RangeInclusive<u16>>,
) -> Result<Vec<RangeInclusive<u16>>, ParsePortError> {
    let mut parsed = Vec::new();

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

                    parsed.push(start..=end);
                } else if let Some(m) = captures.name("single") {
                    if let Ok(v) = NonZeroU16::from_str(m.as_str()) {
                        let port = u16::from(v);
                        parsed.push(port..=port);
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

    if ports.is_empty() {
        parsed.extend(default);
    }

    Ok(parsed)
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

    for iface in getifaddrs()? {
        let Some(address) = iface.address else {
            continue;
        };
        let Some(netmask) = iface.netmask else {
            continue;
        };

        match (destination, address.family()) {
            (IpAddr::V4(dst_v4), Some(AddressFamily::Inet)) => {
                let src_v4 = address
                    .as_sockaddr_in()
                    .expect("family is inet, this must be set");
                let src_v4 = src_v4.ip();
                if let Some(netmask) = netmask.as_sockaddr_in() {
                    let netmask = netmask.ip();
                    if src_v4 & netmask == dst_v4 & netmask {
                        return Ok(IpAddr::from(src_v4));
                    }
                }
            }
            (IpAddr::V6(dst_v6), Some(AddressFamily::Inet6)) => {
                let src_v6 = address
                    .as_sockaddr_in6()
                    .expect("family is inet6, this must be set");
                let src_v6 = src_v6.ip();
                if let Some(netmask) = netmask.as_sockaddr_in6() {
                    let netmask = netmask.ip();
                    if src_v6 & netmask == dst_v6 & netmask {
                        return Ok(IpAddr::V6(src_v6));
                    }
                }
            }
            (_, _) => continue,
        }

        if is_first {
            if let Some(ip) = match address.family() {
                Some(AddressFamily::Inet) => Some(IpAddr::from(
                    address
                        .as_sockaddr_in()
                        .expect("family is inet, this must be set")
                        .ip(),
                )),
                Some(AddressFamily::Inet6) => Some(IpAddr::from(
                    address
                        .as_sockaddr_in6()
                        .expect("family is inet6, this must be set")
                        .ip(),
                )),
                _ => None,
            } {
                if !ip.is_loopback() {
                    is_first = false;
                    first = ip;
                }
            }
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

/// Wrapper around byte slice with an informative [`Debug`] impl
pub struct DebuggableBytes<'a>(pub &'a [u8]);
impl<'a> std::fmt::Debug for DebuggableBytes<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(bytes) = self;
        if let Ok(string) = std::str::from_utf8(bytes) {
            write!(f, "{string:?}")
        } else {
            write!(f, "{bytes:x?}")
        }
    }
}

/// Extension trait to provide additional methods on iterators:
///
/// - [`IteratorExt::try_for_each_concurrent`]
pub(crate) trait IteratorExt: Iterator + Sized {
    /// Runs a fallible async function concurrently on each item in the iterator,
    /// stopping at the first error and returning that error.
    ///
    /// This is semantically the same as calling
    /// [`TryStreamExt::try_for_each_concurrent`](futures::stream::TryStreamExt::try_for_each_concurrent)
    /// on a stream returned by
    /// [`stream::iter`](futures::stream::iter)
    /// but without the weird conversion and using modern rust with tokio.
    async fn try_for_each_concurrent<Fut, Err>(
        self,
        limit: Option<NonZeroUsize>,
        f: impl FnOnce(Self::Item) -> Fut + Clone,
    ) -> Result<(), Err>
    where
        Fut: Future<Output = Result<(), Err>> + Send + 'static,
        Err: Send + 'static,
    {
        let limit = limit.map(NonZeroUsize::get);
        let mut tasks = JoinSet::new();

        for item in self {
            tasks.spawn((f.clone())(item));

            if Some(tasks.len()) == limit {
                // Since the `limit` is non-zero,
                // the above condition will only hold if `tasks.len()` is also non-zero
                #[allow(clippy::expect_used)]
                let join_result = tasks
                    .join_next()
                    .await
                    .expect("There should be at least one task");

                handle_join_result(join_result)?;
            }
        }

        while let Some(join_result) = tasks.join_next().await {
            handle_join_result(join_result)?;
        }

        Ok(())
    }
}
impl<I: Iterator + Sized> IteratorExt for I {}
fn handle_join_result<T>(join_result: Result<T, JoinError>) -> T {
    join_result.unwrap_or_else(|join_error| {
        if join_error.is_panic() {
            panic::resume_unwind(join_error.into_panic())
        } else {
            unreachable!("No task should be canceled while the JoinSet is around")
        }
    })
}
