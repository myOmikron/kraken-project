use std::net::SocketAddr;

use chrono::{DateTime, NaiveDateTime, Utc};
use log::{error, info};
use rorm::prelude::ForeignModelByField;
use rorm::{and, insert, query, Database, FieldAccess, Model};
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status, Streaming};
use uuid::Uuid;

use crate::config::Config;
use crate::models::{
    AttackInsert, AttackType, CertificateTransparencyResultInsert,
    CertificateTransparencyValueNameInsert, LeechApiKey, Workspace, WorkspaceMember,
};
use crate::rpc::definitions::rpc_definitions::attack_results_service_server::AttackResultsService;
use crate::rpc::rpc_definitions::attack_results_service_server::AttackResultsServiceServer;
use crate::rpc::rpc_definitions::{
    CertificateTransparencyResult, ResultResponse, SubdomainEnumerationResult,
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

        // Check existence of workspace
        let (owner,) = query!(&mut tx, (Workspace::F.owner,))
            .condition(Workspace::F.uuid.equals(workspace_uuid))
            .optional()
            .await
            .map_err(status_from_database)?
            .ok_or(Status::new(Code::NotFound, "Unknown workspace"))?;

        // Check if user is owner or member
        if *owner.key() != *user.key() {
            query!(&mut tx, (WorkspaceMember::F.id,))
                .condition(and!(
                    WorkspaceMember::F.member.equals(user_uuid),
                    WorkspaceMember::F.workspace.equals(workspace_uuid)
                ))
                .optional()
                .await
                .map_err(status_from_database)?
                .ok_or(Status::new(
                    Code::PermissionDenied,
                    "You're not part of this workspace",
                ))?;
        }

        let attack_uuid = insert!(&mut tx, AttackInsert)
            .return_primary_key()
            .single(&AttackInsert {
                uuid: Uuid::new_v4(),
                attack_type: AttackType::QueryCertificateTransparency,
                started_by: ForeignModelByField::Key(user_uuid),
                workspace: ForeignModelByField::Key(workspace_uuid),
                finished_at: Some(Utc::now()),
            })
            .await
            .map_err(status_from_database)?;

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
            .add_service(AttackResultsServiceServer::new(Results { db }))
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
