//! This module implements detecting a service behind a port

use std::collections::BTreeMap;

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

    /// The service might be one of the keys
    ///
    /// The values store the transport protocols their service was potentially detected on.
    Maybe(BTreeMap<&'static str, ProtocolSet>),

    /// The service has been identified
    Definitely {
        /// The service's name
        service: &'static str,

        /// The transport protocols the service runs on
        protocols: ProtocolSet,
    },
}

/// Set storing which transport protocols a service runs on
// Not implemented as actual bitset for programmer convenience
#[derive(Default, Copy, Clone, Debug)]
pub struct ProtocolSet {
    /// TCP
    pub tcp: bool,

    /// TLS over TCP
    pub tls: bool,

    /// UDP
    pub udp: bool,
}

impl ProtocolSet {
    /// Empty set
    pub const NONE: Self = Self {
        tcp: false,
        tls: false,
        udp: false,
    };

    /// Set containing only `tcp`
    pub const TCP: Self = Self {
        tcp: true,
        ..Self::NONE
    };

    /// Set containing only `tls`
    pub const TLS: Self = Self {
        tls: true,
        ..Self::NONE
    };

    /// Set containing only `udp`
    pub const UDP: Self = Self {
        udp: true,
        ..Self::NONE
    };

    /// Merges `self` with `other` and store the result in `self`
    pub fn update(&mut self, other: Self) {
        self.tcp |= other.tcp;
        self.tls |= other.tls;
        self.udp |= other.udp;
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
