use ipnetwork::IpNetwork;
use kraken_proto::shared::{dns_txt_scan, spf_directive, spf_part, DnsTxtKnownEntry, DnsTxtScan};
use kraken_proto::{DnsTxtScanRequest, DnsTxtScanResponse};
use rorm::insert;
use rorm::prelude::*;
use uuid::Uuid;

use crate::api::handler::attack_results::schema::SimpleDnsTxtScanResult;
use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::{
    AggregationSource, AggregationTable, DnsTxtScanResultInsert, DnsTxtScanType, DomainCertainty,
    HostCertainty, SourceType,
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

        let Some(rows) = generate_dns_txt_rows(self.attack_uuid, &entry) else {
            // deserialization failure
            return Ok(());
        };

        let mut ws_entries = Vec::new();

        for row in rows {
            let result = insert!(&mut tx, DnsTxtScanResultInsert)
                .single(&row)
                .await?;

            ws_entries.push(SimpleDnsTxtScanResult {
                uuid: result.uuid,
                attack: self.attack_uuid,
                txt_type: result.txt_type,
                domain: result.domain.clone(),
                rule: result.rule.clone(),
                spf_domain: result.spf_domain.clone(),
                spf_ip: result.spf_ip.clone(),
                spf_domain_ipv6_cidr: result.spf_domain_ipv6_cidr,
                spf_domain_ipv4_cidr: result.spf_domain_ipv4_cidr,
                created_at: result.created_at,
            });

            let (store_domain, store_ip) = match row.txt_type {
                // TODO: store these as service? should be a service or some kind of
                // hint to the user to look at it, since it may just be something
                // like Microsoft servers, since they are using their service.
                // Additionally: all the other scan types that are just services.
                // DnsTxtScanType::SpfInclude
                // DnsTxtScanType::SpfExists
                // DnsTxtScanType::SpfRedirect
                // DnsTxtScanType::SpfExplanation
                DnsTxtScanType::SpfA => (true, false),
                DnsTxtScanType::SpfMx => (true, false),
                DnsTxtScanType::SpfPtr => (true, false),
                DnsTxtScanType::SpfIp => (false, true),
                _ => (false, false),
            };

            if let Some(ip) = row.spf_ip {
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

            if let Some(domain) = row.spf_domain {
                if store_domain && !domain.is_empty() {
                    // TODO: domain CIDR
                    let domain_uuid = GLOBAL
                        .aggregator
                        .aggregate_domain(
                            self.workspace.uuid,
                            &domain,
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
        }

        tx.commit().await?;

        self.send_ws(WsMessage::DnsTxtScanResult {
            attack_uuid: self.attack_uuid,
            entries: ws_entries,
        })
        .await;

        Ok(())
    }
}

fn generate_dns_txt_rows(
    attack_uuid: Uuid,
    entry: &DnsTxtScan,
) -> Option<Vec<DnsTxtScanResultInsert>> {
    Some(match entry.info.as_ref()? {
        dns_txt_scan::Info::WellKnown(num) => vec![DnsTxtScanResultInsert {
            uuid: Uuid::new_v4(),
            attack: ForeignModelByField::Key(attack_uuid),
            domain: entry.domain.clone(),
            rule: entry.rule.clone(),
            txt_type: match DnsTxtKnownEntry::try_from(*num).ok()? {
                DnsTxtKnownEntry::HasGoogleAccount => DnsTxtScanType::HasGoogleAccount,
                DnsTxtKnownEntry::HasGlobalsignAccount => DnsTxtScanType::HasGlobalsignAccount,
                DnsTxtKnownEntry::HasGlobalsignSMime => DnsTxtScanType::HasGlobalsignSMime,
                DnsTxtKnownEntry::HasDocusignAccount => DnsTxtScanType::HasDocusignAccount,
                DnsTxtKnownEntry::HasAppleAccount => DnsTxtScanType::HasAppleAccount,
                DnsTxtKnownEntry::HasFacebookAccount => DnsTxtScanType::HasFacebookAccount,
                DnsTxtKnownEntry::HasHubspotAccount => DnsTxtScanType::HasHubspotAccount,
                DnsTxtKnownEntry::HasMsDynamics365 => DnsTxtScanType::HasMSDynamics365,
                DnsTxtKnownEntry::HasStripeAccount => DnsTxtScanType::HasStripeAccount,
                DnsTxtKnownEntry::HasOneTrustSso => DnsTxtScanType::HasOneTrustSso,
                DnsTxtKnownEntry::HasBrevoAccount => DnsTxtScanType::HasBrevoAccount,
                DnsTxtKnownEntry::OwnsAtlassianAccounts => DnsTxtScanType::OwnsAtlassianAccounts,
                DnsTxtKnownEntry::OwnsZoomAccounts => DnsTxtScanType::OwnsZoomAccounts,
                DnsTxtKnownEntry::EmailProtonMail => DnsTxtScanType::EmailProtonMail,
            },
            spf_ip: None,
            spf_domain: None,
            spf_domain_ipv4_cidr: None,
            spf_domain_ipv6_cidr: None,
        }],
        dns_txt_scan::Info::Spf(info) => info
            .parts
            .iter()
            .map(|part| (&part.rule, part.part.as_ref()))
            .filter(|(_, part)| part.is_some())
            .filter_map(|(spf_rule, part)| {
                // checked above in `filter`
                #[allow(clippy::unwrap_used)]
                let part = part.unwrap();
                let (txt_type, spf_ip, spf_domain, spf_domain_ipv4_cidr, spf_domain_ipv6_cidr) =
                    match part {
                        spf_part::Part::Directive(directive) => {
                            match directive.mechanism.as_ref()? {
                                spf_directive::Mechanism::All(_all) => {
                                    (DnsTxtScanType::SpfAll, None, None, None, None)
                                }
                                spf_directive::Mechanism::Include(include) => (
                                    DnsTxtScanType::SpfInclude,
                                    None,
                                    Some(include.domain.clone()),
                                    None,
                                    None,
                                ),
                                spf_directive::Mechanism::A(a) => (
                                    DnsTxtScanType::SpfA,
                                    None,
                                    Some(a.domain.clone()),
                                    a.ipv4_cidr,
                                    a.ipv6_cidr,
                                ),
                                spf_directive::Mechanism::Mx(mx) => (
                                    DnsTxtScanType::SpfMx,
                                    None,
                                    Some(mx.domain.clone()),
                                    mx.ipv4_cidr,
                                    mx.ipv6_cidr,
                                ),
                                spf_directive::Mechanism::Ptr(ptr) => (
                                    DnsTxtScanType::SpfPtr,
                                    None,
                                    Some(ptr.domain.clone()),
                                    None,
                                    None,
                                ),
                                spf_directive::Mechanism::Ip(ip) => (
                                    DnsTxtScanType::SpfIp,
                                    Some(IpNetwork::try_from(ip.ip.clone()?).ok()?),
                                    None,
                                    None,
                                    None,
                                ),
                                spf_directive::Mechanism::Exists(exists) => (
                                    DnsTxtScanType::SpfExists,
                                    None,
                                    Some(exists.domain.clone()),
                                    None,
                                    None,
                                ),
                            }
                        }
                        spf_part::Part::Redirect(redirect) => (
                            DnsTxtScanType::SpfRedirect,
                            None,
                            Some(redirect.domain.clone()),
                            None,
                            None,
                        ),
                        spf_part::Part::Explanation(exp) => (
                            DnsTxtScanType::SpfExplanation,
                            None,
                            Some(exp.domain.clone()),
                            None,
                            None,
                        ),
                        spf_part::Part::UnknownModifier(_) => {
                            (DnsTxtScanType::SpfModifier, None, None, None, None)
                        }
                    };
                Some(DnsTxtScanResultInsert {
                    uuid: Uuid::new_v4(),
                    attack: ForeignModelByField::Key(attack_uuid),
                    domain: entry.domain.clone(),
                    rule: spf_rule.clone(),
                    txt_type,
                    spf_ip,
                    spf_domain,
                    spf_domain_ipv4_cidr,
                    spf_domain_ipv6_cidr,
                })
            })
            .collect(),
    })
}
