use crate::chan::{LeechClient, WsMessage, GLOBAL};
use crate::modules::attack_results::store_host_alive_check_result;
use crate::modules::attacks::{AttackContext, AttackError, DomainOrNetwork, HostAliveParams};
use crate::rpc::rpc_definitions::{HostsAliveRequest, HostsAliveResponse};

impl AttackContext {
    /// Executes the "host alive" attack
    pub async fn host_alive(
        &self,
        mut leech: LeechClient,
        params: HostAliveParams,
    ) -> Result<(), AttackError> {
        let targets =
            DomainOrNetwork::resolve(self.workspace_uuid, self.user_uuid, &leech, &params.targets)
                .await?;
        let request = HostsAliveRequest {
            attack_uuid: self.attack_uuid.to_string(),
            targets: targets.into_iter().map(From::from).collect(),
            timeout: params.timeout,
            concurrent_limit: params.concurrent_limit,
        };
        AttackContext::handle_streamed_response(
            leech.hosts_alive_check(request).await,
            |response| async {
                let HostsAliveResponse { host: Some(host) } = response else {
                    return Err(AttackError::Malformed("Missing `host`"));
                };

                let host = host.try_into()?;
                self.send_ws(WsMessage::HostsAliveCheck {
                    host,
                    attack_uuid: self.attack_uuid,
                })
                .await;

                store_host_alive_check_result(
                    &GLOBAL.db,
                    self.attack_uuid,
                    self.workspace_uuid,
                    host.into(),
                )
                .await?;

                Ok(())
            },
        )
        .await
    }
}
