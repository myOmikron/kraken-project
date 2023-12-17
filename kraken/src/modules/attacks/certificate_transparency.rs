use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::{CertificateTransparencyEntry, WsMessage};
use crate::modules::attack_results::store_query_certificate_transparency_result;
use crate::modules::attacks::{AttackContext, AttackError, CertificateTransparencyParams};
use crate::modules::utc::utc_from_seconds;
use crate::rpc::rpc_definitions::CertificateTransparencyRequest;

impl AttackContext {
    /// Executes the "certificate transparency" attack
    pub async fn certificate_transparency(
        &self,
        mut leech: LeechClient,
        params: CertificateTransparencyParams,
    ) -> Result<(), AttackError> {
        let request = CertificateTransparencyRequest {
            attack_uuid: self.attack_uuid.to_string(),
            target: params.target,
            include_expired: params.include_expired,
            max_retries: params.max_retries,
            retry_interval: params.retry_interval,
        };
        let response = leech
            .query_certificate_transparency(request)
            .await?
            .into_inner();

        for entry in &response.entries {
            store_query_certificate_transparency_result(
                &GLOBAL.db,
                self.attack_uuid,
                self.workspace.uuid,
                entry.clone(),
            )
            .await?;
        }
        self.send_ws(WsMessage::CertificateTransparencyResult {
            attack_uuid: self.attack_uuid,
            entries: response
                .entries
                .into_iter()
                .map(|e| CertificateTransparencyEntry {
                    serial_number: e.serial_number,
                    issuer_name: e.issuer_name,
                    common_name: e.common_name,
                    value_names: e.value_names,
                    not_before: e.not_before.map(|ts| utc_from_seconds(ts.seconds)),
                    not_after: e.not_after.map(|ts| utc_from_seconds(ts.seconds)),
                })
                .collect(),
        })
        .await;
        Ok(())
    }
}
