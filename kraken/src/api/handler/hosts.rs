use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::{get, put, HttpResponse};
use futures::TryStreamExt;
use rorm::conditions::DynamicCollection;
use rorm::prelude::*;
use rorm::{and, insert, query, update, Database};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::handler::{workspaces, ApiError, ApiResult, PathUuid, SimpleTag, TagType};
use crate::models::{GlobalTag, Host, HostGlobalTag, HostWorkspaceTag, OsType, WorkspaceTag};

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

/// The full representation of a host
#[derive(Serialize, Debug, ToSchema)]
pub struct FullHost {
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
    /// The list of tags this host has attached to
    pub tags: Vec<SimpleTag>,
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

/// Retrieve all information about a single host
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieved the selected host", body = FullHost),
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
) -> ApiResult<Json<FullHost>> {
    let path = path.into_inner();

    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    if !workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.w_uuid).await? {
        return Err(ApiError::MissingPrivileges)?;
    }

    let host = query!(&mut tx, Host)
        .condition(and!(
            Host::F.workspace.equals(path.w_uuid),
            Host::F.uuid.equals(path.h_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let mut tags: Vec<_> = query!(&mut tx, (HostGlobalTag::F.global_tag as GlobalTag,))
        .condition(HostGlobalTag::F.host.equals(host.uuid))
        .stream()
        .map_ok(|(x,)| SimpleTag {
            uuid: x.uuid,
            name: x.name,
            color: x.color.into(),
            tag_type: TagType::Global,
        })
        .try_collect()
        .await?;

    let global_tags: Vec<_> = query!(
        &mut tx,
        (HostWorkspaceTag::F.workspace_tag as WorkspaceTag,)
    )
    .condition(HostWorkspaceTag::F.host.equals(host.uuid))
    .stream()
    .map_ok(|(x,)| SimpleTag {
        uuid: x.uuid,
        name: x.name,
        color: x.color.into(),
        tag_type: TagType::Workspace,
    })
    .try_collect()
    .await?;

    tags.extend(global_tags);

    tx.commit().await?;

    Ok(Json(FullHost {
        uuid: host.uuid,
        ip_addr: host.ip_addr.ip().to_string(),
        workspace: *host.workspace.key(),
        os_type: host.os_type,
        comment: host.comment,
        tags,
    }))
}

/// The request to update a host
#[derive(Deserialize, ToSchema)]
pub struct UpdateHostRequest {
    comment: Option<String>,
    global_tags: Option<Vec<Uuid>>,
    workspace_tags: Option<Vec<Uuid>>,
}

/// Update a host
///
/// You must include at least on parameter
#[utoipa::path(
    tag = "Hosts",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Host was updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = UpdateHostRequest,
    params(PathHost),
    security(("api_key" = []))
)]
#[put("/workspaces/{w_uuid}/hosts/{h_uuid}")]
pub async fn update_host(
    req: Json<UpdateHostRequest>,
    path: Path<PathHost>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<HttpResponse> {
    let path = path.into_inner();
    let req = req.into_inner();
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    if req.workspace_tags.is_none() && req.global_tags.is_none() && req.comment.is_none() {
        return Err(ApiError::EmptyJson);
    }

    let mut tx = db.start_transaction().await?;

    if !workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.w_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (Host::F.uuid,))
        .condition(Host::F.uuid.equals(path.h_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if let Some(global_tags) = req.global_tags {
        let cond = DynamicCollection::or(
            global_tags
                .iter()
                .map(|x| GlobalTag::F.uuid.equals(*x))
                .collect(),
        );

        let res = query!(&mut tx, GlobalTag).condition(cond).all().await?;
        if res.len() != global_tags.len() {
            return Err(ApiError::InvalidUuid);
        }

        rorm::delete!(&mut tx, HostGlobalTag)
            .condition(HostGlobalTag::F.host.equals(path.h_uuid))
            .await?;

        insert!(&mut tx, HostGlobalTag)
            .return_nothing()
            .bulk(
                &global_tags
                    .into_iter()
                    .map(|x| HostGlobalTag {
                        uuid: Uuid::new_v4(),
                        host: ForeignModelByField::Key(path.h_uuid),
                        global_tag: ForeignModelByField::Key(x),
                    })
                    .collect::<Vec<_>>(),
            )
            .await?;
    }

    if let Some(workspace_tags) = req.workspace_tags {
        let cond = DynamicCollection::or(
            workspace_tags
                .iter()
                .map(|x| GlobalTag::F.uuid.equals(*x))
                .collect(),
        );

        let res = query!(&mut tx, GlobalTag).condition(cond).all().await?;
        if res.len() != workspace_tags.len() {
            return Err(ApiError::InvalidUuid);
        }

        rorm::delete!(&mut tx, HostWorkspaceTag)
            .condition(HostWorkspaceTag::F.host.equals(path.h_uuid))
            .await?;

        insert!(&mut tx, HostWorkspaceTag)
            .return_nothing()
            .bulk(
                &workspace_tags
                    .into_iter()
                    .map(|x| HostWorkspaceTag {
                        uuid: Uuid::new_v4(),
                        host: ForeignModelByField::Key(path.h_uuid),
                        workspace_tag: ForeignModelByField::Key(x),
                    })
                    .collect::<Vec<_>>(),
            )
            .await?;
    }

    if let Some(comment) = req.comment {
        update!(&mut tx, Host)
            .condition(Host::F.uuid.equals(path.h_uuid))
            .set(Host::F.comment, comment)
            .exec()
            .await?;
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
