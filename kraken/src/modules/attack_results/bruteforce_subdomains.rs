use std::str::FromStr;

use ipnetwork::IpNetwork;
use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use uuid::Uuid;

use crate::models::{
    AggregationSource, AggregationTable, AttackType, BruteforceSubdomainsResultInsert,
    DnsRecordType, Domain, DomainCertainty, DomainDomainRelation, DomainHostRelation, Host,
    HostCertainty,
};

/// Store a bruteforce subdomains' result and update the aggregated domains and hosts
pub async fn store_bruteforce_subdomains_result(
    executor: impl Executor<'_>,
    attack_uuid: Uuid,
    workspace_uuid: Uuid,
    source: String,
    destination: String,
    dns_record_type: DnsRecordType,
) -> Result<(), rorm::Error> {
    let mut guard = executor.ensure_transaction().await?;
    let tx = guard.get_transaction();

    let result_uuid = insert!(&mut *tx, BruteforceSubdomainsResultInsert)
        .return_primary_key()
        .single(&BruteforceSubdomainsResultInsert {
            uuid: Uuid::new_v4(),
            attack: ForeignModelByField::Key(attack_uuid),
            dns_record_type,
            source: source.clone(),
            destination: destination.clone(),
        })
        .await?;

    let source_uuid = Domain::aggregate(
        &mut *tx,
        workspace_uuid,
        &source,
        DomainCertainty::Verified, // we just queried this domain
    )
    .await?;

    let destination = match dns_record_type {
        DnsRecordType::A | DnsRecordType::Aaaa => {
            let host_uuid = Host::aggregate(
                &mut *tx,
                workspace_uuid,
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
            let destination_uuid = Domain::aggregate(
                &mut *tx,
                workspace_uuid,
                &destination,
                DomainCertainty::Unverified, // we haven't queried this domain yet
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
                result_type: AttackType::BruteforceSubdomains,
                result_uuid,
                aggregated_table: AggregationTable::Domain,
                aggregated_uuid: source_uuid,
            }]
            .into_iter()
            .chain(
                destination.map(|(aggregated_table, aggregated_uuid)| AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(workspace_uuid),
                    result_type: AttackType::BruteforceSubdomains,
                    result_uuid,
                    aggregated_table,
                    aggregated_uuid,
                }),
            ),
        )
        .await?;

    guard.commit().await
}
