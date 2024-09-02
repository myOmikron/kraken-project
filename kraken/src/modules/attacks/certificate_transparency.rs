use kraken_proto::CertificateTransparencyRequest;
use kraken_proto::CertificateTransparencyResponse;
use rorm::insert;
use rorm::prelude::*;
use rorm::query;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::CertificateTransparencyEntry;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::Attack;
use crate::models::CertificateTransparencyResultInsert;
use crate::models::CertificateTransparencyValueNameInsert;
use crate::models::DomainCertainty;
use crate::models::SourceType;
use crate::modules::attacks::AttackContext;
use crate::modules::attacks::AttackError;
use crate::modules::attacks::CertificateTransparencyParams;
use crate::modules::attacks::HandleAttackResponse;
use crate::modules::utc::utc_from_seconds;

impl AttackContext {
    /// Executes the "certificate transparency" attack
    pub async fn certificate_transparency(
        &mut self,
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
        self.handle_response(
            leech
                .query_certificate_transparency(request)
                .await?
                .into_inner(),
        )
        .await
    }
}
impl HandleAttackResponse<CertificateTransparencyResponse> for AttackContext {
    async fn handle_response(
        &mut self,
        response: CertificateTransparencyResponse,
    ) -> Result<(), AttackError> {
        for entry in response.entries.clone() {
            let mut tx = GLOBAL.db.start_transaction().await?;

            let user_uuid = *query!(&mut tx, (Attack::F.started_by,))
                .condition(Attack::F.uuid.equals(self.attack_uuid))
                .one()
                .await?
                .0
                .key();

            let result_uuid = insert!(&mut tx, CertificateTransparencyResultInsert)
                .return_primary_key()
                .single(&CertificateTransparencyResultInsert {
                    uuid: Uuid::new_v4(),
                    attack: ForeignModelByField::Key(self.attack_uuid),
                    issuer_name: entry.issuer_name,
                    serial_number: entry.serial_number,
                    common_name: entry.common_name.clone(),
                    not_before: entry.not_before.map(|x| utc_from_seconds(x.seconds)),
                    not_after: entry.not_after.map(|x| utc_from_seconds(x.seconds)),
                })
                .await?;

            insert!(&mut tx, CertificateTransparencyValueNameInsert)
                .return_nothing()
                .bulk(
                    entry
                        .value_names
                        .iter()
                        .map(|x| CertificateTransparencyValueNameInsert {
                            uuid: Uuid::new_v4(),
                            value_name: x.to_string(),
                            ct_result: ForeignModelByField::Key(result_uuid),
                        }),
                )
                .await?;

            let mut domains = Vec::new();
            domains.push(
                GLOBAL
                    .aggregator
                    .aggregate_domain(
                        self.workspace.uuid,
                        &entry.common_name,
                        DomainCertainty::Unverified,
                        user_uuid,
                    )
                    .await?,
            );
            for value in &entry.value_names {
                domains.push(
                    GLOBAL
                        .aggregator
                        .aggregate_domain(
                            self.workspace.uuid,
                            value,
                            DomainCertainty::Unverified,
                            user_uuid,
                        )
                        .await?,
                );
            }

            insert!(&mut tx, AggregationSource)
                .return_nothing()
                .bulk(domains.into_iter().map(|domain_uuid| AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(self.workspace.uuid),
                    source_type: SourceType::QueryCertificateTransparency,
                    source_uuid: result_uuid,
                    aggregated_table: AggregationTable::Domain,
                    aggregated_uuid: domain_uuid,
                }))
                .await?;

            tx.commit().await?;
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
