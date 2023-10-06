use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, Utc};
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use log::{debug, error, info, warn};
use rorm::prelude::ForeignModelByField;
use rorm::{and, insert, query, Database, FieldAccess, Model};
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status, Streaming};
use uuid::Uuid;

use crate::config::Config;
use crate::models::{
    Attack, AttackType, BruteforceSubdomainsResult, BruteforceSubdomainsResultInsert,
    CertificateTransparencyResultInsert, CertificateTransparencyValueNameInsert, DnsRecordType,
    InsertAttackError, LeechApiKey, TcpPortScanResult, TcpPortScanResultInsert, Workspace,
};
use crate::rpc::definitions::rpc_definitions::attack_results_service_server::AttackResultsService;
use crate::rpc::rpc_definitions::attack_results_service_server::AttackResultsServiceServer;
use crate::rpc::rpc_definitions::backlog_service_server::{BacklogService, BacklogServiceServer};
use crate::rpc::rpc_definitions::shared::address::Address;
use crate::rpc::rpc_definitions::shared::dns_record::Record;
use crate::rpc::rpc_definitions::{
    BacklogBruteforceSubdomainRequest, BacklogTcpPortScanRequest, CertificateTransparencyResult,
    EmptyResponse, ResultResponse, SubdomainEnumerationResult,
};

/// Helper type to implement result handler to
pub struct Results {
    db: Database,
}

#[tonic::async_trait]
impl AttackResultsService for Results {
    async fn certificate_transparency(
        &self,
        request: Request<CertificateTransparencyResult>,
    ) -> Result<Response<ResultResponse>, Status> {
        let req = request.into_inner();
        let attack_info = req
            .attack_info
            .ok_or(Status::new(Code::Unknown, "Missing attack_info"))?;
        let workspace_uuid = Uuid::try_parse(&attack_info.workspace_uuid).unwrap();

        let mut tx = self.db.start_transaction().await.unwrap();

        // Check api key and get user
        let (user,) = query!(&mut tx, (LeechApiKey::F.user,))
            .condition(LeechApiKey::F.key.equals(attack_info.api_key))
            .optional()
            .await
            .map_err(status_from_database)?
            .ok_or(Status::new(Code::Unauthenticated, "Invalid api key"))?;
        let user_uuid = *user.key();

        if !Workspace::is_user_member_or_owner(&mut tx, workspace_uuid, user_uuid)
            .await
            .map_err(status_from_database)?
        {
            return Err(Status::permission_denied(
                "You are not part of this workspace",
            ));
        }

        let attack_uuid = Attack::insert(
            &mut tx,
            AttackType::QueryCertificateTransparency,
            user_uuid,
            workspace_uuid,
        )
        .await
        .map_err(|e| match e {
            InsertAttackError::DatabaseError(x) => status_from_database(x),
            InsertAttackError::WorkspaceInvalid => Status::internal("Workspace does not exist"),
        })?;

        for cert_entry in req.entries {
            let entry_uuid = insert!(&mut tx, CertificateTransparencyResultInsert)
                .return_primary_key()
                .single(&CertificateTransparencyResultInsert {
                    uuid: Uuid::new_v4(),
                    attack: ForeignModelByField::Key(attack_uuid),
                    created_at: Utc::now(),
                    issuer_name: cert_entry.issuer_name,
                    common_name: cert_entry.common_name,
                    not_before: cert_entry.not_before.map(|ts| {
                        DateTime::from_naive_utc_and_offset(
                            NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32).unwrap(),
                            Utc,
                        )
                    }),
                    not_after: cert_entry.not_after.map(|ts| {
                        DateTime::from_naive_utc_and_offset(
                            NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32).unwrap(),
                            Utc,
                        )
                    }),
                    serial_number: cert_entry.serial_number,
                })
                .await
                .map_err(status_from_database)?;

            insert!(&mut tx, CertificateTransparencyValueNameInsert)
                .bulk(
                    &cert_entry
                        .value_names
                        .into_iter()
                        .map(|x| CertificateTransparencyValueNameInsert {
                            uuid: Uuid::new_v4(),
                            value_name: x,
                            ct_result: ForeignModelByField::Key(entry_uuid),
                        })
                        .collect::<Vec<_>>(),
                )
                .await
                .map_err(status_from_database)?;
        }

        tx.commit().await.map_err(status_from_database)?;

        Ok(Response::new(ResultResponse {
            uuid: Uuid::new_v4().to_string(),
        }))
    }

    async fn subdomain_enumeration(
        &self,
        _request: Request<Streaming<SubdomainEnumerationResult>>,
    ) -> Result<Response<ResultResponse>, Status> {
        Ok(Response::new(ResultResponse {
            uuid: Uuid::new_v4().to_string(),
        }))
    }
}

#[tonic::async_trait]
impl BacklogService for Results {
    async fn bruteforce_subdomains(
        &self,
        request: Request<BacklogBruteforceSubdomainRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        let Ok(mut db_trx) = self.db.start_transaction().await else {
            error!("could not start batch processing");
            return Err(Status::internal("internal server error"));
        };

        for entry in request.into_inner().entries {
            let Ok(req_attack_uuid) = Uuid::from_str(&entry.attack_uuid) else {
                error!("could not get attack uuid from request");
                continue;
            };

            let Some(record) = entry.record else {
                // nothing to insert
                continue;
            };

            let Some(record) = record.record else {
                // nothing to insert
                continue;
            };

            let (source, destination, dns_record_type) = match record {
                Record::A(v) => {
                    let Some(to) = v.to else {
                        warn!("missing destination address");
                        continue;
                    };
                    (v.source, Ipv4Addr::from(to).to_string(), DnsRecordType::A)
                }
                Record::Aaaa(v) => {
                    let Some(to) = v.to else {
                        warn!("missing destination address");
                        continue;
                    };
                    (
                        v.source,
                        Ipv6Addr::from(to).to_string(),
                        DnsRecordType::Aaaa,
                    )
                }
                Record::Cname(v) => (v.source, v.to, DnsRecordType::Cname),
            };

            if query!(&mut db_trx, Attack)
                .condition(Attack::F.uuid.equals(req_attack_uuid))
                .one()
                .await
                .is_err()
            {
                debug!("attack does not exist");
                continue;
            }

            let Ok(None) = query!(&mut db_trx, BruteforceSubdomainsResult)
                .condition(and!(
                    BruteforceSubdomainsResult::F
                        .attack
                        .equals(&req_attack_uuid),
                    BruteforceSubdomainsResult::F
                        .dns_record_type
                        .equals(dns_record_type.clone()),
                    BruteforceSubdomainsResult::F.source.equals(&source),
                    BruteforceSubdomainsResult::F
                        .destination
                        .equals(&destination)
                ))
                .optional()
                .await
            else {
                debug!("entry already exists");
                continue;
            };

            if let Err(e) = insert!(&mut db_trx, BruteforceSubdomainsResultInsert)
                .single(&BruteforceSubdomainsResultInsert {
                    uuid: Uuid::new_v4(),
                    attack: ForeignModelByField::Key(req_attack_uuid),
                    source,
                    destination,
                    dns_record_type,
                })
                .await
            {
                error!("could not insert into database: {e}");
                continue;
            }
        }

        if let Err(e) = db_trx.commit().await {
            error!("could not commit to database: {e}");
            return Err(Status::internal("internal server error"));
        }

        Ok(Response::new(EmptyResponse {}))
    }

    async fn tcp_port_scan(
        &self,
        request: Request<BacklogTcpPortScanRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        let Ok(mut db_trx) = self.db.start_transaction().await else {
            error!("could not start batch processing");
            return Err(Status::internal("internal server error"));
        };

        for entry in request.into_inner().entries {
            let Ok(req_attack_uuid) = Uuid::from_str(&entry.attack_uuid) else {
                error!("could not get attack uuid from request");
                continue;
            };

            if query!(&mut db_trx, Attack)
                .condition(Attack::F.uuid.equals(req_attack_uuid))
                .one()
                .await
                .is_err()
            {
                debug!("attack does not exist");
                continue;
            }

            let Some(address) = entry.address else {
                warn!("no address");
                continue;
            };

            let Some(address) = address.address else {
                warn!("no address");
                continue;
            };

            let address = match address {
                Address::Ipv4(v) => IpNetwork::V4(Ipv4Network::from(Ipv4Addr::from(v))),
                Address::Ipv6(v) => IpNetwork::V6(Ipv6Network::from(Ipv6Addr::from(v))),
            };

            if let Ok(None) = query!(&mut db_trx, TcpPortScanResult)
                .condition(and!(
                    TcpPortScanResult::F.attack.equals(req_attack_uuid),
                    TcpPortScanResult::F.port.equals(entry.port as i32),
                    TcpPortScanResult::F.address.equals(address),
                ))
                .optional()
                .await
            {
                debug!("entry already exists");
                continue;
            };

            if let Err(e) = insert!(&mut db_trx, TcpPortScanResultInsert)
                .single(&TcpPortScanResultInsert {
                    uuid: Uuid::new_v4(),
                    attack: ForeignModelByField::Key(req_attack_uuid),
                    address,
                    port: entry.port as i32,
                })
                .await
            {
                error!("could not insert into database: {e}");
                continue;
            }
        }

        if let Err(e) = db_trx.commit().await {
            error!("could not commit to database: {e}");
            return Err(Status::internal("internal server error"));
        }

        Ok(Response::new(EmptyResponse {}))
    }
}

/// Starts the gRPC server
///
/// **Parameter**:
/// - `config`: Reference to [Config]
pub fn start_rpc_server(config: &Config, db: Database) -> Result<(), String> {
    let listen_address = config.server.rpc_listen_address.parse().unwrap();
    let listen_port = config.server.rpc_listen_port;

    tokio::spawn(async move {
        info!("Starting gRPC server");
        if let Err(err) = Server::builder()
            .add_service(AttackResultsServiceServer::new(Results { db: db.clone() }))
            .add_service(BacklogServiceServer::new(Results { db }))
            .serve(SocketAddr::new(listen_address, listen_port))
            .await
        {
            // TODO: add loop to continuously restart the gRPC server
            error!("Error running gRPC server: {err}");
        }
    });
    Ok(())
}

/// Convert [`rorm::Error`] to [`tonic::Status`]
fn status_from_database(err: rorm::Error) -> Status {
    error!("Database error in rpc endpoint: {err}");
    Status::new(Code::Internal, "Database error")
}
