use std::future::Future;
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::DnsRecordType;
use crate::modules::attack_results::store_dns_resolution_result;
use crate::modules::attacks::{
    AttackContext, AttackError, DnsResolutionParams, HandleAttackResponse,
};
use crate::rpc::rpc_definitions::shared::dns_record::Record;
use crate::rpc::rpc_definitions::{shared, DnsResolutionRequest, DnsResolutionResponse};

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

        store_dns_resolution_result(
            &GLOBAL.db,
            self.attack_uuid,
            self.workspace.uuid,
            source,
            destination,
            dns_record_type,
        )
        .await?;

        Ok(())
    }
}
