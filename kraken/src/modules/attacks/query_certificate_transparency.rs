use chrono::{NaiveDateTime, TimeZone, Utc};
use rorm::insert;
use rorm::prelude::*;
use uuid::Uuid;

use crate::chan::{CertificateTransparencyEntry, WsMessage};
use crate::models::{
    CertificateTransparencyResultInsert, CertificateTransparencyValueNameInsert, Domain,
};
use crate::modules::attacks::{AttackError, LeechAttackContext};
use crate::rpc::rpc_definitions::{
    CertificateTransparencyRequest, CertificateTransparencyResponse,
};

impl LeechAttackContext {
    /// Query a certificate transparency log collector.
    ///
    /// See [`handler::attacks::query_certificate_transparency`] for more information.
    pub async fn query_certificate_transparency(mut self, req: CertificateTransparencyRequest) {
        match self.leech.query_certificate_transparency(req).await {
            Ok(res) => {
                let res = res.into_inner();

                self.set_finished(
                    self.insert_query_certificate_transparency_result(&res)
                        .await
                        .map_err(AttackError::from)
                        .err(),
                )
                .await;

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

    /// Insert a query certificate transparency's result and update the aggregation
    async fn insert_query_certificate_transparency_result(
        &self,
        res: &CertificateTransparencyResponse,
    ) -> Result<(), rorm::Error> {
        let mut tx = self.db.start_transaction().await?;

        for cert_entry in &res.entries {
            let cert_uuid = insert!(&mut tx, CertificateTransparencyResultInsert)
                .return_primary_key()
                .single(&CertificateTransparencyResultInsert {
                    uuid: Uuid::new_v4(),
                    attack: ForeignModelByField::Key(self.attack_uuid),
                    issuer_name: cert_entry.issuer_name.clone(),
                    serial_number: cert_entry.serial_number.clone(),
                    common_name: cert_entry.common_name.clone(),
                    not_before: cert_entry.not_before.clone().map(|x| {
                        Utc.from_utc_datetime(
                            &NaiveDateTime::from_timestamp_millis(x.seconds * 1000).unwrap(),
                        )
                    }),
                    not_after: cert_entry.not_after.clone().map(|x| {
                        Utc.from_utc_datetime(
                            &NaiveDateTime::from_timestamp_millis(x.seconds * 1000).unwrap(),
                        )
                    }),
                })
                .await?;

            let value_names = cert_entry.value_names.clone().into_iter().map(|x| {
                CertificateTransparencyValueNameInsert {
                    uuid: Uuid::new_v4(),
                    value_name: x,
                    ct_result: ForeignModelByField::Key(cert_uuid),
                }
            });

            insert!(&mut tx, CertificateTransparencyValueNameInsert)
                .return_nothing()
                .bulk(value_names)
                .await?;
        }

        for entry in &res.entries {
            Domain::insert_if_missing(&mut tx, self.workspace_uuid, &entry.common_name).await?;
            for value in &entry.value_names {
                Domain::insert_if_missing(&mut tx, self.workspace_uuid, value).await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }
}
