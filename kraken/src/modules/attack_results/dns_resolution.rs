use std::str::FromStr;

use ipnetwork::IpNetwork;
use rorm::db::Executor;
use rorm::prelude::*;
use rorm::{insert, query};
use uuid::Uuid;

use crate::chan::GLOBAL;
use crate::models::{
    AggregationSource, AggregationTable, Attack, DnsRecordType, DnsResolutionResultInsert,
    DomainCertainty, DomainDomainRelation, DomainHostRelation, HostCertainty, SourceType,
};

/// Store a dns resolution's result and update the aggregated domains and hosts
pub async fn store_dns_resolution_result(
    executor: impl Executor<'_>,
    attack_uuid: Uuid,
    workspace_uuid: Uuid,
    source: String,
    destination: String,
    dns_record_type: DnsRecordType,
) -> Result<(), rorm::Error> {
    let mut guard = executor.ensure_transaction().await?;
    let tx = guard.get_transaction();

    let user_uuid = *query!(&mut *tx, (Attack::F.started_by,))
        .condition(Attack::F.uuid.equals(attack_uuid))
        .one()
        .await?
        .0
        .key();

    let result_uuid = insert!(&mut *tx, DnsResolutionResultInsert)
        .return_primary_key()
        .single(&DnsResolutionResultInsert {
            uuid: Uuid::new_v4(),
            attack: ForeignModelByField::Key(attack_uuid),
            dns_record_type,
            source: source.clone(),
            destination: destination.clone(),
        })
        .await?;

    let source_uuid = GLOBAL
        .aggregator
        .aggregate_domain(
            workspace_uuid,
            &source,
            DomainCertainty::Verified, // we just queried this domain
            user_uuid,
        )
        .await?;

    let destination = match dns_record_type {
        DnsRecordType::A | DnsRecordType::Aaaa => {
            let host_uuid = GLOBAL
                .aggregator
                .aggregate_host(
                    workspace_uuid,
                    // Unwrap is okay, as A and AAAA result in valid IP addresses
                    #[allow(clippy::unwrap_used)]
                    IpNetwork::from_str(&destination).unwrap(),
                    HostCertainty::SupposedTo, // there is a current dns record to it
                )
                .await?;

            DomainHostRelation::insert_if_missing(
                &mut *tx,
                workspace_uuid,
                source_uuid,
                host_uuid,
                true,
            )
            .await?;

            Some((AggregationTable::Host, host_uuid))
        }
        DnsRecordType::Cname => {
            let destination_uuid = GLOBAL
                .aggregator
                .aggregate_domain(
                    workspace_uuid,
                    &destination,
                    DomainCertainty::Unverified, // we haven't queried this domain yet
                    user_uuid,
                )
                .await?;

            DomainDomainRelation::insert_if_missing(
                &mut *tx,
                workspace_uuid,
                source_uuid,
                destination_uuid,
            )
            .await?;

            Some((AggregationTable::Domain, destination_uuid))
        }
        _ => None,
    };

    insert!(&mut *tx, AggregationSource)
        .return_nothing()
        .bulk(
            [AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(workspace_uuid),
                source_type: SourceType::DnsResolution,
                source_uuid: result_uuid,
                aggregated_table: AggregationTable::Domain,
                aggregated_uuid: source_uuid,
            }]
            .into_iter()
            .chain(
                destination.map(|(aggregated_table, aggregated_uuid)| AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(workspace_uuid),
                    source_type: SourceType::DnsResolution,
                    source_uuid: result_uuid,
                    aggregated_table,
                    aggregated_uuid,
                }),
            ),
        )
        .await?;

    guard.commit().await
}
