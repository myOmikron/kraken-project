use std::collections::HashMap;
use std::net::IpAddr;

use actix_web::get;
use actix_web::web::{Data, Json, Path};
use chrono::{DateTime, TimeZone, Utc};
use futures::StreamExt;
use rorm::{query, Database, Model};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{ApiError, ApiResult, PathId};
use crate::models::{Attack, TcpPortScanResult, Workspace};

#[derive(Serialize, ToSchema)]
pub(crate) struct ReportingWorkspaceResults {
    /// List of all tcp port scan attacks
    tcp_port_scan_attacks: Vec<ReportingTcpPortScanAttack>,

    /// List of user which started attacks in this workspace
    attacker: Vec<ReportingUser>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ReportingTcpPortScanAttack {
    /// The attacks database id
    #[schema(example = 1337)]
    id: i64,

    /// When the attack was created i.e. started
    created_at: DateTime<Utc>,

    /// When the leech signaled to be finished with the attack
    finished_at: DateTime<Utc>,

    /// List of found (ip, port) - pairs
    results: Vec<ReportingIpPort>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ReportingUser {
    /// The user's id
    pub(crate) uuid: Uuid,

    /// The user's login name
    #[schema(example = "james1337")]
    pub(crate) username: String,

    /// The user's legal name
    #[schema(example = "James Smith")]
    pub(crate) display_name: String,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ReportingIpPort {
    /// Ip address (v4 or v6)
    #[schema(value_type = String, example = "10.13.37.1")]
    pub(crate) ip: IpAddr,

    /// Port number
    #[schema(example = 80)]
    pub(crate) port: u16,
}

/// Retrieve a workspace's attack results
#[utoipa::path(
    tag = "Reporting data export",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns the user", body = ReportingWorkspaceResults),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathId),
    security(("api_key" = []))
)]
#[get("/reporting/{id}")]
pub(crate) async fn report_workspace_results(
    req: Path<PathId>,
    db: Data<Database>,
) -> ApiResult<Json<ReportingWorkspaceResults>> {
    let id = req.into_inner().id as i64;
    let mut tx = db.start_transaction().await?;

    let mut attackers: HashMap<Uuid, ReportingUser> = HashMap::new();

    // Check workspace to exist
    let (_,) = query!(&mut tx, (Workspace::F.id,))
        .condition(Workspace::F.id.equals(id))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    // Query all tcp port scan results
    let mut tcp_port_scan_results: HashMap<i64, Vec<ReportingIpPort>> = HashMap::new();
    let mut stream = query!(
        &mut tx,
        (
            TcpPortScanResult::F.attack,
            TcpPortScanResult::F.address,
            TcpPortScanResult::F.port
        )
    )
    .condition(TcpPortScanResult::F.attack.workspace.equals(id))
    .stream();
    while let Some(result) = stream.next().await {
        let (attack, address, port) = result?;
        tcp_port_scan_results
            .entry(*attack.key())
            .or_default()
            .push(ReportingIpPort {
                ip: address.into_inner(),
                port: port as u16,
            });
    }
    drop(stream);

    // Query all tcp port scan attacks and combine them with their results
    let mut tcp_port_scan_attacks = Vec::with_capacity(tcp_port_scan_results.len());
    let mut stream = query!(
        &mut tx,
        (
            Attack::F.id,
            Attack::F.created_at,
            Attack::F.finished_at,
            Attack::F.started_from.uuid,
            Attack::F.started_from.username,
            Attack::F.started_from.display_name
        )
    )
    .condition(Attack::F.workspace.equals(id))
    .stream();
    while let Some(result) = stream.next().await {
        let (attack, created_at, finished_at, uuid, username, display_name) = result?;
        if let Some(finished_at) = finished_at {
            tcp_port_scan_attacks.push(ReportingTcpPortScanAttack {
                id: attack,
                created_at: Utc.from_utc_datetime(&created_at),
                finished_at: Utc.from_utc_datetime(&finished_at),
                results: tcp_port_scan_results.remove(&attack).unwrap_or_default(),
            });
        }
        attackers.entry(uuid).or_insert(ReportingUser {
            uuid,
            username,
            display_name,
        });
    }
    drop(stream);

    Ok(Json(ReportingWorkspaceResults {
        tcp_port_scan_attacks,
        attacker: attackers.into_values().collect(),
    }))
}
