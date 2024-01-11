use crate::api::handler::attacks::schema::PortOrRange;

impl From<PortOrRange> for kraken_proto::PortOrRange {
    fn from(value: PortOrRange) -> Self {
        return kraken_proto::PortOrRange {
            port_or_range: Some(match value {
                PortOrRange::Port(port) => {
                    kraken_proto::port_or_range::PortOrRange::Single(port as u32)
                }
                PortOrRange::Range(range) => {
                    kraken_proto::port_or_range::PortOrRange::Range(kraken_proto::PortRange {
                        start: *range.start() as u32,
                        end: *range.end() as u32,
                    })
                }
            }),
        };
    }
}
