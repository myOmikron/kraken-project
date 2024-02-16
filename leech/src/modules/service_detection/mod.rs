//! This module implements detecting a service behind a port

mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated_probes.rs"));
}
mod error;

/// TCP service detection
pub mod tcp;
/// UDP service detection & port scanning.
pub mod udp;

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
type DynResult<T> = Result<T, DynError>;

/// The detected service or a list of possible candidates
#[derive(Debug, Clone)]
pub enum Service {
    /// The service is unknown
    Unknown,

    /// The service might be one of the list
    Maybe(Vec<&'static str>),

    /// The service has been identified
    Definitely(&'static str),
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
