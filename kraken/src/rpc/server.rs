//! Implementation of the [`AttackResultsService`] and [`BacklogService`]
//!
//! As well as the [`start_rpc_server`] starting both.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::{AddrParseError, SocketAddr};
use std::str::FromStr;

use log::{error, info, warn};
use rorm::{query, FieldAccess, Model};
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status, Streaming};
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::config::Config;
use crate::models::{Attack, AttackType, InsertAttackError, Leech, LeechApiKey, Workspace};
use crate::modules::attack_results::store_query_certificate_transparency_result;
use crate::modules::attacks::{AttackContext, HandleAttackResponse};
use crate::rpc::definitions::rpc_definitions::attack_results_service_server::AttackResultsService;
use crate::rpc::rpc_definitions::attack_results_service_server::AttackResultsServiceServer;
use crate::rpc::rpc_definitions::backlog_service_server::{BacklogService, BacklogServiceServer};
use crate::rpc::rpc_definitions::{
    any_attack_response, AnyAttackResponse, BacklogRequest, BacklogResponse,
    CertificateTransparencyResult, ResultResponse, SubdomainEnumerationResult,
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
    async fn submit_backlog(
        &self,
        request: Request<BacklogRequest>,
    ) -> Result<Response<BacklogResponse>, Status> {
        auth_leech(&request).await?;

        let mut attack_cache = HashMap::new();

        let entries = request.into_inner().responses;
        for entry in entries {
            let AnyAttackResponse {
                attack_uuid,
                response: Some(response),
            } = entry
            else {
                continue;
            };

            let Ok(attack_uuid) = Uuid::from_str(&attack_uuid) else {
                error!("Malformed attack uuid: {attack_uuid}");
                continue;
            };

            let Some(attack_context) = (match attack_cache.entry(attack_uuid) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(
                    AttackContext::existing(attack_uuid)
                        .await
                        .map_err(status_from_database)?,
                ),
            })
            .as_ref() else {
                warn!("Unknown attack uuid: {attack_uuid}");
                continue;
            };

            let result: Result<(), _> = match response {
                any_attack_response::Response::DnsResolution(response) => {
                    attack_context.handle_response(response).await
                }
                any_attack_response::Response::HostsAlive(response) => {
                    attack_context.handle_response(response).await
                }
                any_attack_response::Response::TcpPortScan(response) => {
                    attack_context.handle_response(response).await
                }
                any_attack_response::Response::BruteforceSubdomain(response) => {
                    attack_context.handle_response(response).await
                }
                any_attack_response::Response::ServiceDetection(_)
                | any_attack_response::Response::CertificateTransparency(_) => {
                    return Err(Status::unimplemented("Attack type is not implemented yet"));
                }
            };

            if let Err(error) = result {
                error!("Backlog entry failed: {error}");
            }
        }

        Ok(Response::new(BacklogResponse {}))
    }
}

/// Authenticates a leech by checking the `x-leech-secret` header.
pub async fn auth_leech<T>(request: &Request<T>) -> Result<(), Status> {
    let secret = request
        .metadata()
        .get("x-leech-secret")
        .ok_or_else(|| Status::unauthenticated("Missing `x-leech-secret` header"))?;
    let secret = secret
        .to_str()
        .map_err(|_| Status::unauthenticated("Invalid `x-leech-secret`"))?;
    query!(&GLOBAL.db, (Leech::F.uuid,))
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
