use chrono::{NaiveDateTime, TimeZone, Utc};
use rorm::db::Executor;
use rorm::prelude::*;
use rorm::{insert, query};
use uuid::Uuid;

use crate::models::{
    AggregationSource, AggregationTable, Attack, CertificateTransparencyResultInsert,
    CertificateTransparencyValueNameInsert, Domain, DomainCertainty, SourceType,
};
use crate::rpc::rpc_definitions::shared::CertEntry;

/// Store a query certificate transparency's result and update the aggregated domains and hosts
pub async fn store_query_certificate_transparency_result(
    executor: impl Executor<'_>,
    attack_uuid: Uuid,
    workspace_uuid: Uuid,
    entry: CertEntry,
) -> Result<(), rorm::Error> {
    let mut guard = executor.ensure_transaction().await?;
    let tx = guard.get_transaction();

    let user_uuid = *query!(&mut *tx, (Attack::F.started_by,))
        .condition(Attack::F.uuid.equals(attack_uuid))
        .one()
        .await?
        .0
        .key();

    let result_uuid = insert!(&mut *tx, CertificateTransparencyResultInsert)
        .return_primary_key()
        .single(&CertificateTransparencyResultInsert {
            uuid: Uuid::new_v4(),
            attack: ForeignModelByField::Key(attack_uuid),
            issuer_name: entry.issuer_name,
            serial_number: entry.serial_number,
            common_name: entry.common_name.clone(),
            not_before: entry.not_before.clone().map(|x| {
                Utc.from_utc_datetime(
                    &NaiveDateTime::from_timestamp_millis(x.seconds * 1000).unwrap(),
                )
            }),
            not_after: entry.not_after.clone().map(|x| {
                Utc.from_utc_datetime(
                    &NaiveDateTime::from_timestamp_millis(x.seconds * 1000).unwrap(),
                )
            }),
        })
        .await?;

    insert!(&mut *tx, CertificateTransparencyValueNameInsert)
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
        Domain::aggregate(
            &mut *tx,
            workspace_uuid,
            &entry.common_name,
            DomainCertainty::Unverified,
            user_uuid,
        )
        .await?,
    );
    for value in &entry.value_names {
        domains.push(
            Domain::aggregate(
                &mut *tx,
                workspace_uuid,
                value,
                DomainCertainty::Unverified,
                user_uuid,
            )
            .await?,
        );
    }

    insert!(&mut *tx, AggregationSource)
        .return_nothing()
        .bulk(domains.into_iter().map(|domain_uuid| AggregationSource {
            uuid: Uuid::new_v4(),
            workspace: ForeignModelByField::Key(workspace_uuid),
            source_type: SourceType::QueryCertificateTransparency,
            source_uuid: result_uuid,
            aggregated_table: AggregationTable::Domain,
            aggregated_uuid: domain_uuid,
        }))
        .await?;

    guard.commit().await
}
