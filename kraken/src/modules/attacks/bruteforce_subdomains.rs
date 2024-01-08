use std::net::{Ipv4Addr, Ipv6Addr};

use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::DnsRecordType;
use crate::modules::attack_results::store_bruteforce_subdomains_result;
use crate::modules::attacks::{
    AttackContext, AttackError, BruteforceSubdomainsParams, HandleAttackResponse,
};
use crate::rpc::rpc_definitions::shared::dns_record::Record;
use crate::rpc::rpc_definitions::{
    shared, BruteforceSubdomainRequest, BruteforceSubdomainResponse,
};

impl AttackContext {
    /// Executes the "bruteforce subdomains" attack
    pub async fn bruteforce_subdomains(
        &self,
        mut leech: LeechClient,
        params: BruteforceSubdomainsParams,
    ) -> Result<(), AttackError> {
        let request = BruteforceSubdomainRequest {
            attack_uuid: self.attack_uuid.to_string(),
            domain: params.target,
            wordlist_path: params.wordlist_path,
            concurrent_limit: params.concurrent_limit,
        };
        self.handle_streamed_response(leech.bruteforce_subdomains(request))
            .await
    }
}
impl HandleAttackResponse<BruteforceSubdomainResponse> for AttackContext {
    async fn handle_response(
        &self,
        response: BruteforceSubdomainResponse,
    ) -> Result<(), AttackError> {
        let BruteforceSubdomainResponse {
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
            _ => {
                return Err(AttackError::Malformed("unexpected `record type`"));
            }
        };

        self.send_ws(WsMessage::BruteforceSubdomainsResult {
            attack_uuid: self.attack_uuid,
            source: source.clone(),
            destination: destination.clone(),
        })
        .await;

        store_bruteforce_subdomains_result(
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
