use ipnetwork::IpNetwork;
use kraken_proto::shared::{dns_txt_scan, spf_directive, spf_part, DnsTxtScan, DnsTxtServiceHint};
use kraken_proto::{DnsTxtScanRequest, DnsTxtScanResponse};
use rorm::insert;
use rorm::prelude::*;
use uuid::Uuid;

use crate::api::handler::attack_results::schema::{DnsTxtScanEntry, FullDnsTxtScanResult};
use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::{
    AggregationSource, AggregationTable, DnsTxtScanAttackResultInsert,
    DnsTxtScanServiceHintEntryInsert, DnsTxtScanServiceHintType, DnsTxtScanSpfEntryInsert,
    DnsTxtScanSpfType, DnsTxtScanSummaryType, DomainCertainty, HostCertainty, SourceType,
};
use crate::modules::attacks::{AttackContext, AttackError, DnsTxtScanParams, HandleAttackResponse};

impl AttackContext {
    /// Executes the "dns txt scan" attack
    pub async fn dns_txt_scan(
        &self,
        mut leech: LeechClient,
        params: DnsTxtScanParams,
    ) -> Result<(), AttackError> {
        let request = DnsTxtScanRequest {
            attack_uuid: self.attack_uuid.to_string(),
            targets: params.targets,
        };
        self.handle_streamed_response(leech.dns_txt_scan(request))
            .await
    }
}

impl HandleAttackResponse<DnsTxtScanResponse> for AttackContext {
    async fn handle_response(&self, response: DnsTxtScanResponse) -> Result<(), AttackError> {
        let DnsTxtScanResponse {
            record: Some(entry),
        } = response
        else {
            return Err(AttackError::Malformed("Missing `record`"));
        };

        let mut tx = GLOBAL.db.start_transaction().await?;

        let result = insert!(&mut tx, DnsTxtScanAttackResultInsert)
            .single(&DnsTxtScanAttackResultInsert {
                uuid: Uuid::new_v4(),
                attack: ForeignModelByField::Key(self.attack_uuid),
                domain: entry.domain.clone(),
                collection_type: match entry.info {
                    None => return Err(AttackError::Malformed("Missing `record.info`")),
                    Some(ref info) => match info {
                        dns_txt_scan::Info::WellKnown(_) => DnsTxtScanSummaryType::ServiceHints,
                        dns_txt_scan::Info::Spf(_) => DnsTxtScanSummaryType::Spf,
                    },
                },
            })
            .await?;

        let source_uuid = GLOBAL
            .aggregator
            .aggregate_domain(
                self.workspace.uuid,
                &entry.domain,
                DomainCertainty::Verified, // we just queried this domain
                self.user.uuid,
            )
            .await?;

        insert!(&mut tx, AggregationSource)
            .return_nothing()
            .single(&AggregationSource {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(self.workspace.uuid),
                source_type: SourceType::DnsTxtScan,
                source_uuid: result.uuid,
                aggregated_table: AggregationTable::Domain,
                aggregated_uuid: source_uuid,
            })
            .await?;

        let mut ws_result = FullDnsTxtScanResult {
            uuid: result.uuid,
            created_at: result.created_at,
            attack: self.attack_uuid,
            collection_type: result.collection_type,
            domain: result.domain.clone(),
            entries: Vec::new(),
        };

        let Some(rows) = generate_dns_txt_rows(result.uuid, &entry) else {
            // deserialization failure
            return Ok(());
        };

        for row in rows {
            let result = match row {
                GeneratedRow::ServiceHint(service_hint) => {
                    let r = insert!(&mut tx, DnsTxtScanServiceHintEntryInsert)
                        .single(&service_hint)
                        .await?;
                    DnsTxtScanEntry::ServiceHint {
                        uuid: r.uuid,
                        created_at: r.created_at,
                        rule: r.rule.clone(),
                        txt_type: r.txt_type,
                    }
                }
                GeneratedRow::Spf(spf) => {
                    // for A/MX/PTR/IP4/IP6 we generate corresponding domain & host entries:
                    let (store_domain, store_ip) = match spf.spf_type {
                        // DnsTxtScanSpfType::{Include, Exists, Redirect, Explanation} are excluded since they would
                        // bloat the results with generic stuff like google.com for gmail users and aren't precise
                        // enough to tell that the DNS user owns it.
                        // With A/MX/PTR/IP there is a good chance senders could be impersonated if these servers could
                        // be compromised.
                        DnsTxtScanSpfType::A => (true, false),
                        DnsTxtScanSpfType::Mx => (true, false),
                        DnsTxtScanSpfType::Ptr => (true, false),
                        DnsTxtScanSpfType::Ip => (false, true),
                        _ => (false, false),
                    };

                    if let Some(ip) = spf.spf_ip {
                        if store_ip {
                            let host_uuid = GLOBAL
                                .aggregator
                                .aggregate_host(self.workspace.uuid, ip, HostCertainty::SupposedTo)
                                .await?;

                            insert!(&mut tx, AggregationSource)
                                .return_nothing()
                                .single(&AggregationSource {
                                    uuid: Uuid::new_v4(),
                                    workspace: ForeignModelByField::Key(self.workspace.uuid),
                                    source_type: SourceType::DnsTxtScan,
                                    source_uuid: result.uuid,
                                    aggregated_table: AggregationTable::Host,
                                    aggregated_uuid: host_uuid,
                                })
                                .await?;
                        }
                    }

                    if let Some(ref domain) = spf.spf_domain {
                        if store_domain && !domain.is_empty() {
                            // TODO: domain CIDR
                            let domain_uuid = GLOBAL
                                .aggregator
                                .aggregate_domain(
                                    self.workspace.uuid,
                                    domain,
                                    DomainCertainty::Unverified,
                                    self.user.uuid,
                                )
                                .await?;

                            insert!(&mut tx, AggregationSource)
                                .return_nothing()
                                .single(&AggregationSource {
                                    uuid: Uuid::new_v4(),
                                    workspace: ForeignModelByField::Key(self.workspace.uuid),
                                    source_type: SourceType::DnsTxtScan,
                                    source_uuid: result.uuid,
                                    aggregated_table: AggregationTable::Domain,
                                    aggregated_uuid: domain_uuid,
                                })
                                .await?;
                        }
                    }

                    let r = insert!(&mut tx, DnsTxtScanSpfEntryInsert)
                        .single(&spf)
                        .await?;
                    DnsTxtScanEntry::Spf {
                        uuid: r.uuid,
                        created_at: r.created_at,
                        rule: r.rule.clone(),
                        spf_type: r.spf_type,
                        spf_ip: r.spf_ip,
                        spf_domain: r.spf_domain.clone(),
                        spf_domain_ipv4_cidr: r.spf_domain_ipv4_cidr,
                        spf_domain_ipv6_cidr: r.spf_domain_ipv6_cidr,
                    }
                }
            };

            ws_result.entries.push(result);
        }

        tx.commit().await?;

        self.send_ws(WsMessage::DnsTxtScanResult {
            attack_uuid: self.attack_uuid,
            result: ws_result,
        })
        .await;

        Ok(())
    }
}

enum GeneratedRow {
    ServiceHint(DnsTxtScanServiceHintEntryInsert),
    Spf(DnsTxtScanSpfEntryInsert),
}

fn generate_dns_txt_rows(collection_uuid: Uuid, entry: &DnsTxtScan) -> Option<Vec<GeneratedRow>> {
    Some(match entry.info.as_ref()? {
        dns_txt_scan::Info::WellKnown(list) => list
            .hints
            .iter()
            .filter_map(|hint| {
                Some(GeneratedRow::ServiceHint(
                    DnsTxtScanServiceHintEntryInsert {
                        collection: ForeignModelByField::Key(collection_uuid),
                        uuid: Uuid::new_v4(),
                        rule: hint.rule.clone(),
                        txt_type: match DnsTxtServiceHint::try_from(hint.service).ok()? {
                            DnsTxtServiceHint::HasGoogleAccount => {
                                DnsTxtScanServiceHintType::HasGoogleAccount
                            }
                            DnsTxtServiceHint::HasGlobalsignAccount => {
                                DnsTxtScanServiceHintType::HasGlobalsignAccount
                            }
                            DnsTxtServiceHint::HasGlobalsignSMime => {
                                DnsTxtScanServiceHintType::HasGlobalsignSMime
                            }
                            DnsTxtServiceHint::HasDocusignAccount => {
                                DnsTxtScanServiceHintType::HasDocusignAccount
                            }
                            DnsTxtServiceHint::HasAppleAccount => {
                                DnsTxtScanServiceHintType::HasAppleAccount
                            }
                            DnsTxtServiceHint::HasFacebookAccount => {
                                DnsTxtScanServiceHintType::HasFacebookAccount
                            }
                            DnsTxtServiceHint::HasHubspotAccount => {
                                DnsTxtScanServiceHintType::HasHubspotAccount
                            }
                            DnsTxtServiceHint::HasMsDynamics365 => {
                                DnsTxtScanServiceHintType::HasMSDynamics365
                            }
                            DnsTxtServiceHint::HasStripeAccount => {
                                DnsTxtScanServiceHintType::HasStripeAccount
                            }
                            DnsTxtServiceHint::HasOneTrustSso => {
                                DnsTxtScanServiceHintType::HasOneTrustSso
                            }
                            DnsTxtServiceHint::HasBrevoAccount => {
                                DnsTxtScanServiceHintType::HasBrevoAccount
                            }
                            DnsTxtServiceHint::OwnsAtlassianAccounts => {
                                DnsTxtScanServiceHintType::OwnsAtlassianAccounts
                            }
                            DnsTxtServiceHint::OwnsZoomAccounts => {
                                DnsTxtScanServiceHintType::OwnsZoomAccounts
                            }
                            DnsTxtServiceHint::EmailProtonMail => {
                                DnsTxtScanServiceHintType::EmailProtonMail
                            }
                        },
                    },
                ))
            })
            .collect(),
        dns_txt_scan::Info::Spf(info) => info
            .parts
            .iter()
            .map(|part| (&part.rule, part.part.as_ref()))
            .filter(|(_, part)| part.is_some())
            .filter_map(|(spf_rule, part)| {
                // checked above in `filter`
                #[allow(clippy::unwrap_used)]
                let part = part.unwrap();
                let (spf_type, spf_ip, spf_domain, spf_domain_ipv4_cidr, spf_domain_ipv6_cidr) =
                    match part {
                        spf_part::Part::Directive(directive) => {
                            match directive.mechanism.as_ref()? {
                                spf_directive::Mechanism::All(_all) => {
                                    (DnsTxtScanSpfType::All, None, None, None, None)
                                }
                                spf_directive::Mechanism::Include(include) => (
                                    DnsTxtScanSpfType::Include,
                                    None,
                                    Some(include.domain.clone()),
                                    None,
                                    None,
                                ),
                                spf_directive::Mechanism::A(a) => (
                                    DnsTxtScanSpfType::A,
                                    None,
                                    Some(a.domain.clone()),
                                    a.ipv4_cidr,
                                    a.ipv6_cidr,
                                ),
                                spf_directive::Mechanism::Mx(mx) => (
                                    DnsTxtScanSpfType::Mx,
                                    None,
                                    Some(mx.domain.clone()),
                                    mx.ipv4_cidr,
                                    mx.ipv6_cidr,
                                ),
                                spf_directive::Mechanism::Ptr(ptr) => (
                                    DnsTxtScanSpfType::Ptr,
                                    None,
                                    Some(ptr.domain.clone()),
                                    None,
                                    None,
                                ),
                                spf_directive::Mechanism::Ip(ip) => (
                                    DnsTxtScanSpfType::Ip,
                                    Some(IpNetwork::try_from(ip.ip.clone()?).ok()?),
                                    None,
                                    None,
                                    None,
                                ),
                                spf_directive::Mechanism::Exists(exists) => (
                                    DnsTxtScanSpfType::Exists,
                                    None,
                                    Some(exists.domain.clone()),
                                    None,
                                    None,
                                ),
                            }
                        }
                        spf_part::Part::Redirect(redirect) => (
                            DnsTxtScanSpfType::Redirect,
                            None,
                            Some(redirect.domain.clone()),
                            None,
                            None,
                        ),
                        spf_part::Part::Explanation(exp) => (
                            DnsTxtScanSpfType::Explanation,
                            None,
                            Some(exp.domain.clone()),
                            None,
                            None,
                        ),
                        spf_part::Part::UnknownModifier(_) => {
                            (DnsTxtScanSpfType::Modifier, None, None, None, None)
                        }
                    };
                Some(GeneratedRow::Spf(DnsTxtScanSpfEntryInsert {
                    uuid: Uuid::new_v4(),
                    collection: ForeignModelByField::Key(collection_uuid),
                    rule: spf_rule.clone(),
                    spf_type,
                    spf_ip,
                    spf_domain,
                    spf_domain_ipv4_cidr,
                    spf_domain_ipv6_cidr,
                }))
            })
            .collect(),
    })
}
