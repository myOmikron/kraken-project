use std::future::Future;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::str::FromStr;

use ipnetwork::IpNetwork;
use kraken_proto::shared;
use kraken_proto::shared::dns_record::Record;
use kraken_proto::DnsResolutionRequest;
use kraken_proto::DnsResolutionResponse;
use rorm::insert;
use rorm::prelude::*;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::DnsRecordType;
use crate::models::DnsResolutionResultInsert;
use crate::models::DomainCertainty;
use crate::models::DomainDomainRelation;
use crate::models::DomainHostRelation;
use crate::models::HostCertainty;
use crate::models::SourceType;
use crate::modules::attacks::AttackContext;
use crate::modules::attacks::AttackError;
use crate::modules::attacks::DnsResolutionParams;
use crate::modules::attacks::HandleAttackResponse;

impl AttackContext {
    // What's up with this signature?
    //
    // It is a workaround for an interesting problem:
    // `dns_resolution` calls `Domain::aggregate` which calls `dns_resolution` again
    // and SPAWNS it as task.
    // This puts the compiler in a dependency loop:
    // `tokio::spawn` requires the future to be `Send`, so the compiler evaluates `dns_resolution`.
    // Proofing `dns_resolution`'s future to be `Send` requires `Domain::aggregate`'s future to be.
    // But `Domain::aggregate` can't be evaluated yet since the `tokio::spawn` statement isn't figured out yet.
    // Solution:
    // Annotate this function's future explicitly as `Send`, so `tokio::spawn` doesn't trigger the check
    // and the compiler can proof this bound separately after checking `Domain::aggregate`.
    #[allow(clippy::manual_async_fn)]
    /// Executes the "dns resolution" attack
    pub fn dns_resolution(
        &self,
        mut leech: LeechClient,
        params: DnsResolutionParams,
    ) -> impl Future<Output = Result<(), AttackError>> + Send + '_ {
        async move {
            let request = DnsResolutionRequest {
                attack_uuid: self.attack_uuid.to_string(),
                targets: params.targets,
                concurrent_limit: params.concurrent_limit,
            };
            self.handle_streamed_response(leech.dns_resolution(request))
                .await
        }
    }
}
impl HandleAttackResponse<DnsResolutionResponse> for AttackContext {
    async fn handle_response(&self, response: DnsResolutionResponse) -> Result<(), AttackError> {
        let DnsResolutionResponse {
            record: Some(shared::DnsRecord {
                record: Some(record),
            }),
        } = response
        else {
            return Err(AttackError::Malformed("Missing `record`"));
        };

        let source;
        let destination;
        let dns_record_type;
        match record {
            Record::A(a_rec) => {
                let to = a_rec
                    .to
                    .ok_or(AttackError::Malformed("Missing `record.record.a.to`"))?;
                source = a_rec.source;
                destination = Ipv4Addr::from(to).to_string();
                dns_record_type = DnsRecordType::A;
            }
            Record::Aaaa(aaaa_rec) => {
                let to = aaaa_rec.to.ok_or(AttackError::Malformed(
                    "Missing field `record.record.aaaa.to`",
                ))?;
                source = aaaa_rec.source;
                destination = Ipv6Addr::from(to).to_string();
                dns_record_type = DnsRecordType::Aaaa;
            }
            Record::Cname(cname_rec) => {
                source = cname_rec.source;
                destination = cname_rec.to;
                dns_record_type = DnsRecordType::Cname;
            }
            Record::Caa(caa_rec) => {
                source = caa_rec.source;
                destination = caa_rec.to;
                dns_record_type = DnsRecordType::Caa;
            }
            Record::Mx(mx_rec) => {
                source = mx_rec.source;
                destination = mx_rec.to;
                dns_record_type = DnsRecordType::Mx;
            }
            Record::Tlsa(tlsa_rec) => {
                source = tlsa_rec.source;
                destination = tlsa_rec.to;
                dns_record_type = DnsRecordType::Tlsa;
            }
            Record::Txt(txt_rec) => {
                source = txt_rec.source;
                destination = txt_rec.to;
                dns_record_type = DnsRecordType::Txt;
            }
        };

        self.send_ws(WsMessage::DnsResolutionResult {
            attack_uuid: self.attack_uuid,
            source: source.clone(),
            destination: destination.clone(),
        })
        .await;

        let mut tx = GLOBAL.db.start_transaction().await?;

        let result_uuid = insert!(&mut tx, DnsResolutionResultInsert)
            .return_primary_key()
            .single(&DnsResolutionResultInsert {
                uuid: Uuid::new_v4(),
                attack: ForeignModelByField::Key(self.attack_uuid),
                dns_record_type,
                source: source.clone(),
                destination: destination.clone(),
            })
            .await?;

        let source_uuid = GLOBAL
            .aggregator
            .aggregate_domain(
                self.workspace.uuid,
                &source,
                DomainCertainty::Verified, // we just queried this domain
                self.user.uuid,
            )
            .await?;

        let destination = match dns_record_type {
            DnsRecordType::A | DnsRecordType::Aaaa => {
                let host_uuid = GLOBAL
                    .aggregator
                    .aggregate_host(
                        self.workspace.uuid,
                        // Unwrap is okay, as A and AAAA result in valid IP addresses
                        #[allow(clippy::unwrap_used)]
                        IpNetwork::from_str(&destination).unwrap(),
                        HostCertainty::SupposedTo, // there is a current dns record to it
                    )
                    .await?;

                DomainHostRelation::insert_if_missing(
                    &mut tx,
                    self.workspace.uuid,
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
                        self.workspace.uuid,
                        &destination,
                        DomainCertainty::Unverified, // we haven't queried this domain yet
                        self.user.uuid,
                    )
                    .await?;

                DomainDomainRelation::insert_if_missing(
                    &mut tx,
                    self.workspace.uuid,
                    source_uuid,
                    destination_uuid,
                )
                .await?;

                Some((AggregationTable::Domain, destination_uuid))
            }
            _ => None,
        };

        insert!(&mut tx, AggregationSource)
            .return_nothing()
            .bulk(
                [AggregationSource {
                    uuid: Uuid::new_v4(),
                    workspace: ForeignModelByField::Key(self.workspace.uuid),
                    source_type: SourceType::DnsResolution,
                    source_uuid: result_uuid,
                    aggregated_table: AggregationTable::Domain,
                    aggregated_uuid: source_uuid,
                }]
                .into_iter()
                .chain(
                    destination.map(|(aggregated_table, aggregated_uuid)| AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(self.workspace.uuid),
                        source_type: SourceType::DnsResolution,
                        source_uuid: result_uuid,
                        aggregated_table,
                        aggregated_uuid,
                    }),
                ),
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
