use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ops::RangeInclusive;

use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, post, HttpResponse};
use chrono::{DateTime, NaiveDateTime, Utc};
use dehashed_rs::{DehashedError, ScheduledRequest, SearchResult};
use futures::StreamExt;
use ipnet::IpNet;
use ipnetwork::IpNetwork;
use log::{debug, error, warn};
use rorm::db::transaction::Transaction;
use rorm::prelude::ForeignModelByField;
use rorm::{and, insert, query, update, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::handler::users::UserResponse;
use crate::api::handler::{query_user, ApiError, ApiResult, PathUuid, UuidResponse};
use crate::api::server::DehashedScheduler;
use crate::chan::{
    CertificateTransparencyEntry, RpcClients, WsManagerChan, WsManagerMessage, WsMessage,
};
use crate::models::{
    Attack, AttackInsert, AttackType, BruteforceSubdomainsResult, BruteforceSubdomainsResultInsert,
    DehashedQueryResultInsert, DnsRecordType, TcpPortScanResult, TcpPortScanResultInsert,
    Workspace, WorkspaceMember,
};
use crate::rpc::rpc_definitions;
use crate::rpc::rpc_definitions::shared::dns_record::Record;
use crate::rpc::rpc_definitions::CertificateTransparencyRequest;

/// The settings of a subdomain bruteforce request
#[derive(Deserialize, ToSchema)]
pub struct BruteforceSubdomainsRequest {
    pub(crate) leech_uuid: Uuid,
    #[schema(example = "example.com")]
    pub(crate) domain: String,
    #[schema(example = "/opt/wordlists/Discovery/DNS/subdomains-top1million-5000.txt")]
    pub(crate) wordlist_path: String,
    #[schema(example = 20)]
    pub(crate) concurrent_limit: u32,
    pub(crate) workspace_uuid: Uuid,
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
    session: Session,
    rpc_clients: RpcClients,
    ws_manager_chan: Data<WsManagerChan>,
) -> ApiResult<HttpResponse> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut client = rpc_clients
        .get_ref()
        .read()
        .await
        .get(&req.leech_uuid)
        .ok_or(ApiError::InvalidLeech)?
        .clone();

    let attack_uuid = insert!(db.as_ref(), AttackInsert)
        .return_primary_key()
        .single(&AttackInsert {
            uuid: Uuid::new_v4(),
            attack_type: AttackType::BruteforceSubdomains,
            started_by: ForeignModelByField::Key(user_uuid),
            workspace: ForeignModelByField::Key(req.workspace_uuid),
            finished_at: None,
        })
        .await?;

    // start attack
    tokio::spawn(async move {
        let req = rpc_definitions::BruteforceSubdomainRequest {
            attack_uuid: attack_uuid.to_string(),
            domain: req.domain.clone(),
            wordlist_path: req.wordlist_path.clone(),
            concurrent_limit: req.concurrent_limit,
        };

        match client.bruteforce_subdomains(req).await {
            Ok(v) => {
                let mut stream = v.into_inner();

                while let Some(res) = stream.next().await {
                    match res {
                        Ok(v) => {
                            let Some(record) = v.record else {
                                warn!("Missing field record in grpc response of bruteforce subdomains");
                                continue;
                            };
                            let Some(record) = record.record else {
                                warn!("Missing field record.record in grpc response of bruteforce subdomains");
                                continue;
                            };

                            let (source, destination, dns_record_type) = match record {
                                Record::A(a_rec) => {
                                    let Some(to) = a_rec.to else {
                                        warn!("Missing field record.record.a.to in grpc response of bruteforce subdomains");
                                        continue;
                                    };

                                    (
                                        a_rec.source,
                                        Ipv4Addr::from(to).to_string(),
                                        DnsRecordType::A,
                                    )
                                }
                                Record::Aaaa(aaaa_rec) => {
                                    let Some(to) = aaaa_rec.to else {
                                        warn!("Missing field record.record.aaaa.to in grpc response of bruteforce subdomains");
                                        continue;
                                    };

                                    (
                                        aaaa_rec.source,
                                        Ipv6Addr::from(to).to_string(),
                                        DnsRecordType::Aaaa,
                                    )
                                }
                                Record::Cname(cname_rec) => {
                                    (cname_rec.source, cname_rec.to, DnsRecordType::Cname)
                                }
                            };

                            let Ok(None) = query!(db.as_ref(), BruteforceSubdomainsResult)
                                .condition(and!(
                                    BruteforceSubdomainsResult::F.attack.equals(&attack_uuid),
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

                            if let Err(err) = insert!(db.as_ref(), BruteforceSubdomainsResult)
                                .single(&BruteforceSubdomainsResultInsert {
                                    uuid: Uuid::new_v4(),
                                    attack: ForeignModelByField::Key(attack_uuid),
                                    dns_record_type,
                                    source: source.clone(),
                                    destination: destination.clone(),
                                })
                                .await
                            {
                                error!("Could not insert data in db: {err}");
                                return;
                            };

                            if let Err(err) = ws_manager_chan
                                .send(WsManagerMessage::Message(
                                    user_uuid,
                                    WsMessage::BruteforceSubdomainsResult {
                                        attack_uuid,
                                        source,
                                        destination,
                                    },
                                ))
                                .await
                            {
                                error!("Couldn't send subdomain enumeration result to ws manager: {err}");
                            }
                        }
                        Err(err) => {
                            error!("Error while reading from stream: {err}");
                            if let Err(err) = ws_manager_chan
                                .send(WsManagerMessage::Message(
                                    user_uuid,
                                    WsMessage::AttackFinished {
                                        attack_uuid,
                                        finished_successful: false,
                                    },
                                ))
                                .await
                            {
                                error!("Couldn't send attack finished to ws manager: {err}");
                            }
                            return;
                        }
                    }
                }
            }
            Err(err) => {
                error!("Error while reading from stream: {err}");
                if let Err(err) = ws_manager_chan
                    .send(WsManagerMessage::Message(
                        user_uuid,
                        WsMessage::AttackFinished {
                            attack_uuid,
                            finished_successful: false,
                        },
                    ))
                    .await
                {
                    error!("Couldn't send attack finished to ws manager: {err}");
                }
                return;
            }
        };

        if let Err(err) = update!(db.as_ref(), Attack)
            .condition(Attack::F.uuid.equals(attack_uuid))
            .set(Attack::F.finished_at, Some(Utc::now()))
            .exec()
            .await
        {
            error!("Database error: {err}");
        }

        if let Err(err) = ws_manager_chan
            .send(WsManagerMessage::Message(
                user_uuid,
                WsMessage::AttackFinished {
                    attack_uuid,
                    finished_successful: true,
                },
            ))
            .await
        {
            error!("Couldn't send attack finished to ws manager: {err}");
        }
    });

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// The settings to configure a tcp port scan
#[derive(Deserialize, ToSchema)]
pub struct ScanTcpPortsRequest {
    pub(crate) leech_uuid: Uuid,

    #[schema(value_type = Vec<String>, example = json!(["10.13.37.1", "10.13.37.2", "10.13.37.50"]))]
    pub(crate) targets: Vec<IpAddr>,

    #[schema(value_type = Vec<String>, example = json!(["10.13.37.252/30"]))]
    pub(crate) exclude: Vec<IpNet>,

    pub(crate) ports: Vec<PortOrRange>,

    #[schema(example = 100)]
    pub(crate) retry_interval: u64,

    #[schema(example = 2)]
    pub(crate) max_retries: u32,

    #[schema(example = 3000)]
    pub(crate) timeout: u64,

    #[schema(example = 5000)]
    pub(crate) concurrent_limit: u32,

    #[schema(example = false)]
    pub(crate) skip_icmp_check: bool,

    pub(crate) workspace_uuid: Uuid,
}

#[derive(Deserialize, ToSchema)]
#[serde(untagged)]
pub enum PortOrRange {
    #[schema(value_type = u32, example = 8000)]
    Port(u16),
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

impl From<&PortOrRange> for rpc_definitions::PortOrRange {
    fn from(value: &PortOrRange) -> Self {
        rpc_definitions::PortOrRange {
            port_or_range: Some(match value {
                PortOrRange::Port(port) => {
                    rpc_definitions::port_or_range::PortOrRange::Single(*port as u32)
                }
                PortOrRange::Range(range) => {
                    rpc_definitions::port_or_range::PortOrRange::Range(rpc_definitions::PortRange {
                        start: *range.start() as u32,
                        end: *range.end() as u32,
                    })
                }
            }),
        }
    }
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
    session: Session,
    rpc_clients: RpcClients,
    ws_manager_chan: Data<WsManagerChan>,
) -> ApiResult<HttpResponse> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut client = rpc_clients
        .get_ref()
        .read()
        .await
        .get(&req.leech_uuid)
        .ok_or(ApiError::InvalidLeech)?
        .clone();

    let uuid = insert!(db.as_ref(), AttackInsert)
        .return_primary_key()
        .single(&AttackInsert {
            uuid: Uuid::new_v4(),
            attack_type: AttackType::TcpPortScan,
            started_by: ForeignModelByField::Key(user_uuid),
            workspace: ForeignModelByField::Key(req.workspace_uuid),
            finished_at: None,
        })
        .await?;

    // start attack
    tokio::spawn(async move {
        let req = rpc_definitions::TcpPortScanRequest {
            attack_uuid: uuid.to_string(),
            targets: req.targets.iter().map(|addr| (*addr).into()).collect(),
            exclude: req.exclude.iter().map(|addr| addr.to_string()).collect(),
            ports: req.ports.iter().map(From::from).collect(),
            retry_interval: req.retry_interval,
            max_retries: req.max_retries,
            timeout: req.timeout,
            concurrent_limit: req.concurrent_limit,
            skip_icmp_check: req.skip_icmp_check,
        };

        match client.run_tcp_port_scan(req).await {
            Ok(v) => {
                let mut stream = v.into_inner();

                while let Some(res) = stream.next().await {
                    match res {
                        Ok(v) => {
                            let Some(addr) = v.address else {
                                warn!("Missing field address in grpc response of scan tcp ports");
                                continue;
                            };

                            let Some(addr) = addr.address else {
                                warn!("Missing field address.address in grpc response of scan tcp ports");
                                continue;
                            };

                            let address = match addr {
                                rpc_definitions::shared::address::Address::Ipv4(addr) => {
                                    IpAddr::V4(addr.into())
                                }

                                rpc_definitions::shared::address::Address::Ipv6(addr) => {
                                    IpAddr::V6(addr.into())
                                }
                            };

                            if let Err(err) = insert!(db.as_ref(), TcpPortScanResult)
                                .return_nothing()
                                .single(&TcpPortScanResultInsert {
                                    uuid: Uuid::new_v4(),
                                    attack: ForeignModelByField::Key(uuid),
                                    address: IpNetwork::from(address),
                                    port: v.port as i32,
                                })
                                .await
                            {
                                error!("Database error: {err}");
                            }

                            if let Err(err) = ws_manager_chan
                                .send(WsManagerMessage::Message(
                                    user_uuid,
                                    WsMessage::ScanTcpPortsResult {
                                        attack_uuid: uuid,
                                        address: address.to_string(),
                                        port: v.port as u16,
                                    },
                                ))
                                .await
                            {
                                error!("Couldn't send scan tcp ports result to ws manager: {err}");
                            }
                        }
                        Err(err) => {
                            error!("Error while reading from stream: {err}");
                            if let Err(err) = ws_manager_chan
                                .send(WsManagerMessage::Message(
                                    user_uuid,
                                    WsMessage::AttackFinished {
                                        attack_uuid: uuid,
                                        finished_successful: false,
                                    },
                                ))
                                .await
                            {
                                error!("Couldn't send attack finished to ws manager: {err}");
                            }
                            return;
                        }
                    }
                }
            }
            Err(err) => {
                error!("Error while reading from stream: {err}");
                if let Err(err) = ws_manager_chan
                    .send(WsManagerMessage::Message(
                        user_uuid,
                        WsMessage::AttackFinished {
                            attack_uuid: uuid,
                            finished_successful: false,
                        },
                    ))
                    .await
                {
                    error!("Couldn't send attack finished to ws manager: {err}");
                }
                return;
            }
        };

        if let Err(err) = update!(db.as_ref(), Attack)
            .condition(Attack::F.uuid.equals(uuid))
            .set(Attack::F.finished_at, Some(Utc::now()))
            .exec()
            .await
        {
            error!("Database error: {err}");
        }

        if let Err(err) = ws_manager_chan
            .send(WsManagerMessage::Message(
                user_uuid,
                WsMessage::AttackFinished {
                    attack_uuid: uuid,
                    finished_successful: true,
                },
            ))
            .await
        {
            error!("Couldn't send attack finished to ws manager: {err}");
        }
    });

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: user_uuid }))
}

/// The settings to configure a certificate transparency request
#[derive(Deserialize, ToSchema)]
pub struct QueryCertificateTransparencyRequest {
    pub(crate) leech_uuid: Uuid,
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
    session: Session,
    rpc_clients: RpcClients,
    ws_manager_chan: Data<WsManagerChan>,
) -> ApiResult<HttpResponse> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut client = rpc_clients
        .get_ref()
        .read()
        .await
        .get(&req.leech_uuid)
        .ok_or(ApiError::InvalidLeech)?
        .clone();

    let uuid = insert!(db.as_ref(), AttackInsert)
        .return_primary_key()
        .single(&AttackInsert {
            uuid: Uuid::new_v4(),
            attack_type: AttackType::QueryCertificateTransparency,
            started_by: ForeignModelByField::Key(user_uuid),
            workspace: ForeignModelByField::Key(req.workspace_uuid),
            finished_at: None,
        })
        .await?;

    tokio::spawn(async move {
        let req = CertificateTransparencyRequest {
            target: req.target.clone(),
            max_retries: req.max_retries,
            retry_interval: req.retry_interval,
            include_expired: req.include_expired,
        };

        match client.query_certificate_transparency(req).await {
            Ok(res) => {
                let res = res.into_inner();

                if let Err(err) = ws_manager_chan
                    .send(WsManagerMessage::Message(
                        user_uuid,
                        WsMessage::CertificateTransparencyResult {
                            attack_uuid: uuid,
                            entries: res
                                .entries
                                .into_iter()
                                .map(|e| CertificateTransparencyEntry {
                                    serial_number: e.serial_number,
                                    issuer_name: e.issuer_name,
                                    common_name: e.common_name,
                                    value_names: e.value_names,
                                    not_before: e.not_before.map(|ts| {
                                        DateTime::from_naive_utc_and_offset(
                                            NaiveDateTime::from_timestamp_opt(
                                                ts.seconds,
                                                ts.nanos as u32,
                                            )
                                            .unwrap(),
                                            Utc,
                                        )
                                    }),
                                    not_after: e.not_after.map(|ts| {
                                        DateTime::from_naive_utc_and_offset(
                                            NaiveDateTime::from_timestamp_opt(
                                                ts.seconds,
                                                ts.nanos as u32,
                                            )
                                            .unwrap(),
                                            Utc,
                                        )
                                    }),
                                })
                                .collect(),
                        },
                    ))
                    .await
                {
                    error!(
                        "Couldn't send query certificate transparency result to ws manager: {err}"
                    );
                }
            }
            Err(err) => {
                error!("Error while reading from stream: {err}");
                if let Err(err) = ws_manager_chan
                    .send(WsManagerMessage::Message(
                        user_uuid,
                        WsMessage::AttackFinished {
                            attack_uuid: uuid,
                            finished_successful: false,
                        },
                    ))
                    .await
                {
                    error!("Couldn't send attack finished to ws manager: {err}");
                }
                return;
            }
        }

        if let Err(err) = update!(db.as_ref(), Attack)
            .condition(Attack::F.uuid.equals(uuid))
            .set(Attack::F.finished_at, Some(Utc::now()))
            .exec()
            .await
        {
            error!("Database error: {err}");
        }

        if let Err(err) = ws_manager_chan
            .send(WsManagerMessage::Message(
                user_uuid,
                WsMessage::AttackFinished {
                    attack_uuid: uuid,
                    finished_successful: true,
                },
            ))
            .await
        {
            error!("Couldn't send attack finished to ws manager: {err}");
        }
    });

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid }))
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
    session: Session,
    dehashed_scheduler: DehashedScheduler,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let req = req.into_inner();
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let sender = {
        match dehashed_scheduler.try_read()?.as_ref() {
            None => return Err(ApiError::DehashedNotAvailable),
            Some(scheduler) => scheduler.retrieve_sender(),
        }
    };

    let (tx, rx) = oneshot::channel::<Result<SearchResult, DehashedError>>();

    let attack_uuid = insert!(db.as_ref(), AttackInsert)
        .return_primary_key()
        .single(&AttackInsert {
            uuid: Uuid::new_v4(),
            attack_type: AttackType::QueryUnhashed,
            started_by: ForeignModelByField::Key(user_uuid),
            workspace: ForeignModelByField::Key(req.workspace_uuid),
            finished_at: None,
        })
        .await?;

    tokio::spawn(async move {
        if let Err(err) = sender.send(ScheduledRequest::new(req.query, tx)).await {
            error!("Couldn't send to dehashed scheduler: {err}");
            return;
        }

        let res = match rx.await {
            Err(err) => {
                error!("Error waiting for result: {err}");
                return;
            }
            Ok(Err(err)) => {
                error!("Error while using dehashed: {err}");
                return;
            }
            Ok(Ok(res)) => res,
        };

        let entries: Vec<_> = res
            .entries
            .into_iter()
            .map(|x| DehashedQueryResultInsert {
                uuid: Uuid::new_v4(),
                dehashed_id: x.id as i64,
                username: x.username,
                name: x.name,
                email: x.email,
                password: x.password,
                hashed_password: x.hashed_password,
                database_name: x.database_name,
                address: x.address,
                phone: x.phone,
                vin: x.vin,
                ip_address: x.ip_address.map(IpNetwork::from),
                attack: ForeignModelByField::Key(attack_uuid),
            })
            .collect();

        if let Err(err) = insert!(db.as_ref(), DehashedQueryResultInsert)
            .bulk(&entries)
            .await
        {
            error!("Database error: {err}");
            return;
        }

        if let Err(err) = ws_manager_chan
            .send(WsManagerMessage::Message(
                user_uuid,
                WsMessage::AttackFinished {
                    attack_uuid,
                    finished_successful: true,
                },
            ))
            .await
        {
            error!("Couldn't send attack finished to ws manager: {err}");
        };
    });

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// A simple version of an attack
#[derive(Serialize, ToSchema)]
pub(crate) struct SimpleAttack {
    pub(crate) uuid: Uuid,
    pub(crate) workspace_uuid: Uuid,
    pub(crate) attack_type: AttackType,
    pub(crate) started_from: UserResponse,
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
pub(crate) async fn get_attack(
    req: Path<PathUuid>,
    db: Data<Database>,
    session: Session,
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

    let attack = if has_access(&mut tx, req.uuid, &session).await? {
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
            started_from: UserResponse {
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

#[derive(Deserialize, ToSchema, IntoParams)]
pub(crate) struct PageParams {
    /// Number of items to retrieve
    #[schema(example = 50)]
    limit: u64,

    /// Position in the whole list to start retrieving from
    #[schema(example = 0)]
    offset: u64,
}

#[derive(Serialize, ToSchema)]
#[aliases(TcpPortScanResultsPage = Page<SimpleTcpPortScanResult>)]
pub(crate) struct Page<T> {
    /// The page's items
    pub(crate) items: Vec<T>,

    /// The limit this page was retrieved with
    #[schema(example = 50)]
    pub(crate) limit: u64,

    /// The offset this page was retrieved with
    #[schema(example = 0)]
    pub(crate) offset: u64,

    /// The total number of items this page is a subset of
    pub(crate) total: u64,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct SimpleTcpPortScanResult {
    pub uuid: Uuid,
    pub attack: Uuid,
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String)]
    pub address: IpNetwork,
    pub port: u16,
}

/// Retrieve a tcp port scan's results by the attack's id
#[utoipa::path(
tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns attack's results", body = TcpPortScanResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}/tcpPortScanResults")]
pub(crate) async fn get_tcp_port_scan_results(
    path: Path<PathUuid>,
    query: Query<PageParams>,
    session: Session,
    db: Data<Database>,
) -> ApiResult<Json<TcpPortScanResultsPage>> {
    let mut tx = db.start_transaction().await?;

    let uuid = path.uuid;
    let PageParams { limit, offset } = query.into_inner();

    let page = if !has_access(&mut tx, uuid, &session).await? {
        Err(ApiError::MissingPrivileges)
    } else {
        let (total,) = query!(&mut tx, (TcpPortScanResult::F.uuid.count(),))
            .condition(TcpPortScanResult::F.attack.equals(uuid))
            .one()
            .await?;
        let results = query!(&mut tx, TcpPortScanResult)
            .condition(TcpPortScanResult::F.attack.equals(uuid))
            .order_asc(TcpPortScanResult::F.uuid)
            .limit(limit)
            .offset(offset)
            .all()
            .await?
            .into_iter()
            .map(|result| SimpleTcpPortScanResult {
                uuid: result.uuid,
                attack: *result.attack.key(),
                created_at: result.created_at,
                address: result.address,
                port: result.port as u16,
            })
            .collect();
        Ok(Page {
            items: results,
            limit,
            offset,
            total: total as u64,
        })
    };

    tx.commit().await?;

    Ok(Json(page?))
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
pub(crate) async fn delete_attack(
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

/// Does the user have access to the attack's workspace?
/// I.e. is owner or member?
async fn has_access(tx: &mut Transaction, attack_uuid: Uuid, session: &Session) -> ApiResult<bool> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let (workspace, owner) = query!(&mut *tx, (Workspace::F.uuid, Workspace::F.owner))
        .condition(Workspace::F.attacks.uuid.equals(attack_uuid))
        .one()
        .await?;
    if *owner.key() == uuid {
        return Ok(true);
    }

    Ok(query!(&mut *tx, (WorkspaceMember::F.id,))
        .condition(and!(
            WorkspaceMember::F.workspace.equals(workspace),
            WorkspaceMember::F.member.equals(uuid),
        ))
        .optional()
        .await?
        .is_some())
}
