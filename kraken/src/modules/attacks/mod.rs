//! This module implements all attacks

use std::collections::HashSet;
use std::error::Error as StdError;
use std::net::IpAddr;

use chrono::{DateTime, Utc};
use dehashed_rs::{Query, ScheduledRequest};
use futures::{TryFuture, TryStreamExt};
use ipnetwork::IpNetwork;
use log::error;
use rorm::prelude::*;
use rorm::{and, query, update};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tonic::{Response, Status, Streaming};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::attacks::{PortOrRange, SimpleAttack};
use crate::api::handler::users::SimpleUser;
use crate::api::handler::workspaces::SimpleWorkspace;
use crate::chan::{LeechClient, WsMessage, GLOBAL};
use crate::models::{
    Attack, AttackType, Domain, DomainCertainty, DomainHostRelation, InsertAttackError, User,
    Workspace,
};
use crate::rpc::rpc_definitions::AddressConvError;

mod bruteforce_subdomains;
mod certificate_transparency;
mod dehashed_query;
mod dns_resolution;
mod host_alive;
mod service_detection;
mod tcp_port_scan;

/// The parameters of a "bruteforce subdomains" attack
pub struct BruteforceSubdomainsParams {
    /// Domain to construct subdomains for
    pub target: String,

    /// The wordlist to use
    pub wordlist_path: String,

    /// The concurrent task limit
    pub concurrent_limit: u32,
}
/// Start a "bruteforce subdomains" attack
pub async fn start_bruteforce_subdomains(
    workspace: Uuid,
    user: Uuid,
    leech: LeechClient,
    params: BruteforceSubdomainsParams,
) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
    let ctx = AttackContext::new(workspace, user, AttackType::BruteforceSubdomains).await?;
    Ok((
        ctx.attack_uuid,
        tokio::spawn(async move {
            ctx.set_started().await;
            let result = ctx.bruteforce_subdomains(leech, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// The parameters of a "dns resolution" attack
pub struct DnsResolutionParams {
    /// The domains to resolve
    pub targets: Vec<String>,

    /// The concurrent task limit
    pub concurrent_limit: u32,
}
/// Start a "dns resolution" attack
pub async fn start_dns_resolution(
    workspace: Uuid,
    user: Uuid,
    leech: LeechClient,
    params: DnsResolutionParams,
) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
    let ctx = AttackContext::new(workspace, user, AttackType::DnsResolution).await?;
    Ok((
        ctx.attack_uuid,
        tokio::spawn(async move {
            ctx.set_started().await;
            let result = ctx.dns_resolution(leech, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// The parameters of a "host alive" attack
pub struct HostAliveParams {
    /// The ip addresses / networks to scan
    pub targets: Vec<DomainOrNetwork>,

    /// The time to wait until a host is considered down.
    ///
    /// The timeout is specified in milliseconds.
    pub timeout: u64,

    /// The concurrent task limit
    pub concurrent_limit: u32,
}
/// Start a "host alive" attack
pub async fn start_host_alive(
    workspace: Uuid,
    user: Uuid,
    leech: LeechClient,
    params: HostAliveParams,
) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
    let ctx = AttackContext::new(workspace, user, AttackType::HostAlive).await?;
    Ok((
        ctx.attack_uuid,
        tokio::spawn(async move {
            ctx.set_started().await;
            let result = ctx.host_alive(leech, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// The parameters of a "certificate transparency" attack
pub struct CertificateTransparencyParams {
    /// Domain to query certificates for
    pub target: String,

    /// Should expired certificates be included as well
    pub include_expired: bool,

    /// The number of times the query should be retried if it failed.
    pub max_retries: u32,

    /// The interval that should be waited between retries.
    ///
    /// The interval is specified in milliseconds.
    pub retry_interval: u64,
}
/// Start a "certificate transparency" attack
pub async fn start_certificate_transparency(
    workspace: Uuid,
    user: Uuid,
    leech: LeechClient,
    params: CertificateTransparencyParams,
) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
    let ctx = AttackContext::new(workspace, user, AttackType::QueryCertificateTransparency).await?;
    Ok((
        ctx.attack_uuid,
        tokio::spawn(async move {
            ctx.set_started().await;
            let result = ctx.certificate_transparency(leech, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// The parameters of a "dehashed query" attack
pub struct DehashedQueryParams {
    /// The query to send to dehashed
    pub query: Query,
}
/// Start a "dehashed query" attack
pub async fn start_dehashed_query(
    workspace: Uuid,
    user: Uuid,
    sender: mpsc::Sender<ScheduledRequest>,
    params: DehashedQueryParams,
) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
    let ctx = AttackContext::new(workspace, user, AttackType::QueryUnhashed).await?;
    Ok((
        ctx.attack_uuid,
        tokio::spawn(async move {
            ctx.set_started().await;
            let result = ctx.dehashed_query(sender, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// The parameters of a "service detection" attack
pub struct ServiceDetectionParams {
    /// The ip address the service listens on
    pub target: IpAddr,

    /// The port the service listens on
    pub port: u16,

    /// Time to wait for a response after sending the payload
    /// (or after establishing a connection, if not payload is to be sent)
    ///
    /// The timeout is specified in milliseconds.
    pub timeout: u64,
}
/// Start a "service detection" attack
pub async fn start_service_detection(
    workspace: Uuid,
    user: Uuid,
    leech: LeechClient,
    params: ServiceDetectionParams,
) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
    let ctx = AttackContext::new(workspace, user, AttackType::ServiceDetection).await?;
    Ok((
        ctx.attack_uuid,
        tokio::spawn(async move {
            ctx.set_started().await;
            let result = ctx.service_detection(leech, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// The parameters of a "tcp port scan" attack
pub struct TcpPortScanParams {
    /// The ip addresses / networks to scan
    pub targets: Vec<DomainOrNetwork>,

    /// List of single ports and port ranges
    pub ports: Vec<PortOrRange>,

    /// The time to wait until a connection is considered failed.
    ///
    /// The timeout is specified in milliseconds.
    pub timeout: u64,

    /// The concurrent task limit
    pub concurrent_limit: u32,

    /// The number of times the connection should be retried if it failed.
    pub max_retries: u32,

    /// The interval that should be wait between retries on a port.
    ///
    /// The interval is specified in milliseconds.
    pub retry_interval: u64,

    /// Skips the initial icmp check.
    ///
    /// All hosts are assumed to be reachable
    pub skip_icmp_check: bool,
}
/// Start a "tcp port scan" attack
pub async fn start_tcp_port_scan(
    workspace: Uuid,
    user: Uuid,
    leech: LeechClient,
    params: TcpPortScanParams,
) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
    let ctx = AttackContext::new(workspace, user, AttackType::TcpPortScan).await?;
    Ok((
        ctx.attack_uuid,
        tokio::spawn(async move {
            ctx.set_started().await;
            let result = ctx.tcp_port_scan(leech, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// Collection of uuids required for a running attack
#[derive(Clone)]
struct AttackContext {
    /// The user who started the attack
    user: SimpleUser,

    /// The workspace the attack was started in
    workspace: SimpleWorkspace,

    /// The attack's uuid
    attack_uuid: Uuid,

    /// The point in time when this attack was created
    created_at: DateTime<Utc>,

    /// The type of the attack
    attack_type: AttackType,
}

impl AttackContext {
    /// Insert a new attack in the database and bundle all uuids together in one struct
    async fn new(
        workspace_uuid: Uuid,
        user_uuid: Uuid,
        attack_type: AttackType,
    ) -> Result<Self, InsertAttackError> {
        let mut tx = GLOBAL.db.start_transaction().await?;

        let user = query!(&mut tx, SimpleUser)
            .condition(User::F.uuid.equals(user_uuid))
            .optional()
            .await?
            .ok_or(InsertAttackError::UserInvalid)?;
        let (name, description, created_at, owner) = query!(
            &mut tx,
            (
                Workspace::F.name,
                Workspace::F.description,
                Workspace::F.created_at,
                Workspace::F.owner as SimpleUser
            )
        )
        .condition(Workspace::F.uuid.equals(workspace_uuid))
        .optional()
        .await?
        .ok_or(InsertAttackError::WorkspaceInvalid)?;

        tx.commit().await?;

        let workspace = SimpleWorkspace {
            uuid: workspace_uuid,
            name,
            description,
            created_at,
            owner,
        };

        let attack = Attack::insert(&GLOBAL.db, attack_type, user_uuid, workspace_uuid).await?;

        Ok(Self {
            user,
            attack_type,
            workspace,
            attack_uuid: attack.uuid,
            created_at: attack.created_at,
        })
    }

    /// Send a websocket message and log the error
    async fn send_ws(&self, message: WsMessage) {
        GLOBAL
            .ws
            .message_workspace(self.workspace.uuid, message)
            .await;
    }

    /// Send the user a notification
    async fn set_started(&self) {
        self.send_ws(WsMessage::AttackStarted {
            attack: SimpleAttack {
                uuid: self.attack_uuid,
                workspace: self.workspace.clone(),
                attack_type: self.attack_type,
                started_by: self.user.clone(),
                created_at: self.created_at,
                error: None,
                finished_at: None,
            },
        })
        .await;
    }

    /// Send the user a notification and update the [`Attack`] model
    async fn set_finished(self, result: Result<(), AttackError>) {
        let now = Utc::now();

        self.send_ws(WsMessage::AttackFinished {
            attack: SimpleAttack {
                uuid: self.attack_uuid,
                workspace: self.workspace.clone(),
                attack_type: self.attack_type,
                created_at: self.created_at,
                finished_at: Some(now),
                error: result.as_ref().err().map(|x| x.to_string()),
                started_by: self.user.clone(),
            },
        })
        .await;

        if let Err(error) = result.as_ref() {
            error!(
                "Attack {attack_uuid} failed: {error}",
                attack_uuid = self.attack_uuid
            );
        }

        if let Err(err) = update!(&GLOBAL.db, Attack)
            .condition(Attack::F.uuid.equals(self.attack_uuid))
            .set(Attack::F.finished_at, Some(now))
            .set(
                Attack::F.error,
                result.err().map(|err| {
                    let mut string = err.to_string();
                    for (char_index, (byte_index, _)) in string.char_indices().enumerate() {
                        if char_index == 256 {
                            string.truncate(byte_index);
                            break;
                        }
                    }
                    string
                }),
            )
            .exec()
            .await
        {
            error!(
                "Failed to set the attack {attack_uuid} to finished: {err}",
                attack_uuid = self.attack_uuid
            );
        }
    }

    async fn handle_streamed_response<T, Fut>(
        streamed_response: Result<Response<Streaming<T>>, Status>,
        handler: impl FnMut(T) -> Fut,
    ) -> Result<(), AttackError>
    where
        Fut: TryFuture<Ok = (), Error = AttackError>,
    {
        let stream = streamed_response?.into_inner();

        stream
            .map_err(AttackError::from)
            .try_for_each(handler)
            .await
    }
}

/// An error occurring during an attack which is logged and stored on the db
#[derive(Error, Debug)]
pub enum AttackError {
    /// An error returned by grpc i.e. a [`Status`]
    #[error("GRPC: {:?}, {:?}", .0.code(), .0.message())]
    Grpc(#[from] Status),

    /// An error produced by the database
    #[error("DB: {0}")]
    Database(#[from] rorm::Error),

    /// A malformed grpc message
    ///
    /// For example "optional" fields which have to be set
    #[error("Malformed response: {0}")]
    Malformed(&'static str),

    /// An error produced by address conversion
    #[error("Error during address conversion:  {0}")]
    AddressConv(#[from] AddressConvError),

    /// Catch all variant for everything else
    #[error("{0}")]
    Custom(Box<dyn StdError + Send + Sync>),
}
impl From<InsertAttackError> for AttackError {
    fn from(value: InsertAttackError) -> Self {
        match value {
            InsertAttackError::DatabaseError(error) => Self::Database(error),
            InsertAttackError::WorkspaceInvalid => Self::Custom(Box::new(value)),
            InsertAttackError::UserInvalid => Self::Custom(Box::new(value)),
        }
    }
}

/// Either an ip address / network or a domain name
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum DomainOrNetwork {
    /// A ip address / network
    #[schema(value_type = String, example = "10.13.37.10")]
    Network(IpNetwork),

    /// A domain name
    #[schema(value_type = String, example = "kraken.test")]
    Domain(String),
}
impl DomainOrNetwork {
    /// Takes a list of [`DomainOrNetwork`] and produces it into a list of [`IpNetwork`]
    /// by resolving the domains in a given workspace and starting implicit attacks if necessary.
    pub async fn resolve(
        workspace: Uuid,
        user: Uuid,
        leech: &LeechClient,
        targets: &[Self],
    ) -> Result<HashSet<IpNetwork>, AttackError> {
        let mut ips = HashSet::new();
        for domain_or_network in targets {
            match domain_or_network {
                Self::Network(network) => {
                    ips.insert(*network);
                }
                Self::Domain(domain) => {
                    let certainty = query!(&GLOBAL.db, (Domain::F.certainty))
                        .condition(and![
                            Domain::F.workspace.equals(workspace),
                            Domain::F.domain.equals(domain)
                        ])
                        .optional()
                        .await?;
                    if certainty != Some((DomainCertainty::Verified,)) {
                        let (_, attack) = start_dns_resolution(
                            workspace,
                            user,
                            leech.clone(),
                            DnsResolutionParams {
                                targets: vec![domain.to_string()],
                                concurrent_limit: 1,
                            },
                        )
                        .await?;
                        let _ = attack.await;
                    }

                    query!(&GLOBAL.db, (DomainHostRelation::F.host.ip_addr,))
                        .condition(and![
                            DomainHostRelation::F.workspace.equals(workspace),
                            DomainHostRelation::F.domain.domain.equals(domain)
                        ])
                        .stream()
                        .try_for_each(|(ip,)| {
                            ips.insert(ip);
                            async { Ok(()) }
                        })
                        .await?;
                }
            }
        }
        Ok(ips)
    }
}
