use chrono::{NaiveDateTime, TimeZone, Utc};

use crate::chan::{CertificateTransparencyEntry, WsMessage, GLOBAL};
use crate::modules::attack_results::store_query_certificate_transparency_result;
use crate::modules::attacks::{AttackError, LeechAttackContext};
use crate::rpc::rpc_definitions::CertificateTransparencyRequest;

impl LeechAttackContext {
    /// Query a certificate transparency log collector.
    ///
    /// See [`handler::attacks::query_certificate_transparency`] for more information.
    pub async fn query_certificate_transparency(mut self, req: CertificateTransparencyRequest) {
        match self.leech.query_certificate_transparency(req).await {
            Ok(res) => {
                let res = res.into_inner();

                for entry in &res.entries {
                    if let Err(error) = store_query_certificate_transparency_result(
                        &GLOBAL.db,
                        self.attack_uuid,
                        self.workspace_uuid,
                        entry.clone(),
                    )
                    .await
                    {
                        self.set_finished(Some(error.into())).await;
                        return;
                    }
                }
                self.set_finished(None).await;

                self.send_ws(WsMessage::CertificateTransparencyResult {
                    attack_uuid: self.attack_uuid,
                    entries: res
                        .entries
                        .into_iter()
                        .map(|e| CertificateTransparencyEntry {
                            serial_number: e.serial_number,
                            issuer_name: e.issuer_name,
                            common_name: e.common_name,
                            value_names: e.value_names,
                            not_before: e.not_before.map(|ts| {
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32)
                                        .unwrap(),
                                )
                            }),
                            not_after: e.not_after.map(|ts| {
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32)
                                        .unwrap(),
                                )
                            }),
                        })
                        .collect(),
                })
                .await;
            }
            Err(status) => {
                self.set_finished(Some(AttackError::Grpc(status))).await;
            }
        }
    }
}
