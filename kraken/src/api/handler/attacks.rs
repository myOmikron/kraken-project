use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::num::NonZeroU16;

use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json};
use actix_web::{post, HttpResponse};
use chrono::Utc;
use futures::StreamExt;
use ipnet::IpNet;
use log::{error, warn};
use rorm::internal::field::foreign_model::ForeignModelByField;
use rorm::{insert, update, Database, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::handler::{ApiError, ApiResult};
use crate::chan::{RpcClients, WsManagerChan, WsManagerMessage, WsMessage};
use crate::models::{Attack, AttackInsert, AttackType};
use crate::rpc::rpc_attacks;
use crate::rpc::rpc_attacks::shared::dns_record::Record;

#[derive(Serialize, ToSchema)]
pub(crate) struct AttackResponse {
    pub(crate) attack_id: i64,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct BruteforceSubdomainsRequest {
    pub(crate) leech_id: u32,
    pub(crate) domain: String,
    pub(crate) wordlist_path: String,
    pub(crate) concurrent_limit: u32,
}

#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = AttackResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = BruteforceSubdomainsRequest,
    security(("api_key" = []))
)]
#[post("/attacks/bruteforceSubdomains")]
pub(crate) async fn bruteforce_subdomains(
    req: Json<BruteforceSubdomainsRequest>,
    db: Data<Database>,
    session: Session,
    rpc_clients: RpcClients,
    ws_manager_chan: Data<WsManagerChan>,
) -> ApiResult<HttpResponse> {
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut client = rpc_clients
        .get_ref()
        .read()
        .await
        .get(&(req.leech_id as i64))
        .ok_or(ApiError::InvalidLeech)?
        .clone();

    let id = insert!(&db, AttackInsert)
        .single(&AttackInsert {
            attack_type: AttackType::BruteforceSubdomains.into(),
            started_from: ForeignModelByField::Key(uuid.clone()),
            finished_at: None,
        })
        .await?;

    // start attack
    tokio::spawn(async move {
        let req = rpc_attacks::BruteforceSubdomainRequest {
            attack_id: id as u64,
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

                            let (source, to) = match record {
                                Record::A(a_rec) => {
                                    let Some(to) = a_rec.to else  {
                                        warn!("Missing field record.record.a.to in grpc response of bruteforce subdomains");
                                        continue;
                                    };

                                    (a_rec.source, Ipv4Addr::from(to).to_string())
                                }
                                Record::Aaaa(aaaa_rec) => {
                                    let Some(to) = aaaa_rec.to else  {
                                        warn!("Missing field record.record.aaaa.to in grpc response of bruteforce subdomains");
                                        continue;
                                    };

                                    (aaaa_rec.source, Ipv6Addr::from(to).to_string())
                                }
                                Record::Cname(cname_rec) => (cname_rec.source, cname_rec.to),
                            };

                            if let Err(err) = ws_manager_chan
                                .send(WsManagerMessage::Message(
                                    uuid.clone(),
                                    WsMessage::BruteforceSubdomainsResult {
                                        attack_id: id,
                                        source,
                                        to,
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
                                    uuid.clone(),
                                    WsMessage::AttackFinished {
                                        attack_id: id,
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
                        uuid.clone(),
                        WsMessage::AttackFinished {
                            attack_id: id,
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

        let now = Utc::now();
        if let Err(err) = update!(&db, Attack)
            .condition(Attack::F.id.equals(id))
            .set(Attack::F.finished_at, Some(now.naive_utc()))
            .exec()
            .await
        {
            error!("Database error: {err}");
        }

        if let Err(err) = ws_manager_chan
            .send(WsManagerMessage::Message(
                uuid.clone(),
                WsMessage::AttackFinished {
                    attack_id: id,
                    finished_successful: true,
                },
            ))
            .await
        {
            error!("Couldn't send attack finished to ws manager: {err}");
        }
    });

    Ok(HttpResponse::Accepted().json(AttackResponse { attack_id: id }))
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({
    "leech_id": 1,
    "targets": [
        "10.13.37.1",
        "10.13.37.2",
        "10.13.37.50"
    ],
    "exclude": ["10.13.37.252/30"],
    "ports": [80, 443, 3306, 5432],
    "retry_interval": 100,
    "max_retries": 2,
    "timeout": 3000,
    "concurrent_limit": 5000,
    "skip_icmp_check": false
}))]
pub(crate) struct ScanTcpPortsRequest {
    pub(crate) leech_id: u32,
    #[schema(value_type = Vec<String>)]
    pub(crate) targets: Vec<IpAddr>,
    #[schema(value_type = Vec<String>)]
    pub(crate) exclude: Vec<IpNet>,
    #[schema(value_type = Vec<u16>)]
    pub(crate) ports: Vec<NonZeroU16>,
    pub(crate) retry_interval: u64,
    pub(crate) max_retries: u32,
    pub(crate) timeout: u64,
    pub(crate) concurrent_limit: u32,
    pub(crate) skip_icmp_check: bool,
}

/// Start a tcp port scan.
///
/// Use this method to start a tcp port scan.
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
        (status = 202, description = "Attack scheduled", body = AttackResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = ScanTcpPortsRequest,
    security(("api_key" = []))
)]
#[post("/attacks/scanTcpPorts")]
pub(crate) async fn scan_tcp_ports(
    req: Json<ScanTcpPortsRequest>,
    db: Data<Database>,
    session: Session,
    rpc_clients: RpcClients,
    ws_manager_chan: Data<WsManagerChan>,
) -> ApiResult<HttpResponse> {
    let uuid: Vec<u8> = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut client = rpc_clients
        .get_ref()
        .read()
        .await
        .get(&(req.leech_id as i64))
        .ok_or(ApiError::InvalidLeech)?
        .clone();

    let id = insert!(&db, AttackInsert)
        .single(&AttackInsert {
            attack_type: AttackType::TcpPortScan.into(),
            started_from: ForeignModelByField::Key(uuid.clone()),
            finished_at: None,
        })
        .await?;

    // start attack
    tokio::spawn(async move {
        let req = rpc_attacks::TcpPortScanRequest {
            attack_id: id as u64,
            targets: req.targets.iter().map(|addr| (*addr).into()).collect(),
            exclude: req.exclude.iter().map(|addr| addr.to_string()).collect(),
            ports: req.ports.iter().map(|p| u16::from(*p) as u32).collect(),
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
                                rpc_attacks::shared::address::Address::Ipv4(addr) => {
                                    let a: Ipv4Addr = addr.into();
                                    a.to_string()
                                }
                                rpc_attacks::shared::address::Address::Ipv6(addr) => {
                                    let a: Ipv6Addr = addr.into();
                                    a.to_string()
                                }
                            };

                            if let Err(err) = ws_manager_chan
                                .send(WsManagerMessage::Message(
                                    uuid.clone(),
                                    WsMessage::ScanTcpPortsResult {
                                        attack_id: id,
                                        address,
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
                                    uuid.clone(),
                                    WsMessage::AttackFinished {
                                        attack_id: id,
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
                        uuid.clone(),
                        WsMessage::AttackFinished {
                            attack_id: id,
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

        let now = Utc::now();
        if let Err(err) = update!(&db, Attack)
            .condition(Attack::F.id.equals(id))
            .set(Attack::F.finished_at, Some(now.naive_utc()))
            .exec()
            .await
        {
            error!("Database error: {err}");
        }

        if let Err(err) = ws_manager_chan
            .send(WsManagerMessage::Message(
                uuid.clone(),
                WsMessage::AttackFinished {
                    attack_id: id,
                    finished_successful: true,
                },
            ))
            .await
        {
            error!("Couldn't send attack finished to ws manager: {err}");
        }
    });

    Ok(HttpResponse::Accepted().json(AttackResponse { attack_id: id }))
}
