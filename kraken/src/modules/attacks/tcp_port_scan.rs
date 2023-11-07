use std::net::IpAddr;

use ipnetwork::IpNetwork;

use crate::chan::WsMessage;
use crate::modules::attack_results::store_tcp_port_scan_result;
use crate::modules::attacks::{AttackContext, AttackError, LeechAttackContext};
use crate::rpc::rpc_definitions::shared::address::Address;
use crate::rpc::rpc_definitions::{shared, TcpPortScanRequest, TcpPortScanResponse};

impl LeechAttackContext {
    /// Start a tcp port scan
    ///
    /// See [`handler::attacks::scan_tcp_ports`] for more information.
    pub async fn tcp_port_scan(mut self, req: TcpPortScanRequest) {
        let result = AttackContext::handle_streamed_response(
            self.leech.run_tcp_port_scan(req.clone()).await,
            |response| async {
                let TcpPortScanResponse {
                    address:
                        Some(shared::Address {
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
                    &self.db,
                    self.attack_uuid,
                    self.workspace_uuid,
                    IpNetwork::from(address),
                    port,
                )
                .await?;

                Ok(())
            },
        )
        .await;
        self.set_finished(result.err()).await;
    }
}
