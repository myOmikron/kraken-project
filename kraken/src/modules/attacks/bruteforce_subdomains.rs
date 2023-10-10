use std::net::{Ipv4Addr, Ipv6Addr};

use futures::StreamExt;
use log::{debug, error, warn};
use rorm::prelude::*;
use rorm::{and, insert, query};
use uuid::Uuid;

use crate::chan::WsMessage;
use crate::models::{
    BruteforceSubdomainsResult, BruteforceSubdomainsResultInsert, DnsRecordType, Domain,
};
use crate::modules::attacks::LeechAttackContext;
use crate::rpc::rpc_definitions::shared::dns_record::Record;
use crate::rpc::rpc_definitions::BruteforceSubdomainRequest;

impl LeechAttackContext {
    /// Bruteforce subdomains through a DNS wordlist attack
    ///
    /// See [`handler::attacks::bruteforce_subdomains`] for more information.
    pub async fn bruteforce_subdomains(mut self, req: BruteforceSubdomainRequest) {
        match self.leech.bruteforce_subdomains(req).await {
            Ok(v) => {
                let mut stream = v.into_inner();

                while let Some(res) = stream.next().await {
                    match res {
                        Ok(v) => {
                            let Some(record) = v.record else {
                                warn!("Missing field record in grpc response of bruteforce subdomains");
                                continue;
                            };
                            let Some(record) = record.record else {
                                warn!("Missing field record.record in grpc response of bruteforce subdomains");
                                continue;
                            };

                            let source;
                            let destination;
                            let dns_record_type;
                            match record {
                                Record::A(a_rec) => {
                                    let Some(to) = a_rec.to else {
                                        warn!("Missing field record.record.a.to in grpc response of bruteforce subdomains");
                                        continue;
                                    };
                                    source = a_rec.source;
                                    destination = Ipv4Addr::from(to).to_string();
                                    dns_record_type = DnsRecordType::A;
                                }
                                Record::Aaaa(aaaa_rec) => {
                                    let Some(to) = aaaa_rec.to else {
                                        warn!("Missing field record.record.aaaa.to in grpc response of bruteforce subdomains");
                                        continue;
                                    };
                                    source = aaaa_rec.source;
                                    destination = Ipv6Addr::from(to).to_string();
                                    dns_record_type = DnsRecordType::Aaaa;
                                }
                                Record::Cname(cname_rec) => {
                                    source = cname_rec.source;
                                    destination = cname_rec.to;
                                    dns_record_type = DnsRecordType::Cname;
                                }
                            };

                            if let Err(err) = self
                                .insert_bruteforce_subdomains_result(
                                    source.clone(),
                                    destination.clone(),
                                    dns_record_type,
                                )
                                .await
                            {
                                error!("Could not insert data in db: {err}");
                                return;
                            }

                            self.send_ws(WsMessage::BruteforceSubdomainsResult {
                                attack_uuid: self.attack_uuid,
                                source,
                                destination,
                            })
                            .await;
                        }
                        Err(err) => {
                            error!("Error while reading from stream: {err}");
                            self.set_finished(false).await;
                            return;
                        }
                    }
                }
            }
            Err(err) => {
                error!("Error while reading from stream: {err}");
                self.set_finished(false).await;
                return;
            }
        };

        self.set_finished(true).await;
    }

    /// Insert a tcp port scan's result and update the aggregation
    async fn insert_bruteforce_subdomains_result(
        &self,
        source: String,
        destination: String,
        dns_record_type: DnsRecordType,
    ) -> Result<(), rorm::Error> {
        let mut tx = self.db.start_transaction().await?;

        if query!(&mut tx, BruteforceSubdomainsResult)
            .condition(and!(
                BruteforceSubdomainsResult::F
                    .attack
                    .equals(self.attack_uuid),
                BruteforceSubdomainsResult::F
                    .dns_record_type
                    .equals(dns_record_type.clone()),
                BruteforceSubdomainsResult::F.source.equals(&source),
                BruteforceSubdomainsResult::F
                    .destination
                    .equals(&destination)
            ))
            .optional()
            .await?
            .is_some()
        {
            debug!("entry already exists");
        } else {
            insert!(&mut tx, BruteforceSubdomainsResult)
                .single(&BruteforceSubdomainsResultInsert {
                    uuid: Uuid::new_v4(),
                    attack: ForeignModelByField::Key(self.attack_uuid),
                    dns_record_type,
                    source: source.clone(),
                    destination: destination.clone(),
                })
                .await?;

            Domain::insert_if_missing(&mut tx, self.workspace_uuid, &source).await?;
        }

        tx.commit().await
    }
}
