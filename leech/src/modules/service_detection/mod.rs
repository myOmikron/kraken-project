mod generated;
mod tls;

use std::net::SocketAddr;

use log::{debug, trace, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

pub async fn detect_service(socket: SocketAddr) -> DynResult<()> {
    let tcp_banner = probe_tcp(socket, b"").await?;
    for prev in 0..3 {
        for probe in &generated::PROBES.empty_tcp_probes[prev] {
            if probe.is_match(&tcp_banner) {
                println!("Detected {}", probe.service);
            }
        }

        for probe in &generated::PROBES.payload_tcp_probes[prev] {
            let data = probe_tcp(socket, probe.payload).await?;
            trace!(target: probe.service, "Got data: {:?}", DebuggableBytes(&data));
            if probe.is_match(&data) {
                println!("Detected {}", probe.service);
            }
        }
    }

    match tls::probe(socket, b"").await? {
        Ok(tls_banner) => {
            println!("Detected tls");

            for prev in 0..3 {
                for probe in &generated::PROBES.empty_tls_probes[prev] {
                    if probe.is_match(&tls_banner) {
                        println!("Detected {}", probe.service);
                    }
                }

                for probe in &generated::PROBES.payload_tls_probes[prev] {
                    match tls::probe(socket, probe.payload).await? {
                        Ok(data) => {
                            trace!(target: probe.service, "Got data: {:?}", DebuggableBytes(&data));
                            if probe.is_match(&data) {
                                println!("Detected {}", probe.service);
                            }
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

    // TODO impl udp

    Ok(())
}

async fn probe_tcp(socket: SocketAddr, payload: &[u8]) -> DynResult<Vec<u8>> {
    let mut tcp = TcpStream::connect(socket).await?;
    tcp.write_all(payload.as_ref()).await?;
    tcp.shutdown().await?;

    let mut data = Vec::new();
    tcp.read_to_end(&mut data).await?;

    Ok(data)
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
// http over TLS
// http2 [DONE]
// http2 over TLS
// all databases
// - postgres [DONE]
// - mysql
// - mariadb [DONE]
// - sqlite [XXX]
// tls (StartTLS)
// rdp
// kerberos
// netbios
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
