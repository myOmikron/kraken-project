//! This module holds all attack handlers and their request and response schemas

use std::net::IpAddr;
use std::ops::RangeInclusive;

use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, HttpResponse};
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use log::debug;
use rorm::prelude::*;
use rorm::{query, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::users::SimpleUser;
use crate::api::handler::{query_user, ApiError, ApiResult, PathUuid, UuidResponse};
use crate::api::server::DehashedScheduler;
use crate::chan::{RpcClients, WsManagerChan};
use crate::models::{Attack, AttackType, WordList};
use crate::modules::attacks::AttackContext;
use crate::rpc::rpc_definitions;
use crate::rpc::rpc_definitions::CertificateTransparencyRequest;

/// The settings of a subdomain bruteforce request
#[derive(Deserialize, ToSchema)]
pub struct BruteforceSubdomainsRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// Domain to construct subdomains for
    #[schema(example = "example.com")]
    pub domain: String,

    /// The wordlist to use
    pub wordlist_uuid: Uuid,

    /// The concurrent task limit
    #[schema(example = 20)]
    pub(crate) concurrent_limit: u32,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// Bruteforce subdomains through a DNS wordlist attack
///
/// Enumerate possible subdomains by querying a DNS server with constructed domains.
/// See [OWASP](https://owasp.org/www-community/attacks/Brute_force_attack) for further information.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = BruteforceSubdomainsRequest,
    security(("api_key" = []))
)]
#[post("/attacks/bruteforceSubdomains")]
pub async fn bruteforce_subdomains(
    req: Json<BruteforceSubdomainsRequest>,
    db: Data<Database>,
    rpc_clients: RpcClients,
    ws_manager_chan: Data<WsManagerChan>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let BruteforceSubdomainsRequest {
        leech_uuid,
        domain,
        wordlist_uuid,
        concurrent_limit,
        workspace_uuid,
    } = req.into_inner();

    let (wordlist_path,) = query!(db.as_ref(), (WordList::F.path,))
        .condition(WordList::F.uuid.equals(wordlist_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let client = if let Some(leech_uuid) = leech_uuid {
        rpc_clients.get_leech(&leech_uuid)?
    } else {
        rpc_clients.random_leech()?
    };

    let attack_uuid = Attack::insert(
        db.as_ref(),
        AttackType::BruteforceSubdomains,
        user_uuid,
        workspace_uuid,
    )
    .await?;

    // start attack
    tokio::spawn(
        AttackContext {
            db: Database::clone(&db),
            ws_manager: WsManagerChan::clone(&ws_manager_chan),
            user_uuid,
            workspace_uuid,
            attack_uuid,
        }
        .leech(client)
        .bruteforce_subdomains(rpc_definitions::BruteforceSubdomainRequest {
            attack_uuid: attack_uuid.to_string(),
            domain: domain.to_string(),
            wordlist_path,
            concurrent_limit,
        }),
    );

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// The settings to configure a tcp port scan
#[derive(Deserialize, ToSchema)]
pub struct ScanTcpPortsRequest {
    /// The leech to use
    ///
    /// Leave empty to use a random leech
    pub leech_uuid: Option<Uuid>,

    /// The ip addresses / networks to scan
    #[schema(value_type = Vec<String>, example = json!(["10.13.37.1", "10.13.37.2", "10.13.37.0/24"]))]
    pub targets: Vec<IpNetwork>,

    /// List of single ports and port ranges
    ///
    /// If no values are supplied, 1-65535 is used as default
    #[serde(default)]
    pub ports: Vec<PortOrRange>,

    /// The interval that should be wait between retries on a port.
    ///
    /// The interval is specified in milliseconds.
    #[schema(example = 100)]
    pub retry_interval: u64,

    /// The number of times the connection should be retried if it failed.
    #[schema(example = 2)]
    pub max_retries: u32,

    /// The time to wait until a connection is considered failed.
    ///
    /// The timeout is specified in milliseconds.
    #[schema(example = 3000)]
    pub timeout: u64,

    /// The concurrent task limit
    #[schema(example = 5000)]
    pub concurrent_limit: u32,

    /// Skips the initial icmp check.
    ///
    /// All hosts are assumed to be reachable
    #[schema(example = false)]
    pub skip_icmp_check: bool,

    /// The workspace to execute the attack in
    pub workspace_uuid: Uuid,
}

/// Single port or a range of ports
#[derive(Deserialize, ToSchema)]
#[serde(untagged)]
pub enum PortOrRange {
    /// A single port
    #[schema(example = 8000)]
    Port(u16),
    /// In inclusive range of ports
    #[schema(value_type = String, example = "1-1024")]
    Range(#[serde(deserialize_with = "deserialize_port_range")] RangeInclusive<u16>),
}

fn deserialize_port_range<'de, D>(deserializer: D) -> Result<RangeInclusive<u16>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value
        .split_once('-')
        .and_then(|(start, end)| Some((start.parse::<u16>().ok()?)..=(end.parse::<u16>().ok()?)))
        .ok_or_else(|| {
            <D::Error as serde::de::Error>::invalid_value(serde::de::Unexpected::Str(&value), &"")
        })
}

/// Start a tcp port scan
///
/// `exclude` accepts a list of ip networks in CIDR notation.
///
/// All intervals are interpreted in milliseconds. E.g. a `timeout` of 3000 means 3 seconds.
///
/// Set `max_retries` to 0 if you don't want to try a port more than 1 time.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = ScanTcpPortsRequest,
    security(("api_key" = []))
)]
#[post("/attacks/scanTcpPorts")]
pub async fn scan_tcp_ports(
    req: Json<ScanTcpPortsRequest>,
    db: Data<Database>,
    rpc_clients: RpcClients,
    ws_manager_chan: Data<WsManagerChan>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let ScanTcpPortsRequest {
        leech_uuid,
        targets,
        ports,
        retry_interval,
        max_retries,
        timeout,
        concurrent_limit,
        skip_icmp_check,
        workspace_uuid,
    } = req.into_inner();

    let client = if let Some(leech_uuid) = leech_uuid {
        rpc_clients.get_leech(&leech_uuid)?
    } else {
        rpc_clients.random_leech()?
    };

    let attack_uuid = Attack::insert(
        db.as_ref(),
        AttackType::TcpPortScan,
        user_uuid,
        workspace_uuid,
    )
    .await?;

    // start attack
    tokio::spawn(
        AttackContext {
            db: Database::clone(&db),
            ws_manager: WsManagerChan::clone(&ws_manager_chan),
            user_uuid,
            workspace_uuid,
            attack_uuid,
        }
        .leech(client)
        .tcp_port_scan(rpc_definitions::TcpPortScanRequest {
            attack_uuid: attack_uuid.to_string(),
            targets: targets.iter().map(|addr| (*addr).into()).collect(),
            ports: ports
                .iter()
                .map(|value| rpc_definitions::PortOrRange {
                    port_or_range: Some(match value {
                        PortOrRange::Port(port) => {
                            rpc_definitions::port_or_range::PortOrRange::Single(*port as u32)
                        }
                        PortOrRange::Range(range) => {
                            rpc_definitions::port_or_range::PortOrRange::Range(
                                rpc_definitions::PortRange {
                                    start: *range.start() as u32,
                                    end: *range.end() as u32,
                                },
                            )
                        }
                    }),
                })
                .collect(),
            retry_interval,
            max_retries,
            timeout,
            concurrent_limit,
            skip_icmp_check,
        }),
    );

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: user_uuid }))
}

/// Host Alive check request
#[derive(Deserialize, ToSchema)]
pub struct HostsAliveRequest {
    pub(crate) leech_uuid: Option<Uuid>,

    #[schema(value_type = Vec<String>, example = json!(["10.13.37.1", "10.13.37.2", "10.13.37.0/24"]))]
    pub(crate) targets: Vec<IpNetwork>,

    #[schema(example = 3000)]
    pub(crate) timeout: u64,

    #[schema(example = 30)]
    pub(crate) concurrent_limit: u32,

    pub(crate) workspace_uuid: Uuid,
}

/// Check if hosts are reachable
///
/// Just an ICMP scan for now to see which targets respond.
///
/// All intervals are interpreted in milliseconds. E.g. a `timeout` of 3000 means 3 seconds.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = HostsAliveRequest,
    security(("api_key" = []))
)]
#[post("/attacks/hostsAlive")]
pub async fn hosts_alive_check(
    db: Data<Database>,
    req: Json<HostsAliveRequest>,
    SessionUser(user_uuid): SessionUser,
    rpc_clients: RpcClients,
    ws_manager_chan: Data<WsManagerChan>,
) -> ApiResult<HttpResponse> {
    let HostsAliveRequest {
        leech_uuid,
        targets,
        timeout,
        concurrent_limit,
        workspace_uuid,
    } = req.into_inner();

    let attack_uuid = Attack::insert(
        db.as_ref(),
        AttackType::HostAlive,
        user_uuid,
        workspace_uuid,
    )
    .await?;

    let leech = if let Some(leech_uuid) = leech_uuid {
        rpc_clients.get_leech(&leech_uuid)?
    } else {
        rpc_clients.random_leech()?
    };

    tokio::spawn(
        AttackContext {
            db: Database::clone(&db),
            ws_manager: WsManagerChan::clone(&ws_manager_chan),
            user_uuid,
            workspace_uuid,
            attack_uuid,
        }
        .leech(leech)
        .host_alive_check(rpc_definitions::HostsAliveRequest {
            attack_uuid: attack_uuid.to_string(),
            targets: targets.into_iter().map(Into::into).collect(),
            timeout,
            concurrent_limit,
        }),
    );

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// The settings to configure a certificate transparency request
#[derive(Deserialize, ToSchema)]
pub struct QueryCertificateTransparencyRequest {
    #[schema(example = "example.com")]
    pub(crate) target: String,
    #[schema(example = true)]
    pub(crate) include_expired: bool,
    #[schema(example = 3)]
    pub(crate) max_retries: u32,
    #[schema(example = 500)]
    pub(crate) retry_interval: u64,
    pub(crate) workspace_uuid: Uuid,
}

/// Query a certificate transparency log collector.
///
/// For further information, see [the explanation](https://certificate.transparency.dev/).
///
/// Certificate transparency can be used to find subdomains or related domains.
///
/// `retry_interval` is specified in milliseconds.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = QueryCertificateTransparencyRequest,
    security(("api_key" = []))
)]
#[post("/attacks/queryCertificateTransparency")]
pub async fn query_certificate_transparency(
    req: Json<QueryCertificateTransparencyRequest>,
    db: Data<Database>,
    rpc_clients: RpcClients,
    ws_manager_chan: Data<WsManagerChan>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let QueryCertificateTransparencyRequest {
        target,
        include_expired,
        max_retries,
        retry_interval,
        workspace_uuid,
    } = req.into_inner();

    let client = rpc_clients.random_leech()?;

    let attack_uuid = Attack::insert(
        db.as_ref(),
        AttackType::QueryCertificateTransparency,
        user_uuid,
        workspace_uuid,
    )
    .await?;

    tokio::spawn(
        AttackContext {
            db: Database::clone(&db),
            ws_manager: WsManagerChan::clone(&ws_manager_chan),
            user_uuid,
            workspace_uuid,
            attack_uuid,
        }
        .leech(client)
        .query_certificate_transparency(CertificateTransparencyRequest {
            attack_uuid: attack_uuid.to_string(),
            target,
            include_expired,
            max_retries,
            retry_interval,
        }),
    );

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// The request to query the dehashed API
#[derive(ToSchema, Deserialize)]
pub struct QueryDehashedRequest {
    #[schema(value_type = Query)]
    query: dehashed_rs::Query,
    workspace_uuid: Uuid,
}

/// Query the [dehashed](https://dehashed.com/) API.
/// It provides email, password, credit cards and other types of information from leak-databases.
///
/// Note that you are only able to query the API if you have bought access and have a running
/// subscription saved in kraken.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = QueryDehashedRequest,
    security(("api_key" = []))
)]
#[post("/attacks/queryDehashed")]
pub async fn query_dehashed(
    req: Json<QueryDehashedRequest>,
    ws_manager_chan: Data<WsManagerChan>,
    SessionUser(user_uuid): SessionUser,
    dehashed_scheduler: DehashedScheduler,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let QueryDehashedRequest {
        query,
        workspace_uuid,
    } = req.into_inner();

    let sender = {
        match dehashed_scheduler.try_read()?.as_ref() {
            None => return Err(ApiError::DehashedNotAvailable),
            Some(scheduler) => scheduler.retrieve_sender(),
        }
    };

    let attack_uuid = Attack::insert(
        db.as_ref(),
        AttackType::QueryUnhashed,
        user_uuid,
        workspace_uuid,
    )
    .await?;

    tokio::spawn(
        AttackContext {
            db: Database::clone(&db),
            ws_manager: WsManagerChan::clone(&ws_manager_chan),
            user_uuid,
            workspace_uuid,
            attack_uuid,
        }
        .query_dehashed(sender, query),
    );

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// The request to start a service detection
#[derive(Debug, ToSchema, Deserialize)]
pub struct ServiceDetectionRequest {
    /// If missing - a random leech is chosen
    pub(crate) leech_uuid: Option<Uuid>,
    #[schema(value_type = String, example = "10.13.37.1")]
    pub(crate) address: IpAddr,
    #[schema(example = 443)]
    pub(crate) port: u16,
    #[schema(example = 3000)]
    pub(crate) timeout: u64,
    pub(crate) workspace_uuid: Uuid,
}

/// Perform service detection on a ip and port combination
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = ServiceDetectionRequest,
    security(("api_key" = []))
)]
#[post("/attacks/serviceDetection")]
pub async fn service_detection(
    req: Json<ServiceDetectionRequest>,
    ws_manager_chan: Data<WsManagerChan>,
    rpc_clients: RpcClients,
    SessionUser(user_uuid): SessionUser,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let ServiceDetectionRequest {
        leech_uuid,
        address,
        port,
        timeout,
        workspace_uuid,
    } = req.into_inner();

    if port == 0 {
        return Err(ApiError::InvalidPort);
    }

    let client = if let Some(leech_uuid) = leech_uuid {
        rpc_clients.get_leech(&leech_uuid)?
    } else {
        rpc_clients.random_leech()?
    };

    let attack_uuid = Attack::insert(
        db.as_ref(),
        AttackType::ServiceDetection,
        user_uuid,
        workspace_uuid,
    )
    .await?;

    // start attack
    tokio::spawn(
        AttackContext {
            db: Database::clone(&db),
            ws_manager: WsManagerChan::clone(&ws_manager_chan),
            user_uuid,
            workspace_uuid,
            attack_uuid,
        }
        .leech(client)
        .service_detection(
            rpc_definitions::ServiceDetectionRequest {
                attack_uuid: attack_uuid.to_string(),
                address: Some(address.into()),
                timeout,
                port: port as u32,
            },
            address,
            port,
        ),
    );

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// Request to resolve domains
#[derive(Deserialize, ToSchema)]
pub struct DnsResolutionRequest {
    /// If missing - a random leech is chosen
    pub(crate) leech_uuid: Option<Uuid>,
    #[schema(value_type = Vec<String>, example = json!(["example.com", "example.org"]))]
    pub(crate) targets: Vec<String>,
    #[schema(example = 2)]
    pub(crate) concurrent_limit: u32,
    pub(crate) workspace_uuid: Uuid,
}

/// Perform domain name resolution
#[utoipa::path(
tag = "Attacks",
context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = DnsResolutionRequest,
    security(("api_key" = []))
)]
#[post("/attacks/dnsResolution")]
pub async fn dns_resolution(
    req: Json<DnsResolutionRequest>,
    db: Data<Database>,
    rpc_clients: RpcClients,
    SessionUser(user_uuid): SessionUser,
    ws_manager_chan: Data<WsManagerChan>,
) -> ApiResult<HttpResponse> {
    let DnsResolutionRequest {
        leech_uuid,
        targets,
        concurrent_limit,
        workspace_uuid,
    } = req.into_inner();

    if targets.is_empty() {
        return Err(ApiError::EmptyTargets);
    }

    let client = if let Some(leech_uuid) = leech_uuid {
        rpc_clients.get_leech(&leech_uuid)?
    } else {
        rpc_clients.random_leech()?
    };

    let attack_uuid = Attack::insert(
        db.as_ref(),
        AttackType::DnsResolution,
        user_uuid,
        workspace_uuid,
    )
    .await?;

    // start attack
    tokio::spawn(
        AttackContext {
            db: Database::clone(&db),
            ws_manager: WsManagerChan::clone(&ws_manager_chan),
            user_uuid,
            workspace_uuid,
            attack_uuid,
        }
        .leech(client)
        .dns_resolution(rpc_definitions::DnsResolutionRequest {
            attack_uuid: attack_uuid.to_string(),
            targets,
            concurrent_limit,
        }),
    );

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// A simple version of an attack
#[derive(Serialize, ToSchema)]
pub struct SimpleAttack {
    pub(crate) uuid: Uuid,
    pub(crate) workspace_uuid: Uuid,
    pub(crate) attack_type: AttackType,
    pub(crate) started_from: SimpleUser,
    pub(crate) finished_at: Option<DateTime<Utc>>,
    pub(crate) created_at: DateTime<Utc>,
}

/// Retrieve an attack by id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns the attack", body = SimpleAttack),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}")]
pub async fn get_attack(
    req: Path<PathUuid>,
    db: Data<Database>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<SimpleAttack>> {
    let mut tx = db.start_transaction().await?;

    let attack = query!(
        &mut tx,
        (
            Attack::F.uuid,
            Attack::F.workspace,
            Attack::F.attack_type,
            Attack::F.finished_at,
            Attack::F.created_at,
            Attack::F.started_by.uuid,
            Attack::F.started_by.username,
            Attack::F.started_by.display_name,
        )
    )
    .condition(Attack::F.uuid.equals(req.uuid))
    .optional()
    .await?
    .ok_or(ApiError::InvalidUuid)?;

    let attack = if Attack::has_access(&mut tx, req.uuid, user_uuid).await? {
        let (
            uuid,
            workspace,
            attack_type,
            finished_at,
            created_at,
            by_uuid,
            username,
            display_name,
        ) = attack;
        Ok(SimpleAttack {
            uuid,
            workspace_uuid: *workspace.key(),
            attack_type,
            started_from: SimpleUser {
                uuid: by_uuid,
                username,
                display_name,
            },
            finished_at,
            created_at,
        })
    } else {
        Err(ApiError::MissingPrivileges)
    };

    tx.commit().await?;

    Ok(Json(attack?))
}

/// Delete an attack and its results
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Attack was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/attacks/{uuid}")]
pub async fn delete_attack(
    req: Path<PathUuid>,
    session: Session,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let mut tx = db.start_transaction().await?;

    let user = query_user(&mut tx, &session).await?;

    let attack = query!(&mut tx, Attack)
        .condition(Attack::F.uuid.equals(req.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if user.admin || *attack.started_by.key() == user.uuid {
        debug!("Attack {} got deleted by {}", attack.uuid, user.username);

        rorm::delete!(&mut tx, Attack).single(&attack).await?;
    } else {
        debug!(
            "User {} does not has the privileges to delete the attack {}",
            user.username, attack.uuid
        );

        return Err(ApiError::MissingPrivileges);
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
