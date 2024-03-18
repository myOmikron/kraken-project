//! This module implements all attacks

use std::collections::HashSet;
use std::error::Error as StdError;
use std::future::Future;

use chrono::DateTime;
use chrono::Utc;
use dehashed_rs::Query;
use dehashed_rs::ScheduledRequest;
use futures::TryStreamExt;
use ipnetwork::IpNetwork;
use kraken_proto::InvalidArgumentError;
use log::error;
use rorm::and;
use rorm::prelude::*;
use rorm::query;
use rorm::update;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tonic::Response;
use tonic::Status;
use tonic::Streaming;
use uuid::Uuid;

use crate::api::handler::attacks::schema::DomainOrNetwork;
use crate::api::handler::attacks::schema::PortOrRange;
use crate::api::handler::attacks::schema::SimpleAttack;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::Attack;
use crate::models::AttackType;
use crate::models::Domain;
use crate::models::DomainCertainty;
use crate::models::DomainHostRelation;
use crate::models::InsertAttackError;
use crate::models::User;
use crate::models::Workspace;

mod bruteforce_subdomains;
mod certificate_transparency;
mod dehashed_query;
mod dns_resolution;
mod dns_txt_scan;
mod host_alive;
mod os_detection;
mod service_detection;
mod udp_service_detection;

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

/// The parameters of a "dns resolution" attack
pub struct DnsTxtScanParams {
    /// The domains to resolve
    pub targets: Vec<String>,
}
/// Start a "dns resolution" attack
pub async fn start_dns_txt_scan(
    workspace: Uuid,
    user: Uuid,
    leech: LeechClient,
    params: DnsTxtScanParams,
) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
    let ctx = AttackContext::new(workspace, user, AttackType::DnsTxtScan).await?;
    Ok((
        ctx.attack_uuid,
        tokio::spawn(async move {
            ctx.set_started().await;
            let result = ctx.dns_txt_scan(leech, params).await;
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

/// The parameters of a "OS detection" attack
pub struct OsDetectionParams {
    /// The ip addresses / networks to scan
    pub targets: Vec<DomainOrNetwork>,
    /// set to skip open port detection and use this port for TCP fingerprinting
    pub fingerprint_port: Option<u32>,
    /// set to perform OS detection through SSH header
    pub ssh_port: Option<u32>,
    /// timeout for TCP fingerprint detection task, in ms
    pub fingerprint_timeout: u64,
    /// timeout for establishing an SSH connection, if ssh_port is set, in ms
    pub ssh_connect_timeout: u64,
    /// timeout for the full SSH os detection task, in ms
    pub ssh_timeout: u64,
    /// If fingerprint_port is not set, timeout for each port how long to wait for ACKs
    pub port_ack_timeout: u64,
    /// If fingerprint_port is not set, maximum parallel TCP SYN requests
    pub port_parallel_syns: u32,
    /// The concurrent host scan limit
    pub concurrent_limit: u32,
}
/// Start a "OS detection" attack
pub async fn start_os_detection(
    workspace: Uuid,
    user: Uuid,
    leech: LeechClient,
    params: crate::modules::attacks::OsDetectionParams,
) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
    let ctx = AttackContext::new(workspace, user, AttackType::OSDetection).await?;
    Ok((
        ctx.attack_uuid,
        tokio::spawn(async move {
            ctx.set_started().await;
            let result = ctx.os_detection(leech, params).await;
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
    /// The ip addresses / networks to scan
    pub targets: Vec<DomainOrNetwork>,

    /// List of single ports and port ranges
    pub ports: Vec<PortOrRange>,

    /// The time to wait until a connection is considered failed.
    ///
    /// The timeout is specified in milliseconds.
    pub connect_timeout: u64,

    /// Time to wait for a response after sending the payload
    /// (or after establishing a connection, if not payload is to be sent)
    ///
    /// The timeout is specified in milliseconds.
    pub receive_timeout: u64,

    /// The number of times the connection should be retried if it failed.
    pub max_retries: u32,

    /// The interval that should be wait between retries on a port.
    ///
    /// The interval is specified in milliseconds.
    pub retry_interval: u64,

    /// The concurrent task limit
    pub concurrent_limit: u32,

    /// Skips the initial icmp check.
    ///
    /// All hosts are assumed to be reachable
    pub skip_icmp_check: bool,
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

/// The parameters of a "service detection" attack
pub struct UdpServiceDetectionParams {
    /// The ip addresses / networks to scan
    pub targets: Vec<DomainOrNetwork>,

    /// List of single ports and port ranges
    pub ports: Vec<PortOrRange>,

    /// Time to wait for a response after sending the payload
    /// (or after establishing a connection, if not payload is to be sent)
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
}
/// Start a "service detection" attack
pub async fn start_udp_service_detection(
    workspace: Uuid,
    user: Uuid,
    leech: LeechClient,
    params: UdpServiceDetectionParams,
) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
    let ctx = AttackContext::new(workspace, user, AttackType::UdpServiceDetection).await?;
    Ok((
        ctx.attack_uuid,
        tokio::spawn(async move {
            ctx.set_started().await;
            let result = ctx.udp_service_detection(leech, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// Collection of uuids required for a running attack
#[derive(Debug, Clone)]
pub(crate) struct AttackContext {
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
    pub(crate) async fn new(
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

    /// Query the context for an existing attack
    pub(crate) async fn existing(attack_uuid: Uuid) -> Result<Option<Self>, rorm::Error> {
        let Some((attack_type, started_at, user, uuid, name, description, created_at, owner)) =
            query!(
                &GLOBAL.db,
                (
                    Attack::F.attack_type,
                    Attack::F.created_at,
                    Attack::F.started_by as SimpleUser,
                    Attack::F.workspace.uuid,
                    Attack::F.workspace.name,
                    Attack::F.workspace.description,
                    Attack::F.workspace.created_at,
                    Attack::F.workspace.owner as SimpleUser
                )
            )
            .condition(Attack::F.uuid.equals(attack_uuid))
            .optional()
            .await?
        else {
            return Ok(None);
        };

        Ok(Some(Self {
            user,
            workspace: SimpleWorkspace {
                uuid,
                name,
                description,
                created_at,
                owner,
            },
            attack_uuid,
            created_at: started_at,
            attack_type,
        }))
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
    ///
    /// Returns the attack's uuid
    pub(crate) async fn set_finished(self, result: Result<(), AttackError>) -> Uuid {
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

        self.attack_uuid
    }
}

pub(crate) trait HandleAttackResponse<T> {
    async fn handle_response(&self, response: T) -> Result<(), AttackError>;

    async fn handle_streamed_response(
        &self,
        streamed_response: impl Future<Output = Result<Response<Streaming<T>>, Status>>,
    ) -> Result<(), AttackError> {
        let stream = streamed_response.await?.into_inner();

        stream
            .map_err(AttackError::from)
            .try_for_each(|response| self.handle_response(response))
            .await
    }

    async fn handle_vec_response(&self, vec_response: Vec<T>) -> Result<(), AttackError> {
        for response in vec_response {
            self.handle_response(response).await?;
        }
        Ok(())
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

    /// A malformed grpc message
    #[error("Invalid argument: {0}")]
    InvalidArgument(#[from] InvalidArgumentError),

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
