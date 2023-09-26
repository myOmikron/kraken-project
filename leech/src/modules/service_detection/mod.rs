//! This module implements detecting a service behind a port

use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use log::{debug, info, trace, warn};
use probe_config::generated::Match;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated_probes.rs"));
}
mod error;
mod postgres;

use self::error::{Extended, ResultExt};

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
type DynResult<T> = Result<T, DynError>;

/// Settings for a service detection
pub struct DetectServiceSettings {
    /// Socket to scan
    pub socket: SocketAddr,

    /// Time to wait for a response after sending the payload
    /// (or after establishing a connection, if not payload is to be sent)
    pub wait_for_response: Duration,

    /// Always run all probes and test to completion instead of returning the first exact match.
    ///
    /// The first exact match is still the one being returned, but everything will be tested to produce logs for debugging.
    pub always_run_everything: bool,
}

/// The detected service or a list of possible candidates
#[derive(Debug)]
pub enum Service {
    /// The service is unknown
    Unknown,

    /// The service might be one of the list
    Maybe(Vec<&'static str>),

    /// The service has been identified
    Definitely(&'static str),
}

/// Detect the service behind a socket by talking to it
pub async fn detect_service(settings: DetectServiceSettings) -> DynResult<Service> {
    let mut exact_match = None;
    let mut partial_matches = Vec::new();

    /// Call `is_match` on a probe with the given data and handle the result
    ///
    /// This is a macro because it needs mutable access to stack variables and the ability to return early.
    macro_rules! check_match {
        ($probe:expr, $haystack:expr) => {
            match $probe.is_match($haystack) {
                Match::No => {}
                Match::Partial => {
                    partial_matches.push($probe.service);
                }
                Match::Exact => {
                    if settings.always_run_everything {
                        info!("Found exact match: {}", $probe.service);
                        exact_match.get_or_insert($probe.service);
                    } else {
                        return Ok(Service::Definitely($probe.service));
                    }
                }
            }
        };
    }

    debug!("Retrieving tcp banner");
    let tcp_banner = settings.probe_tcp(b"").await?;
    for prev in 0..3 {
        if let Some(tcp_banner) = tcp_banner.as_deref() {
            debug!("Scanning tcp banner #{prev}");
            for probe in &generated::PROBES.empty_tcp_probes[prev] {
                check_match!(probe, tcp_banner);
            }
        }

        debug!("Starting tcp payload scans #{prev}");
        for probe in &generated::PROBES.payload_tcp_probes[prev] {
            if let Some(data) = settings.probe_tcp(probe.payload).await? {
                check_match!(probe, &data);
            }
        }
    }

    debug!(target: "regex", "Starting tls banner scan");
    match settings.probe_tls(b"", None).await? {
        Ok(tls_banner) => {
            partial_matches.push("tls");

            for prev in 0..3 {
                for probe in &generated::PROBES.empty_tls_probes[prev] {
                    check_match!(probe, &tls_banner);
                }

                debug!(target: "regex", "Starting tls payload scans");
                for probe in &generated::PROBES.payload_tls_probes[prev] {
                    match settings.probe_tls(probe.payload, probe.alpn).await? {
                        Ok(data) => {
                            trace!(target: probe.service, "Got data over tls: {:?}", DebuggableBytes(&data));
                            check_match!(probe, &data);
                        }
                        Err(err) => {
                            warn!(target: "tls", "Failed to connect while probing {}: {err}", probe.service)
                        }
                    }
                }
            }
        }
        Err(err) => debug!(target: "tls", "TLS error: {err:?}"),
    }

    if postgres::probe(&settings).await? {
        if settings.always_run_everything {
            exact_match.get_or_insert("postgres");
        } else {
            return Ok(Service::Definitely("postgres"));
        }
    }

    // TODO impl udp

    if let Some(exact_match) = exact_match {
        Ok(Service::Definitely(exact_match))
    } else if !partial_matches.is_empty() {
        Ok(Service::Maybe(partial_matches))
    } else {
        Ok(Service::Unknown)
    }
}

impl DetectServiceSettings {
    /// Send `payload` and receive answer over TCP
    ///
    /// Errors when an unrecoverable error occurred.
    /// Returns `Ok(None)` when the service refused to respond to the payload.
    async fn probe_tcp(&self, payload: &[u8]) -> DynResult<Option<Vec<u8>>> {
        match self.raw_probe_tcp(payload).await {
            Ok(data) => Ok(Some(data)),
            Err(error) => match error.kind() {
                io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted => Ok(None),
                io::ErrorKind::NotConnected if error.context == "TcpStream::shutdown" => Ok(None),
                _ => Err(error.into()),
            },
        }
    }

    /// 1. Connect to the socket using tcp
    /// 2. Send `payload`
    /// 3. Wait for the configured amount of time
    /// 4. Return everything which has been received
    async fn raw_probe_tcp(&self, payload: &[u8]) -> Result<Vec<u8>, Extended<io::Error>> {
        // Connect
        let mut tcp = TcpStream::connect(self.socket)
            .await
            .context("TcpStream::connect")?;

        // Send payload
        if !payload.is_empty() {
            tcp.write_all(payload)
                .await
                .context("TcpStream::write_all")?;
            tcp.flush().await.context("TcpStream::flush")?;
            trace!(target: "tcp", "Send data: {:?}", DebuggableBytes(payload));
        }

        // Wait
        sleep(self.wait_for_response).await;

        // Read
        tcp.shutdown().await.context("TcpStream::shutdown")?;
        let mut data = Vec::new();
        tcp.read_to_end(&mut data)
            .await
            .context("TcpStream::read_to_end")?;

        // Log and Return
        trace!(target: "tcp", "Received data: {:?}", DebuggableBytes(&data));
        Ok(data)
    }

    /// 1. Connect to the socket using tls over tcp
    /// 2. Send `payload`
    /// 3. Wait for the configured amount of time
    /// 4. Return everything which has been received
    async fn probe_tls(
        &self,
        payload: &[u8],
        alpn: Option<&str>,
    ) -> DynResult<Result<Vec<u8>, native_tls::Error>> {
        // Configure TLS
        let alpns = alpn.as_ref().map(std::slice::from_ref).unwrap_or(&[]);
        let connector = tokio_native_tls::TlsConnector::from(
            native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .use_sni(false)
                .request_alpns(alpns)
                .build()?,
        );

        // Connect
        let tcp = TcpStream::connect(self.socket)
            .await
            .context("TcpStream::connect")?;
        let mut tls = match connector.connect("<ignored>", tcp).await {
            Ok(tls) => tls,
            Err(err) => return Ok(Err(err)),
        };

        // Send payload
        if !payload.is_empty() {
            tls.write_all(payload)
                .await
                .context("TlsStream::write_all")?;
            tls.flush().await.context("TlsStream::flush")?;
        }

        // Wait
        sleep(self.wait_for_response).await;

        // Read and Close
        tls.shutdown().await.context("TlsStream::shutdown")?;
        let mut data = Vec::new();
        tls.read_to_end(&mut data)
            .await
            .context("TlsStream::read_to_end")?;

        // Log and Return
        trace!(target: "tls", "Received data: {:?}", DebuggableBytes(&data));
        Ok(Ok(data))
    }
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

// ftp
// http [DONE]
// https [DONE]
// http2 [DONE]
// http2 over TLS [DONE]
// all databases
// - postgres [DONE]
// - mysql
// - mariadb [DONE]
// - sqlite [XXX]
// tls (StartTLS)
// rdp
// kerberos
// netbios [DONE]
// microsoft ds
// snmp (trap)
// ssh [DONE]
// smtp
// pop3
// imap
// IPSec
// ldap
// upnp
// grpc

// telnet
// dns
// dhcp
// vnc
// rsync
// ipam
// radius
// bittorrent
// sip
// openvpn
// wireguard
// tinc vpn
// samba
// nfs
// redis
// tor
// bgp
// dicom
// sftp
// syslog
// rtsp
// quick
// socks
// wins
// ipmi
// mqtt
// cvs
// svn
// sieve
// nrpe
// teamviewer
// x11
// veeam
// check mk
// esxi
// git
// zabbix
// NSClient
// minecraft
// jenkins
