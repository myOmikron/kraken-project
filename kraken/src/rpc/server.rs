//! Implementation of the [`AttackResultsService`] and [`BacklogService`]
//!
//! As well as the [`start_rpc_server`] starting both.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use log::{error, info, warn};
use rorm::db::Executor;
use rorm::{query, Database, FieldAccess, Model};
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status, Streaming};
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::config::Config;
use crate::models::{
    Attack, AttackType, DnsRecordType, InsertAttackError, Leech, LeechApiKey, Workspace,
};
use crate::modules::attack_results::{
    store_dns_resolution_result, store_host_alive_check_result,
    store_query_certificate_transparency_result, store_tcp_port_scan_result,
};
use crate::rpc::definitions::rpc_definitions::attack_results_service_server::AttackResultsService;
use crate::rpc::rpc_definitions::attack_results_service_server::AttackResultsServiceServer;
use crate::rpc::rpc_definitions::backlog_service_server::{BacklogService, BacklogServiceServer};
use crate::rpc::rpc_definitions::shared::address::Address;
use crate::rpc::rpc_definitions::shared::dns_record::Record;
use crate::rpc::rpc_definitions::{
    BacklogDnsRequest, BacklogHostAliveRequest, BacklogTcpPortScanRequest,
    CertificateTransparencyResult, EmptyResponse, ResultResponse, SubdomainEnumerationResult,
};

/// Helper type to implement result handler to
pub struct Results;

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
        let workspace_uuid = Uuid::try_parse(&attack_info.workspace_uuid)
            .map_err(|_| Status::new(Code::Internal, "Invalid UUID supplied"))?;

        let mut tx = GLOBAL
            .db
            .start_transaction()
            .await
            .map_err(status_from_database)?;

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

        let attack = Attack::insert(
            &mut tx,
            AttackType::QueryCertificateTransparency,
            user_uuid,
            workspace_uuid,
        )
        .await
        .map_err(|e| match e {
            InsertAttackError::DatabaseError(x) => status_from_database(x),
            InsertAttackError::WorkspaceInvalid => Status::internal("Workspace does not exist"),
            InsertAttackError::UserInvalid => unreachable!("User was queried beforehand"),
        })?;

        for cert_entry in req.entries {
            store_query_certificate_transparency_result(
                &mut tx,
                attack.uuid,
                workspace_uuid,
                cert_entry,
            )
            .await
            .map_err(status_from_database)?
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
        Err(Status::unimplemented("TODO"))
    }
}

#[tonic::async_trait]
impl BacklogService for Results {
    async fn dns_results(
        &self,
        request: Request<BacklogDnsRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        auth_leech(&GLOBAL.db, &request).await?;

        let Ok(mut db_trx) = GLOBAL.db.start_transaction().await else {
            error!("could not start batch processing");
            return Err(Status::internal("internal server error"));
        };

        // Map from attack_uuid to workspace_uuid
        let mut workspaces = WorkspaceCache::default();

        for entry in request.into_inner().entries {
            let Ok(attack_uuid) = Uuid::from_str(&entry.attack_uuid) else {
                error!("could not get attack uuid from request");
                continue;
            };

            let Some(workspace_uuid) = workspaces
                .get(&mut db_trx, attack_uuid)
                .await
                .map_err(status_from_database)?
            else {
                warn!("Got result for unknown attack: {attack_uuid}");
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
                Record::Caa(v) => (v.source, v.to, DnsRecordType::Caa),
                Record::Mx(v) => (v.source, v.to, DnsRecordType::Mx),
                Record::Tlsa(v) => (v.source, v.to, DnsRecordType::Tlsa),
                Record::Txt(v) => (v.source, v.to, DnsRecordType::Txt),
            };

            store_dns_resolution_result(
                &mut db_trx,
                attack_uuid,
                workspace_uuid,
                source,
                destination,
                dns_record_type,
            )
            .await
            .map_err(status_from_database)?;
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
        auth_leech(&GLOBAL.db, &request).await?;

        let Ok(mut db_trx) = GLOBAL.db.start_transaction().await else {
            error!("could not start batch processing");
            return Err(Status::internal("internal server error"));
        };

        // Map from attack_uuid to workspace_uuid
        let mut workspaces = WorkspaceCache::default();

        for entry in request.into_inner().entries {
            let Ok(attack_uuid) = Uuid::from_str(&entry.attack_uuid) else {
                error!("could not get attack uuid from request");
                continue;
            };

            let Some(workspace_uuid) = workspaces
                .get(&mut db_trx, attack_uuid)
                .await
                .map_err(status_from_database)?
            else {
                warn!("Got result for unknown attack: {attack_uuid}");
                continue;
            };

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

            store_tcp_port_scan_result(
                &mut db_trx,
                attack_uuid,
                workspace_uuid,
                address,
                entry.port as u16,
            )
            .await
            .map_err(status_from_database)?;
        }

        if let Err(e) = db_trx.commit().await {
            error!("could not commit to database: {e}");
            return Err(Status::internal("internal server error"));
        }

        Ok(Response::new(EmptyResponse {}))
    }

    async fn host_alive_check(
        &self,
        request: Request<BacklogHostAliveRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        auth_leech(&GLOBAL.db, &request).await?;

        let Ok(mut db_trx) = GLOBAL.db.start_transaction().await else {
            error!("could not start batch processing");
            return Err(Status::internal("internal server error"));
        };

        // Map from attack_uuid to workspace_uuid
        let mut workspaces = WorkspaceCache::default();

        for entry in request.into_inner().entries {
            let Ok(attack_uuid) = Uuid::from_str(&entry.attack_uuid) else {
                error!("could not get attack uuid from request");
                continue;
            };

            let Some(workspace_uuid) = workspaces
                .get(&mut db_trx, attack_uuid)
                .await
                .map_err(status_from_database)?
            else {
                warn!("Got result for unknown attack: {attack_uuid}");
                continue;
            };

            let Some(host) = entry.host else {
                warn!("no host");
                continue;
            };

            let Ok(host): Result<IpAddr, _> = host.try_into() else {
                continue;
            };

            store_host_alive_check_result(&mut db_trx, attack_uuid, workspace_uuid, host.into())
                .await
                .map_err(|err| {
                    error!("Database error in backlog: {err}");
                    Status::internal("database error")
                })?;
        }

        if let Err(e) = db_trx.commit().await {
            error!("could not commit to database: {e}");
            return Err(Status::internal("internal server error"));
        }

        Ok(Response::new(EmptyResponse {}))
    }
}

/// Authenticates a leech by checking the `x-leech-secret` header.
pub async fn auth_leech<T>(db: &Database, request: &Request<T>) -> Result<(), Status> {
    let secret = request
        .metadata()
        .get("x-leech-secret")
        .ok_or_else(|| Status::unauthenticated("Missing `x-leech-secret` header"))?;
    let secret = secret
        .to_str()
        .map_err(|_| Status::unauthenticated("Invalid `x-leech-secret`"))?;
    query!(db, (Leech::F.uuid,))
        .condition(Leech::F.secret.equals(secret))
        .optional()
        .await
        .map_err(status_from_database)?
        .ok_or_else(|| Status::unauthenticated("Invalid `x-leech-secret`"))?;
    Ok(())
}

/// Starts the gRPC server
///
/// **Parameter**:
/// - `config`: Reference to [Config]
///
/// Returns an error if the rpc listen address is invalid
pub fn start_rpc_server(config: &Config) -> Result<(), AddrParseError> {
    let listen_address = config.server.rpc_listen_address.parse()?;
    let listen_port = config.server.rpc_listen_port;
    let tls_config = GLOBAL.tls.tonic_server();

    tokio::spawn(async move {
        info!("Starting gRPC server");
        // TLS config should be valid is it is constructed by our TLS manager
        #[allow(clippy::expect_used)]
        if let Err(err) = Server::builder()
            .tls_config(tls_config)
            .expect("The tls config should be valid")
            .add_service(AttackResultsServiceServer::new(Results))
            .add_service(BacklogServiceServer::new(Results))
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

/// Helper for retrieving an attack's workspace
#[derive(Debug, Default)]
pub struct WorkspaceCache(HashMap<Uuid, Option<Uuid>>);
impl WorkspaceCache {
    /// Get the workspace uuid for a given attack uuid
    ///
    /// Returns `Ok(None)``if the attack does not exist.
    pub async fn get(
        &mut self,
        executor: impl Executor<'_>,
        attack: Uuid,
    ) -> Result<Option<Uuid>, rorm::Error> {
        Ok(match self.0.entry(attack) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let workspace_uuid = query!(executor, (Attack::F.workspace))
                    .condition(Attack::F.uuid.equals(attack))
                    .optional()
                    .await?
                    .map(|(foreign,)| *foreign.key());
                entry.insert(workspace_uuid);
                workspace_uuid
            }
        })
    }
}
