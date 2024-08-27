use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::ops::RangeInclusive;

use ipnetwork::IpNetwork;
use ipnetwork::Ipv4Network;
use ipnetwork::Ipv6Network;
use ipnetwork::NetworkSize;
use thiserror::Error;
use tonic::Status;

use crate::port_or_range;
use crate::shared::address;
use crate::shared::net;
use crate::shared::net_or_address;
use crate::shared::Address;
use crate::shared::Ipv4;
use crate::shared::Ipv4Net;
use crate::shared::Ipv6;
use crate::shared::Ipv6Net;
use crate::shared::Net;
use crate::shared::NetOrAddress;
use crate::PortOrRange;
use crate::PortRange;

/// Error while converting a protobuf message to its rust type
///
/// It indicates a problem with the message sender and should be reported to him as [`Status::invalid_argument`].
#[derive(Debug, Error, Copy, Clone)]
pub enum InvalidArgumentError {
    /// A `oneof` field was empty
    #[error("The oneof field `{}` of message `{}` was empty", .field_name, .message_name)]
    EmptyOneOfError {
        /// The message's (i.e. rust struct's) name
        message_name: &'static str,
        /// The field's name
        field_name: &'static str,
    },

    /// A required submessage was missing
    #[error("The submessage `{}` of message `{}` was missing", .field_name, .message_name)]
    MissingSubmessage {
        /// The message's (i.e. rust struct's) name
        message_name: &'static str,
        /// The field's name
        field_name: &'static str,
    },

    /// Received an invalid value for a port number
    #[error("Got invalid port number: {}", .0)]
    InvalidPort(u32),

    /// Received an invalid value for a ipv4 network prefix
    #[error("Got invalid network prefix for v4: {}", .0)]
    InvalidV4Prefix(u32),

    /// Received an invalid value for a ipv4 network prefix
    #[error("Got invalid network prefix for v6: {}", .0)]
    InvalidV6Prefix(u32),
}
impl From<InvalidArgumentError> for Status {
    fn from(value: InvalidArgumentError) -> Self {
        Status::invalid_argument(value.to_string())
    }
}

impl TryFrom<Address> for IpAddr {
    type Error = InvalidArgumentError;
    fn try_from(value: Address) -> Result<Self, Self::Error> {
        match value.address {
            Some(address::Address::Ipv4(ip)) => Ok(IpAddr::V4(Ipv4Addr::from(ip))),
            Some(address::Address::Ipv6(ip)) => Ok(IpAddr::V6(Ipv6Addr::from(ip))),
            None => Err(InvalidArgumentError::EmptyOneOfError {
                message_name: "Address",
                field_name: "address",
            }),
        }
    }
}
impl From<IpAddr> for Address {
    fn from(value: IpAddr) -> Self {
        Self {
            address: Some(match value {
                IpAddr::V4(ip) => address::Address::Ipv4(Ipv4::from(ip)),
                IpAddr::V6(ip) => address::Address::Ipv6(Ipv6::from(ip)),
            }),
        }
    }
}

impl From<Ipv4> for Ipv4Addr {
    fn from(value: Ipv4) -> Self {
        Ipv4Addr::from(value.address.to_le_bytes())
    }
}
impl From<Ipv4Addr> for Ipv4 {
    fn from(value: Ipv4Addr) -> Self {
        Self {
            address: i32::from_le_bytes(value.octets()),
        }
    }
}

impl From<Ipv6> for Ipv6Addr {
    fn from(value: Ipv6) -> Self {
        let [a, b, c, d, e, f, g, h] = value.part0.to_le_bytes();
        let [i, j, k, l, m, n, o, p] = value.part1.to_le_bytes();
        Ipv6Addr::from([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
    }
}
impl From<Ipv6Addr> for Ipv6 {
    fn from(value: Ipv6Addr) -> Self {
        let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = value.octets();
        Self {
            part0: i64::from_le_bytes([a, b, c, d, e, f, g, h]),
            part1: i64::from_le_bytes([i, j, k, l, m, n, o, p]),
        }
    }
}

impl TryFrom<NetOrAddress> for IpNetwork {
    type Error = InvalidArgumentError;
    fn try_from(value: NetOrAddress) -> Result<Self, Self::Error> {
        let net_or_address = value
            .net_or_address
            .ok_or(InvalidArgumentError::EmptyOneOfError {
                message_name: "NetOrAddress",
                field_name: "net_or_address",
            })?;

        match net_or_address {
            net_or_address::NetOrAddress::Address(address) => {
                Ok(IpNetwork::from(IpAddr::try_from(address)?))
            }
            net_or_address::NetOrAddress::Net(network) => IpNetwork::try_from(network),
        }
    }
}
impl From<IpNetwork> for NetOrAddress {
    fn from(value: IpNetwork) -> Self {
        Self {
            net_or_address: Some(match value.size() {
                NetworkSize::V4(0) | NetworkSize::V6(0) => {
                    net_or_address::NetOrAddress::Address(Address::from(value.ip()))
                }
                _ => net_or_address::NetOrAddress::Net(Net::from(value)),
            }),
        }
    }
}

impl TryFrom<Net> for IpNetwork {
    type Error = InvalidArgumentError;
    fn try_from(value: Net) -> Result<Self, Self::Error> {
        let net = value.net.ok_or(InvalidArgumentError::EmptyOneOfError {
            message_name: "Net",
            field_name: "net",
        })?;

        match net {
            net::Net::Ipv4net(network) => Ok(IpNetwork::V4(Ipv4Network::try_from(network)?)),
            net::Net::Ipv6net(network) => Ok(IpNetwork::V6(Ipv6Network::try_from(network)?)),
        }
    }
}
impl From<IpNetwork> for Net {
    fn from(value: IpNetwork) -> Self {
        Self {
            net: Some(match value {
                IpNetwork::V4(network) => net::Net::Ipv4net(Ipv4Net::from(network)),
                IpNetwork::V6(network) => net::Net::Ipv6net(Ipv6Net::from(network)),
            }),
        }
    }
}

impl TryFrom<Ipv4Net> for Ipv4Network {
    type Error = InvalidArgumentError;
    fn try_from(value: Ipv4Net) -> Result<Self, Self::Error> {
        let invalid_prefix = InvalidArgumentError::InvalidV4Prefix(value.prefix);
        let address = value
            .address
            .ok_or(InvalidArgumentError::MissingSubmessage {
                message_name: "Ipv4Net",
                field_name: "address",
            })?;
        let prefix = u8::try_from(value.prefix).map_err(|_| invalid_prefix)?;
        Ipv4Network::new(Ipv4Addr::from(address), prefix).map_err(|_| invalid_prefix)
    }
}
impl From<Ipv4Network> for Ipv4Net {
    fn from(value: Ipv4Network) -> Self {
        Self {
            address: Some(Ipv4::from(value.ip())),
            prefix: value.prefix() as u32,
        }
    }
}

impl TryFrom<Ipv6Net> for Ipv6Network {
    type Error = InvalidArgumentError;
    fn try_from(value: Ipv6Net) -> Result<Self, Self::Error> {
        let invalid_prefix = InvalidArgumentError::InvalidV6Prefix(value.prefix);
        let address = value
            .address
            .ok_or(InvalidArgumentError::MissingSubmessage {
                message_name: "Ipv6Net",
                field_name: "address",
            })?;
        let prefix = u8::try_from(value.prefix).map_err(|_| invalid_prefix)?;
        Ipv6Network::new(Ipv6Addr::from(address), prefix).map_err(|_| invalid_prefix)
    }
}
impl From<Ipv6Network> for Ipv6Net {
    fn from(value: Ipv6Network) -> Self {
        Self {
            address: Some(Ipv6::from(value.ip())),
            prefix: value.prefix() as u32,
        }
    }
}

impl TryFrom<PortOrRange> for RangeInclusive<u16> {
    type Error = InvalidArgumentError;
    fn try_from(value: PortOrRange) -> Result<Self, Self::Error> {
        let port_or_range = value
            .port_or_range
            .ok_or(InvalidArgumentError::EmptyOneOfError {
                message_name: "PortOrRange",
                field_name: "port_or_range",
            })?;
        match port_or_range {
            port_or_range::PortOrRange::Single(port) => {
                let port = convert_port(port)?;
                Ok(RangeInclusive::new(port, port))
            }
            port_or_range::PortOrRange::Range(range) => Self::try_from(range),
        }
    }
}
impl From<RangeInclusive<u16>> for PortOrRange {
    fn from(value: RangeInclusive<u16>) -> Self {
        Self {
            port_or_range: Some(if *value.start() == *value.end() {
                port_or_range::PortOrRange::Single(*value.start() as u32)
            } else {
                port_or_range::PortOrRange::Range(PortRange::from(value))
            }),
        }
    }
}

impl TryFrom<PortRange> for RangeInclusive<u16> {
    type Error = InvalidArgumentError;
    fn try_from(value: PortRange) -> Result<Self, Self::Error> {
        Ok(RangeInclusive::new(
            convert_port(value.start)?,
            convert_port(value.end)?,
        ))
    }
}
impl From<RangeInclusive<u16>> for PortRange {
    fn from(value: RangeInclusive<u16>) -> Self {
        Self {
            start: *value.start() as u32,
            end: *value.end() as u32,
        }
    }
}

fn convert_port(port: u32) -> Result<u16, InvalidArgumentError> {
    let error = InvalidArgumentError::InvalidPort(port);

    let Ok(port) = u16::try_from(port) else {
        return Err(error);
    };

    if port == 0 {
        Err(error)
    } else {
        Ok(port)
    }
}
