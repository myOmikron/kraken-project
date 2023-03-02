pub mod rpc_attacks {
    use std::net::{Ipv4Addr, Ipv6Addr};

    pub mod shared {
        tonic::include_proto!("attacks.shared");
    }

    tonic::include_proto!("attacks");

    impl From<shared::Ipv4> for Ipv4Addr {
        fn from(value: shared::Ipv4) -> Self {
            Ipv4Addr::from(value.address.to_le_bytes())
        }
    }

    impl From<shared::Ipv6> for Ipv6Addr {
        fn from(value: shared::Ipv6) -> Self {
            let [a, b, c, d, e, f, g, h] = value.part0.to_le_bytes();
            let [i, j, k, l, m, n, o, p] = value.part1.to_le_bytes();
            Ipv6Addr::from([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
        }
    }
}
