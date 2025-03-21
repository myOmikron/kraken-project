//! This module implements detecting a service behind a port

use std::collections::BTreeSet;

use serde::Serialize;

mod error;

mod probe_impls;

mod generated;
/// TCP service detection
pub mod tcp;
/// UDP service detection & port scanning.
pub mod udp;

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
type DynResult<T> = Result<T, DynError>;

/// The detected service or a list of possible candidates
#[derive(Debug, Serialize, Clone)]
pub enum Service {
    /// The service is unknown
    Unknown,

    /// The service might be one of the keys
    ///
    /// The values store the transport protocols their service was potentially detected on.
    Maybe(BTreeSet<&'static str>),

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
