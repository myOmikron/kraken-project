use std::collections::HashSet;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::time::Duration;

use etherparse::packet_filter::ElementFilter;
use etherparse::InternetSlice;
use etherparse::IpHeader;
use etherparse::Ipv4Extensions;
use etherparse::Ipv4Header;
use etherparse::Ipv6ExtensionSlice;
use etherparse::Ipv6Extensions;
use etherparse::Ipv6ExtensionsSlice;
use etherparse::Ipv6Header;
use etherparse::Ipv6RoutingExtensions;
use etherparse::PacketBuilder;
use etherparse::SlicedPacket;
use etherparse::TcpHeader;
use etherparse::TcpOptionElement;
use etherparse::TransportSlice;
use futures::stream;
use futures::StreamExt;
use futures::TryFutureExt;
use log::trace;
use rand::seq::SliceRandom;
use rand::thread_rng;
use socket2::Domain;
use socket2::Protocol;
use tokio::net::UdpSocket;

use crate::modules::os_detection::errors::RawTcpError;
use crate::utils::allocate_tcp_port;
use crate::utils::find_source_ip;
use crate::utils::raw_socket;

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

    ret
}

/// Sends out the given raw SYN packet and awaits an incoming matching SYN ACK response
pub async fn tcp_get_syn_ack(
    socket: UdpSocket,
    address: SocketAddr,
    recv_port: u16,
    syn: &[u8],
) -> Result<(IpHeader, TcpHeader), RawTcpError> {
    let recv_tcp_filter = etherparse::packet_filter::TransportFilter::Tcp {
        source_port: Some(address.port()),
        destination_port: Some(recv_port),
    };
    let recv_ip_filter = match address.ip() {
        IpAddr::V4(v4) => etherparse::packet_filter::IpFilter::Ipv4 {
            source: Some(v4.octets()),
            destination: None,
        },
        IpAddr::V6(v6) => etherparse::packet_filter::IpFilter::Ipv6 {
            source: Some(v6.octets()),
            destination: None,
        },
    };
    let recv_filter = etherparse::packet_filter::Filter {
        ip: ElementFilter::Some(recv_ip_filter),
        transport: ElementFilter::Some(recv_tcp_filter),
        link: ElementFilter::Any,
        vlan: ElementFilter::Any,
    };

    socket.send_to(syn, address).await?;

    let mut buf = [0u8; 256];

    loop {
        let (len, _) = socket.recv_from(&mut buf).await?;

        let packet = SlicedPacket::from_ip(&buf[0..len])?;
        if !recv_filter.applies_to_slice(&packet) {
            continue;
        }

        if let Some(transport_slice) = packet.transport {
            return Ok(match transport_slice {
                TransportSlice::Tcp(tcp) => {
                    if !(tcp.syn() && tcp.ack()) {
                        continue;
                    }

                    (match packet.ip.expect("must have ip since we decoded from ip") {
                        InternetSlice::Ipv4(ip, ext) => IpHeader::Version4(
                            ip.to_header(),
                            ext.to_header()
                        ),
                        InternetSlice::Ipv6(ip, ext) => IpHeader::Version6(
                            ip.to_header(),
                            make_ipv6_headers(&ext)
                        )
                    }, tcp.to_header())
                },
                _ => panic!("should never reach this since the TransportFilter::Tcp (recv_filter) matched on this")
            });
        }
    }
}

/// Tests if the given IP & TCP port responds to a TCP SYN packet (start of connection handshake) with SYN ACK
pub async fn check_syn_ack(address: SocketAddr, timeout: Duration) -> Result<bool, RawTcpError> {
    let socket = raw_socket(Domain::for_address(address), Protocol::TCP)?;

    let source_ip = find_source_ip(address.ip())?;

    let port = allocate_tcp_port(address)?;

    trace!(
        "Sending TCP SYN from {source_ip:?}:{} to {address:?}",
        port.port()
    );

    let syn = PacketBuilder::ip(match (address, source_ip) {
        (SocketAddr::V4(addr), IpAddr::V4(local_addr)) => {
            let mut ip = Ipv4Header::new(0, 42, 6, local_addr.octets(), addr.ip().octets());
            ip.identification = rand::random();

            IpHeader::Version4(ip, Ipv4Extensions { auth: None })
        }
        (SocketAddr::V6(addr), IpAddr::V6(local_addr)) => IpHeader::Version6(
            Ipv6Header {
                traffic_class: 0, // TODO: sane values?
                source: local_addr.octets(),
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
        (_, _) => return Err(RawTcpError::InvalidLocalAddrDomain),
    })
    .tcp(port.port(), address.port(), 0x31337421, 31337)
    .syn()
    .options(&[
        TcpOptionElement::MaximumSegmentSize(1337),
        TcpOptionElement::SelectiveAcknowledgementPermitted,
        TcpOptionElement::Timestamp(0xf0031337, 0), // timestamp is an opaque value that will just be echo'd
        TcpOptionElement::Noop,
        TcpOptionElement::WindowScale(7),
    ])
    .expect("the TCP options above should never reach 40 bytes!");

    let len = syn.size(0);
    let mut result = Vec::<u8>::with_capacity(len);
    syn.write(&mut result, &[])
        .expect("IP/TCP syn build failed?!");

    let Ok(v) = tokio::time::timeout(
        timeout,
        tcp_get_syn_ack(socket, address, port.port(), &result),
    )
    .await
    else {
        // timeout
        return Ok(false);
    };

    let v = v?;
    Ok(v.1.syn && v.1.ack)
}

/// Looks at random ports and returns the first detected open and closed (no response) TCP ports
pub async fn find_open_and_closed_port(
    ip_addr: IpAddr,
    each_timeout: Duration,
    max_parallel: usize,
) -> Result<(u16, u16), RawTcpError> {
    let mut opened = 0u16;
    let mut closed = 0u16;

    let common_ports = HashSet::from([
        21, 22, 23, 25, 80, 123, 137, 138, 139, 143, 220, 389, 443, 993, 3306, 3389, 5357, 8080,
    ]);

    trace!("Looking for opened and closed port on {ip_addr} (timeout={each_timeout:?}, parallel={max_parallel})");

    let mut remaining_ports: Vec<_> = (1u16..=65535u16)
        .filter(|p| !common_ports.contains(p))
        .collect();
    remaining_ports.shuffle(&mut thread_rng());

    let mut all = stream::iter(common_ports.iter().copied().chain(remaining_ports))
        .map(|port| SocketAddr::new(ip_addr, port))
        .map(|host| check_syn_ack(host, each_timeout).map_ok(move |d| (host.port(), d)))
        .buffer_unordered(max_parallel);

    while let Some(result) = all.next().await {
        let result = result?;
        if result.1 && opened == 0 {
            opened = result.0;
        } else if !result.1 && closed == 0 {
            closed = result.0;
        } else {
            continue;
        }

        if opened != 0 && closed != 0 {
            trace!("Found opened port {opened} and closed port {closed} on {ip_addr}");
            return Ok((opened, closed));
        }
    }

    Err(RawTcpError::NoPortsFound)
}
