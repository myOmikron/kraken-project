use std::net::{Ipv4Addr, Ipv6Addr};

use crate::chan::{WsMessage, GLOBAL};
use crate::models::DnsRecordType;
use crate::modules::attack_results::store_dns_resolution_result;
use crate::modules::attacks::{AttackContext, AttackError, LeechAttackContext};
use crate::rpc::rpc_definitions;
use crate::rpc::rpc_definitions::shared::dns_record::Record;
use crate::rpc::rpc_definitions::{shared, DnsResolutionResponse};

impl LeechAttackContext {
    /// Resolve domain names
    ///
    /// See [`handler::attacks::dns_resolution`] for more information.
    pub async fn dns_resolution(mut self, req: rpc_definitions::DnsResolutionRequest) {
        let result = AttackContext::handle_streamed_response(
            self.leech.dns_resolution(req).await,
            |response| async {
                let DnsResolutionResponse {
                    record:
                        Some(shared::DnsRecord {
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

                store_dns_resolution_result(
                    &GLOBAL.db,
                    self.attack_uuid,
                    self.workspace_uuid,
                    source,
                    destination,
                    dns_record_type,
                )
                .await?;

                Ok(())
            },
        )
        .await;

        self.set_finished(result.err()).await;
    }
}
