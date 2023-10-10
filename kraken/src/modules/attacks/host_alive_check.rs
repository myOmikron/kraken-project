use futures::StreamExt;
use ipnetwork::IpNetwork;
use log::{error, warn};
use rorm::prelude::*;
use rorm::{and, insert, query};
use uuid::Uuid;

use crate::chan::WsMessage;
use crate::models::{Host, HostAliveResultInsert, HostInsert, OsType};
use crate::modules::attacks::LeechAttackContext;
use crate::rpc::rpc_definitions::HostsAliveRequest;

impl LeechAttackContext {
    /// Check if hosts are reachable
    ///
    /// See [`handler::attacks::hosts_alive_check`] for more information.
    pub async fn host_alive_check(mut self, req: HostsAliveRequest) {
        match self.leech.hosts_alive_check(req).await {
            Ok(v) => {
                let mut stream = v.into_inner();

                while let Some(res) = stream.next().await {
                    match res {
                        Ok(v) => {
                            let Some(host) = v.host else {
                                warn!(
                                    "Missing field `host` in grpc response of bruteforce subdomains"
                                );
                                continue;
                            };

                            let host = host.into();
                            self.send_ws(WsMessage::HostsAliveCheck {
                                host,
                                attack_uuid: self.attack_uuid,
                            })
                            .await;
                            if let Err(err) = self.insert_host_alive_check_result(host.into()).await
                            {
                                error!(
                                    "Failed to insert query certificate transparency result: {err}"
                                );
                            }
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

    /// Insert a host alive's result and update the aggregation
    async fn insert_host_alive_check_result(&self, host: IpNetwork) -> Result<(), rorm::Error> {
        let mut tx = self.db.start_transaction().await?;

        insert!(&mut tx, HostAliveResultInsert)
            .return_nothing()
            .single(&HostAliveResultInsert {
                uuid: Uuid::new_v4(),
                attack: ForeignModelByField::Key(self.attack_uuid),
                host,
            })
            .await?;

        if let Some((_host_uuid,)) = query!(&mut tx, (Host::F.uuid,))
            .condition(and!(
                Host::F.ip_addr.equals(host),
                Host::F.workspace.equals(self.workspace_uuid)
            ))
            .optional()
            .await?
        {
            // TODO update reachable
        } else {
            insert!(&mut tx, HostInsert)
                .return_nothing()
                .single(&HostInsert {
                    uuid: Uuid::new_v4(),
                    ip_addr: host,
                    os_type: OsType::Unknown,
                    response_time: None,
                    comment: String::new(),
                    workspace: ForeignModelByField::Key(self.workspace_uuid),
                })
                .await?;
        }

        tx.commit().await
    }
}
