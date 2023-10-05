pub mod rpc_definitions {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    pub mod shared {
        tonic::include_proto!("attacks.shared");
    }

    tonic::include_proto!("attacks");

    impl From<shared::Ipv4> for Ipv4Addr {
        fn from(value: shared::Ipv4) -> Self {
            Ipv4Addr::from(value.address.to_le_bytes())
        }
    }

    impl From<Ipv4Addr> for shared::Ipv4 {
        fn from(value: Ipv4Addr) -> Self {
            shared::Ipv4 {
                address: i32::from_le_bytes(value.octets()),
            }
        }
    }

    impl From<shared::Ipv6> for Ipv6Addr {
        fn from(value: shared::Ipv6) -> Self {
            let [a, b, c, d, e, f, g, h] = value.part0.to_le_bytes();
            let [i, j, k, l, m, n, o, p] = value.part1.to_le_bytes();
            Ipv6Addr::from([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
        }
    }

    impl From<Ipv6Addr> for shared::Ipv6 {
        fn from(value: Ipv6Addr) -> Self {
            let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = value.octets();
            shared::Ipv6 {
                part0: i64::from_le_bytes([a, b, c, d, e, f, g, h]),
                part1: i64::from_le_bytes([i, j, k, l, m, n, o, p]),
            }
        }
    }

    impl From<IpAddr> for shared::Address {
        fn from(value: IpAddr) -> Self {
            Self {
                address: Some(match value {
                    IpAddr::V4(addr) => shared::address::Address::Ipv4(addr.into()),
                    IpAddr::V6(addr) => shared::address::Address::Ipv6(addr.into()),
                }),
            }
        }
    }

    impl From<shared::Address> for IpAddr {
        fn from(value: shared::Address) -> Self {
            let shared::Address { address } = value;
            match address.unwrap() {
                shared::address::Address::Ipv4(v) => IpAddr::from(Ipv4Addr::from(v)),
                shared::address::Address::Ipv6(v) => IpAddr::from(Ipv6Addr::from(v)),
            }
        }
    }
}
