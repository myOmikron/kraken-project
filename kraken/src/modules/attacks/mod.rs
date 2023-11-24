//! This module implements all attacks

mod bruteforce_subdomains;
mod dns_resolution;
mod host_alive_check;
mod query_certificate_transparency;
mod query_dehashed;
mod service_detection;
mod tcp_port_scan;

use std::error::Error as StdError;
use std::fmt;
use std::net::IpAddr;

use chrono::Utc;
use dehashed_rs::{Query, ScheduledRequest};
use futures::{TryFuture, TryStreamExt};
use ipnetwork::IpNetwork;
use log::error;
use rorm::prelude::*;
use rorm::update;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tonic::{Response, Status, Streaming};
use uuid::Uuid;

use crate::api::handler::attacks::PortOrRange;
use crate::chan::{LeechClient, WsMessage, GLOBAL};
use crate::models::{Attack, AttackType, InsertAttackError};
use crate::rpc::rpc_definitions::AddressConvError;

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
            let result = ctx.dns_resolution(leech, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// The parameters of a "host alive" attack
pub struct HostAliveParams {
    /// The ip addresses / networks to scan
    pub targets: Vec<IpNetwork>,

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
            let result = ctx.host_alive_check(leech, params).await;
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
            let result = ctx.query_certificate_transparency(leech, params).await;
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
            let result = ctx.query_dehashed(sender, params).await;
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
            let result = ctx.service_detection(leech, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// The parameters of a "tcp port scan" attack
pub struct TcpPortScanParams {
    /// The ip addresses / networks to scan
    pub targets: Vec<IpNetwork>,

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
            let result = ctx.tcp_port_scan(leech, params).await;
            ctx.set_finished(result).await;
        }),
    ))
}

/// Collection of uuids required for a running attack
#[derive(Clone)]
struct AttackContext {
    /// The user who started the attack
    user_uuid: Uuid,

    /// The workspace the attack was started in
    workspace_uuid: Uuid,

    /// The attack's uuid
    attack_uuid: Uuid,
}

impl AttackContext {
    /// Insert a new attack in the database and bundle all uuids together in one struct
    async fn new(
        workspace_uuid: Uuid,
        user_uuid: Uuid,
        attack_type: AttackType,
    ) -> Result<Self, InsertAttackError> {
        Ok(Self {
            user_uuid,
            workspace_uuid,
            attack_uuid: Attack::insert(&GLOBAL.db, attack_type, user_uuid, workspace_uuid).await?,
        })
    }

    /// Send a websocket message and log the error
    async fn send_ws(&self, message: WsMessage) {
        GLOBAL.ws.message(self.user_uuid, message).await;
    }

    /// Send the user a notification and update the [`Attack`] model
    async fn set_finished(self, result: Result<(), AttackError>) {
        self.send_ws(WsMessage::AttackFinished {
            attack_uuid: self.attack_uuid,
            finished_successful: result.is_ok(),
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
            .set(Attack::F.finished_at, Some(Utc::now()))
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
    Grpc(#[from] Status),

    /// An error produced by the database
    Database(#[from] rorm::Error),

    /// A malformed grpc message
    ///
    /// For example "optional" fields which have to be set
    Malformed(&'static str),

    /// An error produced by address conversion
    AddressConv(#[from] AddressConvError),

    /// Catch all variant for everything else
    Custom(Box<dyn StdError + Send + Sync>),
}
impl fmt::Display for AttackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttackError::Grpc(status) => write!(
                f,
                "GRPC: {code:?}, {msg:?}",
                code = status.code(),
                msg = status.message()
            ),
            AttackError::Database(err) => write!(f, "DB: {err}"),
            AttackError::Malformed(err) => write!(f, "Malformed response: {err}"),
            AttackError::AddressConv(err) => write!(f, "Error during address conversion: {err}"),
            AttackError::Custom(err) => write!(f, "{err}"),
        }
    }
}
