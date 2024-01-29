//! TCP fingerprinting by looking at default values
//!
//! See https://en.wikipedia.org/wiki/TCP/IP_stack_fingerprinting

use std::io;
use std::net::{SocketAddr, SocketAddrV4};

use etherparse::{
    InternetSlice, IpHeader, Ipv4Extensions, Ipv4Header, Ipv6ExtensionSlice, Ipv6Extensions,
    Ipv6ExtensionsSlice, Ipv6Header, Ipv6RoutingExtensions, PacketBuilder, SlicedPacket, TcpHeader,
    TcpOptionElement, TcpOptionReadError, TransportSlice,
};
use socket2::{Domain, Protocol};
use tokio::net::TcpSocket;

use crate::modules::os_detection::errors::RawTcpError;
use crate::utils::raw_socket;

async fn tcp_get_syn_ack(
    address: SocketAddr,
    recv_port: u16,
    syn: &[u8],
) -> Result<(IpHeader, TcpHeader), RawTcpError> {
    let mut buf = [0u8; 4096];
    let socket = raw_socket(Domain::for_address(address), Protocol::TCP)?;

    let recv_filter = etherparse::packet_filter::TransportFilter::Tcp {
        source_port: None,
        destination_port: Some(recv_port),
    };

    socket.send_to(&syn, address).await?;

    let mut buf = [0u8; 256];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        // TODO: rely on OS recv_from address or parse IP header instead?
        // need to check what routing does to these fields!
        if address.ip() != addr.ip() {
            continue;
        }

        let packet = SlicedPacket::from_ip(&buf[0..len])?;

        if let Some(transport_slice) = packet.transport {
            if recv_filter.applies_to_slice(&transport_slice) {
                return Ok(match transport_slice {
                    TransportSlice::Tcp(tcp) => (match packet.ip.expect("must have ip since we decoded from ip") {
                        InternetSlice::Ipv4(ip, ext) => IpHeader::Version4(
                            ip.to_header(),
                            ext.to_header()
                        ),
                        InternetSlice::Ipv6(ip, ext) => IpHeader::Version6(
                            ip.to_header(),
                            make_ipv6_headers(&ext)
                        )
                    }, tcp.to_header()),
                    _ => panic!("should never reach this since the TransportFilter::Tcp (recv_filter) matched on this")
                });
            }
        }
    }
}

fn make_ipv6_headers(ext: &Ipv6ExtensionsSlice) -> Ipv6Extensions {
    let mut ret = Ipv6Extensions {
        routing: None,
        hop_by_hop_options: None,
        fragment: None,
        destination_options: None,
        auth: None,
    };

    for header in ext.clone().into_iter() {
        match header {
            Ipv6ExtensionSlice::HopByHop(ext) => ret.hop_by_hop_options = Some(ext.to_header()),
            Ipv6ExtensionSlice::Routing(ext) => {
                ret.routing = Some(Ipv6RoutingExtensions {
                    routing: ext.to_header(),
                    final_destination_options: None,
                })
            }
            Ipv6ExtensionSlice::Fragment(ext) => ret.fragment = Some(ext.to_header()),
            Ipv6ExtensionSlice::DestinationOptions(ext) => {
                ret.destination_options = Some(ext.to_header())
            }
            Ipv6ExtensionSlice::Authentication(ext) => ret.auth = Some(ext.to_header()),
        }
    }

    return ret;
}

fn allocate_tcp_port(address: SocketAddr) -> io::Result<u16> {
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

    return Ok(tcp_port_socket.local_addr()?.port());
}

#[derive(Debug)]
pub struct TcpFingerprint {
    pub ip: IpHeader,
    pub tcp: TcpHeader,
}

pub async fn fingerprint_tcp(address: SocketAddr) -> Result<TcpFingerprint, RawTcpError> {
    let port = allocate_tcp_port(address)?;

    let syn = PacketBuilder::ip(match address {
        SocketAddr::V4(addr) => {
            let mut ip = Ipv4Header::new(0, 64, 6, [0u8; 4], addr.ip().octets());
            ip.identification = rand::random();
            ip.differentiated_services_code_point = 0b010010;

            IpHeader::Version4(ip, Ipv4Extensions { auth: None })
        }
        SocketAddr::V6(addr) => IpHeader::Version6(
            Ipv6Header {
                traffic_class: 0,  // TODO: sane values?
                source: [0u8; 16], // filled in by OS
                destination: addr.ip().octets(),
                flow_label: 0,     // TODO: sane values?
                hop_limit: 0,      // TODO: sane values?
                next_header: 0,    // TODO: sane values?
                payload_length: 0, // filled in by OS
            },
            Ipv6Extensions {
                auth: None,
                destination_options: None,
                fragment: None,
                hop_by_hop_options: None,
                routing: None,
            },
        ),
    })
    .tcp(port, address.port(), 0x31337421, 0x7337)
    .syn()
    .options(&[
        TcpOptionElement::MaximumSegmentSize(1420),
        TcpOptionElement::SelectiveAcknowledgementPermitted,
        TcpOptionElement::Timestamp(0x12345678, 0), // timestamp is an opaque value that will just be echo'd
        TcpOptionElement::Noop,
        TcpOptionElement::WindowScale(7),
    ])
    .expect("the TCP options above should never reach 40 bytes!");

    let len = syn.size(0);
    let mut result = Vec::<u8>::with_capacity(len);
    syn.write(&mut result, &[])
        .expect("IP/TCP syn build failed?!");
    // TODO: timeout
    let (ip, tcp) = tcp_get_syn_ack(address, port, &result).await?;
    Ok(TcpFingerprint { ip, tcp })
}
