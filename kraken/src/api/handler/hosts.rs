use actix_toolbox::tb_middleware::Session;
use actix_web::get;
use actix_web::web::{Data, Json, Path};
use rorm::prelude::*;
use rorm::{and, query, Database};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::handler::{workspaces, ApiError, ApiResult, PathUuid};
use crate::models::{Host, OsType};

/// The simple representation of a host
#[derive(Serialize, Debug, ToSchema)]
pub struct SimpleHost {
    /// The primary key of the host
    pub uuid: Uuid,
    /// The ip address of the host
    #[schema(example = "172.0.0.1")]
    pub ip_addr: String,
    /// The type of OS
    pub os_type: OsType,
    /// A comment
    pub comment: String,
    /// The workspace this host is in
    pub workspace: Uuid,
}

/// The reseponse to a get all hosts reqeust
#[derive(Serialize, Debug, ToSchema)]
pub struct GetAllHostsResponse {
    pub(crate) hosts: Vec<SimpleHost>,
}

/// Retrieve all hosts.
///
/// Hosts are created out of aggregating data or by user input.
/// They represent a single host and can be created by providing an IP address
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "All hosts in the workspace", body = GetAllHostsResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/hosts")]
pub async fn get_all_hosts(
    path: Path<PathUuid>,
    session: Session,
    db: Data<Database>,
) -> ApiResult<Json<GetAllHostsResponse>> {
    let path = path.into_inner();

    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    if workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.uuid).await? {
        let hosts = query!(&mut tx, Host)
            .condition(Host::F.workspace.equals(path.uuid))
            .all()
            .await?;

        tx.commit().await?;

        Ok(Json(GetAllHostsResponse {
            hosts: hosts
                .into_iter()
                .map(|x| SimpleHost {
                    uuid: x.uuid,
                    ip_addr: x.ip_addr.ip().to_string(),
                    comment: x.comment,
                    os_type: x.os_type,
                    workspace: *x.workspace.key(),
                })
                .collect(),
        }))
    } else {
        Err(ApiError::MissingPrivileges)
    }
}

/// The path parameter of a host
#[derive(Deserialize, IntoParams)]
pub struct PathHost {
    w_uuid: Uuid,
    h_uuid: Uuid,
}

#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieved the selected host", body = SimpleHost),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathHost),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/hosts{h_uuid}")]
pub async fn get_host(
    path: Path<PathHost>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<SimpleHost>> {
    let path = path.into_inner();

    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    if workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.w_uuid).await? {
        let host = query!(&mut tx, Host)
            .condition(and!(
                Host::F.workspace.equals(path.w_uuid),
                Host::F.uuid.equals(path.h_uuid)
            ))
            .optional()
            .await?
            .ok_or(ApiError::InvalidUuid)?;

        tx.commit().await?;

        Ok(Json(SimpleHost {
            uuid: host.uuid,
            ip_addr: host.ip_addr.ip().to_string(),
            workspace: *host.workspace.key(),
            os_type: host.os_type,
            comment: host.comment,
        }))
    } else {
        Err(ApiError::MissingPrivileges)
    }
}
