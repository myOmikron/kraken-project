use std::net::IpAddr;

use futures::StreamExt;
use ipnetwork::IpNetwork;
use log::{error, warn};
use rorm::prelude::*;
use rorm::{and, insert, query};
use uuid::Uuid;

use crate::chan::WsMessage;
use crate::models::{
    Host, HostInsert, OsType, Port, PortInsert, PortProtocol, TcpPortScanResultInsert,
};
use crate::modules::attacks::LeechAttackContext;
use crate::rpc::rpc_definitions::shared::address::Address;
use crate::rpc::rpc_definitions::TcpPortScanRequest;

impl LeechAttackContext {
    /// Start a tcp port scan
    ///
    /// See [`handler::attacks::scan_tcp_ports`] for more information.
    pub async fn tcp_port_scan(mut self, req: TcpPortScanRequest) {
        match self.leech.run_tcp_port_scan(req).await {
            Ok(v) => {
                let mut stream = v.into_inner();

                while let Some(res) = stream.next().await {
                    match res {
                        Ok(v) => {
                            let Some(addr) = v.address else {
                                warn!("Missing field address in grpc response of scan tcp ports");
                                continue;
                            };

                            let Some(addr) = addr.address else {
                                warn!("Missing field address.address in grpc response of scan tcp ports");
                                continue;
                            };

                            let address = match addr {
                                Address::Ipv4(addr) => IpAddr::V4(addr.into()),
                                Address::Ipv6(addr) => IpAddr::V6(addr.into()),
                            };

                            if let Err(err) = self
                                .insert_tcp_port_scan_result(
                                    IpNetwork::from(address),
                                    v.port as u16,
                                )
                                .await
                            {
                                error!("Database error: {err}");
                            }

                            self.send_ws(WsMessage::ScanTcpPortsResult {
                                attack_uuid: self.attack_uuid,
                                address: address.to_string(),
                                port: v.port as u16,
                            })
                            .await;
                        }
                        Err(err) => {
                            error!("Error while reading from stream: {err}");
                            self.set_finished(false).await;
                            return;
                        }
                    }
                }
            }
            Err(err) => {
                error!("Error while reading from stream: {err}");
                self.set_finished(false).await;
                return;
            }
        };

        self.set_finished(true).await;
    }

    /// Insert a tcp port scan's result and update the aggregation
    async fn insert_tcp_port_scan_result(
        &self,
        ip_addr: IpNetwork,
        port_num: u16,
    ) -> Result<(), rorm::Error> {
        insert!(&self.db, TcpPortScanResultInsert)
            .return_nothing()
            .single(&TcpPortScanResultInsert {
                uuid: Uuid::new_v4(),
                attack: ForeignModelByField::Key(self.attack_uuid),
                address: ip_addr,
                port: port_num as i32,
            })
            .await?;

        let mut tx = self.db.start_transaction().await?;
        let host = query!(&mut tx, (Host::F.uuid,))
            .condition(and![
                Host::F.ip_addr.equals(ip_addr),
                Host::F.workspace.equals(self.workspace_uuid)
            ])
            .optional()
            .await?;

        let host_uuid = if let Some((uuid,)) = host {
            uuid
        } else {
            insert!(&mut tx, HostInsert)
                .return_primary_key()
                .single(&HostInsert {
                    uuid: Uuid::new_v4(),
                    ip_addr,
                    os_type: OsType::Unknown,
                    response_time: None,
                    comment: String::new(),
                    workspace: ForeignModelByField::Key(self.workspace_uuid),
                })
                .await?
        };

        let port = query!(&mut tx, Port)
            .condition(and![
                Port::F
                    .port
                    .equals(i16::from_ne_bytes(port_num.to_ne_bytes())),
                Port::F.protocol.equals(PortProtocol::Tcp),
                Port::F.host.equals(host_uuid),
                Port::F.workspace.equals(self.workspace_uuid),
            ])
            .optional()
            .await?;
        if port.is_none() {
            insert!(&mut tx, PortInsert)
                .return_nothing()
                .single(&PortInsert {
                    uuid: Uuid::new_v4(),
                    port: i16::from_ne_bytes(port_num.to_ne_bytes()),
                    protocol: PortProtocol::Tcp,
                    host: ForeignModelByField::Key(host_uuid),
                    comment: String::new(),
                    workspace: ForeignModelByField::Key(self.workspace_uuid),
                })
                .await?;
        }
        tx.commit().await?;

        Ok(())
    }
}
