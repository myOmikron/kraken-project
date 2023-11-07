use crate::chan::WsMessage;
use crate::modules::attack_results::store_host_alive_check_result;
use crate::modules::attacks::{AttackContext, AttackError, LeechAttackContext};
use crate::rpc::rpc_definitions::{HostsAliveRequest, HostsAliveResponse};

impl LeechAttackContext {
    /// Check if hosts are reachable
    ///
    /// See [`handler::attacks::hosts_alive_check`] for more information.
    pub async fn host_alive_check(mut self, req: HostsAliveRequest) {
        let result = AttackContext::handle_streamed_response(
            self.leech.hosts_alive_check(req).await,
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
                    &self.db,
                    self.attack_uuid,
                    self.workspace_uuid,
                    host.into(),
                )
                .await?;

                Ok(())
            },
        )
        .await;
        self.set_finished(result.err()).await;
    }
}
