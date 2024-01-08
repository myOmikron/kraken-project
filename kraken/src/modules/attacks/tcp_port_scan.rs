use std::net::IpAddr;

use ipnetwork::IpNetwork;

use crate::api::handler::attacks::schema::PortOrRange;
use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::WsMessage;
use crate::modules::attack_results::store_tcp_port_scan_result;
use crate::modules::attacks::{
    AttackContext, AttackError, DomainOrNetwork, HandleAttackResponse, TcpPortScanParams,
};
use crate::rpc::rpc_definitions;
use crate::rpc::rpc_definitions::shared::address::Address;
use crate::rpc::rpc_definitions::{shared, TcpPortScanRequest, TcpPortScanResponse};

impl AttackContext {
    /// Executes the "tcp port scan" attack
    pub async fn tcp_port_scan(
        &self,
        mut leech: LeechClient,
        params: TcpPortScanParams,
    ) -> Result<(), AttackError> {
        let targets =
            DomainOrNetwork::resolve(self.workspace.uuid, self.user.uuid, &leech, &params.targets)
                .await?;
        let request = TcpPortScanRequest {
            attack_uuid: self.attack_uuid.to_string(),
            targets: targets.into_iter().map(From::from).collect(),
            ports: params
                .ports
                .into_iter()
                .map(|x| rpc_definitions::PortOrRange {
                    port_or_range: Some(match x {
                        PortOrRange::Port(port) => {
                            rpc_definitions::port_or_range::PortOrRange::Single(port as u32)
                        }
                        PortOrRange::Range(range) => {
                            rpc_definitions::port_or_range::PortOrRange::Range(
                                rpc_definitions::PortRange {
                                    start: *range.start() as u32,
                                    end: *range.end() as u32,
                                },
                            )
                        }
                    }),
                })
                .collect(),
            timeout: params.timeout,
            concurrent_limit: params.concurrent_limit,
            max_retries: params.max_retries,
            retry_interval: params.retry_interval,
            skip_icmp_check: params.skip_icmp_check,
        };
        self.handle_streamed_response(leech.run_tcp_port_scan(request))
            .await
    }
}
impl HandleAttackResponse<TcpPortScanResponse> for AttackContext {
    async fn handle_response(&self, response: TcpPortScanResponse) -> Result<(), AttackError> {
        let TcpPortScanResponse {
            address: Some(shared::Address {
                address: Some(addr),
            }),
            port,
        } = response
        else {
            return Err(AttackError::Malformed("Missing `address`"));
        };

        let address = match addr {
            Address::Ipv4(addr) => IpAddr::V4(addr.into()),
            Address::Ipv6(addr) => IpAddr::V6(addr.into()),
        };
        let port = port as u16;

        self.send_ws(WsMessage::ScanTcpPortsResult {
            attack_uuid: self.attack_uuid,
            address: address.to_string(),
            port,
        })
        .await;

        store_tcp_port_scan_result(
            &GLOBAL.db,
            self.attack_uuid,
            self.workspace.uuid,
            IpNetwork::from(address),
            port,
        )
        .await?;

        Ok(())
    }
}
