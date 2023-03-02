use std::net::{Ipv4Addr, Ipv6Addr};

use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json};
use actix_web::{post, HttpResponse};
use chrono::Utc;
use futures::StreamExt;
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

#[derive(Deserialize)]
pub(crate) struct BruteforceSubdomainsRequest {
    pub(crate) leech_id: u32,
    pub(crate) domain: String,
    pub(crate) wordlist_path: String,
    pub(crate) concurrent_limit: u32,
}

#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1/",
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
                                    WsMessage::SubdomainEnumerationResult {
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
