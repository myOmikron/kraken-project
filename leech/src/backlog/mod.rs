//! This modules handles all backlog tasks

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use log::{error, info};
use rorm::{insert, Database};
use uuid::Uuid;

use crate::models::{BruteforceSubdomainsResultInsert, DnsRecordType, TcpPortScanResultInsert};
use crate::rpc::rpc_attacks::shared::dns_record::Record;
use crate::rpc::rpc_attacks::{
    BruteforceSubdomainRequest, BruteforceSubdomainResponse, TcpPortScanRequest,
};

/// The main struct for the Backlog,
/// holds a connection to the database
#[derive(Clone)]
pub struct Backlog {
    /// Connection to the database
    pub(crate) db: Database,
}

impl Backlog {
    /// Stores the [bruteforce subdomain](crate::models::BruteforceSubdomainsResult)
    /// information in the Leech's database
    pub(crate) async fn store_bruteforce_subdomains(
        &self,
        req: &BruteforceSubdomainRequest,
        response: BruteforceSubdomainResponse,
    ) {
        let Ok(attack_uuid) = Uuid::parse_str(&req.attack_uuid) else {
            error!("Invalid attack uuid");
            return;
        };

        let Some(dns_record) = &response.record else {
            info!("No DNS record");
            return;
        };

        let Some(record) = &dns_record.record else {
            info!("No dsn.Record");
            return;
        };

        let (source, destination, dns_r_type) = match record.clone() {
            Record::A(a_rec) => {
                let Some(to) = a_rec.to else {
                    info!("No a.destination");
                    return;
                };

                (
                    a_rec.source,
                    Ipv4Addr::from(to).to_string(),
                    DnsRecordType::A,
                )
            }
            Record::Aaaa(aaaa_rec) => {
                let Some(to) = aaaa_rec.to else {
                    info!("No aaaa.destination");
                    return;
                };

                (
                    aaaa_rec.source,
                    Ipv6Addr::from(to).to_string(),
                    DnsRecordType::Aaaa,
                )
            }
            Record::Cname(cname_rec) => (cname_rec.source, cname_rec.to, DnsRecordType::Cname),
        };

        if let Err(err) = insert!(&self.db, BruteforceSubdomainsResultInsert)
            .return_nothing()
            .single(&BruteforceSubdomainsResultInsert {
                uuid: Uuid::new_v4(),
                attack: attack_uuid,
                source,
                destination,
                dns_record_type: dns_r_type,
            })
            .await
        {
            error!("Could not insert data into database: {err}");
        }
    }

    /// Stores the [TCP port scan results](crate::models::TcpPortScanResult)
    /// information in the Leech's database
    pub(crate) async fn store_tcp_port_scans(
        &self,
        req: &TcpPortScanRequest,
        socket_addr: SocketAddr,
    ) {
        let Ok(attack_uuid) = Uuid::parse_str(&req.attack_uuid) else {
            error!("Invalid attack uuid");
            return;
        };

        if let Err(err) = insert!(&self.db, TcpPortScanResultInsert)
            .return_nothing()
            .single(&TcpPortScanResultInsert {
                uuid: Uuid::new_v4(),
                attack: attack_uuid,
                address: socket_addr.ip().into(),
                port: socket_addr.port() as i32,
            })
            .await
        {
            error!("Could not insert data into database: {err}");
        }
    }
}
