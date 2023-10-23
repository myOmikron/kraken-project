//! This modules handles all backlog tasks

use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use ipnetwork::IpNetwork;
use log::{debug, error, info, warn};
use rorm::{delete, insert, query, Database};
use uuid::Uuid;

use crate::config::KrakenConfig;
use crate::models::{
    BruteforceSubdomainsResultInsert, DnsRecordType, DnsResolutionResult,
    DnsResolutionResultInsert, DnsResult, HostAliveResult, HostAliveResultInsert,
    TcpPortScanResult, TcpPortScanResultInsert,
};
use crate::rpc::rpc_attacks::backlog_service_client::BacklogServiceClient;
use crate::rpc::rpc_attacks::shared::dns_record::Record;
use crate::rpc::rpc_attacks::{
    BacklogDnsRequest, BacklogHostAliveRequest, BacklogTcpPortScanRequest,
    BruteforceSubdomainResponse, DnsResolutionResponse,
};
use crate::utils::kraken_endpoint;

/// The main struct for the Backlog,
/// holds a connection to the database
#[derive(Clone)]
pub struct Backlog {
    db: Database,
}

impl Backlog {
    /// Stores the [bruteforce subdomain](crate::models::BruteforceSubdomainsResult)
    /// information in the Leech's database
    pub(crate) async fn store_bruteforce_subdomains(
        &self,
        attack_uuid: Uuid,
        response: BruteforceSubdomainResponse,
    ) {
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
            _ => {
                debug!("record type not of concern");
                return;
            }
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
    pub(crate) async fn store_tcp_port_scans(&self, attack_uuid: Uuid, socket_addr: SocketAddr) {
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

    /// Stores the [Host alive results](crate::models::HostAliveResult)
    /// information in the Leech's database
    pub(crate) async fn store_hosts_alive_check(&self, attack_uuid: Uuid, item: IpAddr) {
        if let Err(err) = insert!(&self.db, HostAliveResult)
            .return_nothing()
            .single(&HostAliveResultInsert {
                uuid: Uuid::new_v4(),
                attack: attack_uuid,
                host: IpNetwork::from(item),
            })
            .await
        {
            error!("Could not insert data into database: {err}");
        }
    }

    /// Stores the [DNS resolution results](crate::models::DnsResolutionResult)
    /// information in the Leech's database
    pub(crate) async fn store_dns_resolution(
        &self,
        attack_uuid: Uuid,
        item: DnsResolutionResponse,
    ) {
        let Some(dns_record) = &item.record else {
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
            Record::Caa(caa_rec) => (caa_rec.source, caa_rec.to, DnsRecordType::Caa),
            Record::Mx(mx_rec) => (mx_rec.source, mx_rec.to, DnsRecordType::Mx),
            Record::Tlsa(tlsa_rec) => (tlsa_rec.source, tlsa_rec.to, DnsRecordType::Tlsa),
            Record::Txt(txt_rec) => (txt_rec.source, txt_rec.to, DnsRecordType::Txt),
        };

        if let Err(err) = insert!(&self.db, DnsResolutionResult)
            .return_nothing()
            .single(&DnsResolutionResultInsert {
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
}

const KRAKEN_RETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);
const DB_QUERY_INTERVAL: Duration = Duration::from_secs(10);
const DB_QUERY_LIMIT: u64 = 1000;

/// Starts the backlog upload server
pub async fn start_backlog(
    db: Database,
    kraken_config: &KrakenConfig,
) -> Result<Backlog, Box<dyn Error>> {
    let kraken_endpoint = kraken_endpoint(kraken_config)?;

    let db_clone = db.clone();
    tokio::spawn(async move {
        loop {
            let mut kraken;
            loop {
                let Ok(chan) = kraken_endpoint.connect().await else {
                    debug!(
                        "could not connect to kraken, retrying in {} minutes",
                        KRAKEN_RETRY_INTERVAL.as_secs() / 60
                    );
                    tokio::time::sleep(KRAKEN_RETRY_INTERVAL).await;
                    continue;
                };
                kraken = BacklogServiceClient::new(chan);
                info!("connected to kraken @ {}", kraken_endpoint.uri());
                break;
            }

            let mut data_changed;
            loop {
                data_changed = false;
                let Ok(mut db_trx) = db_clone.start_transaction().await else {
                    continue;
                };

                if let Ok(data) = query!(&mut db_trx, DnsResult)
                    .limit(DB_QUERY_LIMIT)
                    .all()
                    .await
                {
                    if !data.is_empty() {
                        data_changed = true;
                        if let Err(e) = delete!(&mut db_trx, DnsResult).bulk(&data).await {
                            warn!("bulk delete failed: {e}");
                        };

                        let data: BacklogDnsRequest = data.into();
                        if let Err(e) = kraken.dns_results(data).await {
                            error!("could not send data to kraken: {e}. Restarting connection");
                            break;
                        }
                    };
                }; // end DnsResult

                if let Ok(data) = query!(&mut db_trx, TcpPortScanResult)
                    .limit(DB_QUERY_LIMIT)
                    .all()
                    .await
                {
                    if !data.is_empty() {
                        data_changed = true;
                        if let Err(e) = delete!(&mut db_trx, TcpPortScanResult).bulk(&data).await {
                            warn!("bulk delete failed: {e}");
                        };

                        let data: BacklogTcpPortScanRequest = data.into();
                        if let Err(e) = kraken.tcp_port_scan(data).await {
                            error!("could not send data to kraken: {e}. restarting connection");
                            break;
                        }
                    }
                }; // end TcpPortScanResult

                if let Ok(data) = query!(&mut db_trx, HostAliveResult)
                    .limit(DB_QUERY_LIMIT)
                    .all()
                    .await
                {
                    if !data.is_empty() {
                        data_changed = true;
                        if let Err(e) = delete!(&mut db_trx, HostAliveResult).bulk(&data).await {
                            warn!("bulk delete failed: {e}");
                        };

                        let data: BacklogHostAliveRequest = data.into();
                        if let Err(e) = kraken.host_alive_check(data).await {
                            error!("could not send data to kraken: {e}. restarting connection");
                            break;
                        }
                    }
                }; // end HostAliveResult

                if data_changed {
                    if let Err(e) = db_trx.commit().await {
                        error!("error committing changes to database: {e}");
                    }
                    info!("database committed");
                }

                tokio::time::sleep(DB_QUERY_INTERVAL).await;
            } // end database query loop
        }
    });

    Ok(Backlog { db })
}
