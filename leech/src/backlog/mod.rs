//! This modules handles all backlog tasks

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;

use log::{debug, error, info, warn};
use rorm::{delete, insert, query, Database};
use tonic::transport::Endpoint;
use uuid::Uuid;

use crate::config::KrakenConfig;
use crate::models::{
    BruteforceSubdomainsResult, BruteforceSubdomainsResultInsert, DnsRecordType, TcpPortScanResult,
    TcpPortScanResultInsert,
};
use crate::rpc::rpc_attacks::backlog_service_client::BacklogServiceClient;
use crate::rpc::rpc_attacks::shared::dns_record::Record;
use crate::rpc::rpc_attacks::{
    BacklogBruteforceSubdomainRequest, BacklogTcpPortScanRequest, BruteforceSubdomainRequest,
    BruteforceSubdomainResponse, TcpPortScanRequest,
};

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

const KRAKEN_RETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);
const DB_QUERY_INTERVAL: Duration = Duration::from_secs(10);
const DB_QUERY_LIMIT: u64 = 1000;

/// Starts the backlog upload server
pub async fn start_backlog(db: Database, kraken_config: &KrakenConfig) -> Result<Backlog, String> {
    let kraken_endpoint = Endpoint::from_str(kraken_config.kraken_uri.as_str())
        .map_err(|e| format!("error creating endpoint: {e}"))?;

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

                if let Ok(data) = query!(&mut db_trx, BruteforceSubdomainsResult)
                    .limit(DB_QUERY_LIMIT)
                    .all()
                    .await
                {
                    if !data.is_empty() {
                        data_changed = true;
                        if let Err(e) = delete!(&mut db_trx, BruteforceSubdomainsResult)
                            .bulk(&data)
                            .await
                        {
                            warn!("bulk delete failed: {e}");
                        };

                        let data: BacklogBruteforceSubdomainRequest = data.into();
                        if let Err(e) = kraken.bruteforce_subdomains(data).await {
                            error!("could not send data to kraken: {e}. Restarting connection");
                            break;
                        }
                    };
                }; // end BruteforceSubdomainsResult

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
