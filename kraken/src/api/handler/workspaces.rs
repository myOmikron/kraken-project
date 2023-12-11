//! Everything regarding workspace management is located in this module

use actix_web::web::{Json, Path, Query};
use actix_web::{delete, get, post, put, HttpResponse};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use log::{debug, error, info};
use rorm::db::executor::Stream;
use rorm::db::sql::value::Value;
use rorm::db::transaction::Transaction;
use rorm::db::Executor;
use rorm::prelude::ForeignModelByField;
use rorm::{and, insert, query, update, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::attack_results::{
    FullQueryCertificateTransparencyResult, FullServiceDetectionResult, SimpleDnsResolutionResult,
    SimpleHostAliveResult, SimpleQueryUnhashedResult, SimpleTcpPortScanResult,
};
use crate::api::handler::attacks::SimpleAttack;
use crate::api::handler::domains::SimpleDomain;
use crate::api::handler::hosts::SimpleHost;
use crate::api::handler::ports::SimplePort;
use crate::api::handler::services::SimpleService;
use crate::api::handler::users::SimpleUser;
use crate::api::handler::workspace_invitations::{
    FullWorkspaceInvitation, WorkspaceInvitationList,
};
use crate::api::handler::{
    de_optional, ApiError, ApiResult, Page, PageParams, PathUuid, SearchResultPage,
    SearchesResultPage, UuidResponse,
};
use crate::chan::{WsMessage, GLOBAL};
use crate::models::{
    Attack, CertificateTransparencyResult, CertificateTransparencyValueName, DehashedQueryResult,
    DnsResolutionResult, Domain, Host, HostAliveResult, ModelType, Port, Search, SearchInsert,
    SearchResult, Service, TcpPortScanResult, User, UserPermission, Workspace, WorkspaceInvitation,
    WorkspaceMember,
};

/// The request to create a new workspace
#[derive(Deserialize, ToSchema)]
pub struct CreateWorkspaceRequest {
    #[schema(example = "secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is super secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
}

/// Create a new workspace
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateWorkspaceRequest,
    security(("api_key" = []))
)]
#[post("/workspaces")]
pub async fn create_workspace(
    req: Json<CreateWorkspaceRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<UuidResponse>> {
    let req = req.into_inner();

    let uuid = Workspace::insert(&GLOBAL.db, req.name, req.description, user_uuid).await?;

    Ok(Json(UuidResponse { uuid }))
}

/// Delete a workspace by its id
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/workspaces/{uuid}")]
pub async fn delete_workspace(
    req: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let executing_user = query!(&mut tx, User)
        .condition(User::F.uuid.equals(user_uuid))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    let workspace = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(req.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if executing_user.permission == UserPermission::Admin
        || *workspace.owner.key() == executing_user.uuid
    {
        debug!(
            "Workspace {} got deleted by {}",
            workspace.uuid, executing_user.username
        );

        rorm::delete!(&mut tx, Workspace).single(&workspace).await?;
    } else {
        debug!(
            "User {} does not have the privileges to delete the workspace {}",
            executing_user.username, workspace.uuid
        );

        return Err(ApiError::MissingPrivileges);
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// A simple version of a workspace
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SimpleWorkspace {
    pub(crate) uuid: Uuid,
    #[schema(example = "ultra-secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
    pub(crate) owner: SimpleUser,
    pub(crate) created_at: DateTime<Utc>,
}

/// A full version of a workspace
#[derive(Serialize, ToSchema)]
pub struct FullWorkspace {
    pub(crate) uuid: Uuid,
    #[schema(example = "ultra-secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
    pub(crate) owner: SimpleUser,
    pub(crate) attacks: Vec<SimpleAttack>,
    pub(crate) members: Vec<SimpleUser>,
    pub(crate) created_at: DateTime<Utc>,
}

/// Retrieve a workspace by id
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns the workspace", body = FullWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}")]
pub async fn get_workspace(
    req: Path<PathUuid>,

    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<FullWorkspace>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let workspace = if Workspace::is_user_member_or_owner(&mut tx, req.uuid, user_uuid).await? {
        get_workspace_unchecked(req.uuid, &mut tx).await
    } else {
        Err(ApiError::MissingPrivileges)
    };

    tx.commit().await?;

    Ok(Json(workspace?))
}

/// The response to retrieve all workspaces
#[derive(Serialize, ToSchema)]
pub struct GetAllWorkspacesResponse {
    pub(crate) workspaces: Vec<SimpleWorkspace>,
}

/// Retrieve all workspaces that the executing user has access to
///
/// For administration access, look at the `/admin/workspaces` endpoint.
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns all workspaces that the executing user has access to", body = GetAllWorkspacesResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub async fn get_all_workspaces(
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<GetAllWorkspacesResponse>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let session_user = query!(&mut tx, User)
        .condition(User::F.uuid.equals(user_uuid))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    let mut workspaces: Vec<(Workspace, SimpleUser)> = query!(&mut tx, Workspace)
        .condition(Workspace::F.owner.equals(session_user.uuid))
        .stream()
        .map_ok(|x| {
            (
                x,
                SimpleUser {
                    uuid: session_user.uuid,
                    username: session_user.username.clone(),
                    display_name: session_user.display_name.clone(),
                },
            )
        })
        .try_collect()
        .await?;

    let w: Vec<(Workspace, SimpleUser)> = query!(
        &mut tx,
        (
            WorkspaceMember::F.workspace as Workspace,
            WorkspaceMember::F.workspace.owner as SimpleUser
        )
    )
    .condition(WorkspaceMember::F.member.equals(session_user.uuid))
    .all()
    .await?;

    tx.commit().await?;

    workspaces.extend(w);

    Ok(Json(GetAllWorkspacesResponse {
        workspaces: workspaces
            .into_iter()
            .map(|(w, owner)| SimpleWorkspace {
                uuid: w.uuid,
                name: w.name,
                description: w.description,
                owner,
                created_at: w.created_at,
            })
            .collect(),
    }))
}

/// The request type to update a workspace
///
/// All parameter are optional, but at least one of them must be specified
#[derive(Deserialize, ToSchema)]
pub struct UpdateWorkspaceRequest {
    #[schema(example = "Workspace for work")]
    name: Option<String>,
    #[schema(example = "This workspace is for work and for work only!")]
    #[serde(deserialize_with = "de_optional")]
    description: Option<Option<String>>,
}

/// Updates a workspace by its id
///
/// All parameter are optional, but at least one of them must be specified.
///
/// `name` must not be empty.
///
/// You can set `description` to null to remove the description from the database.
/// If you leave the parameter out, the description will remain unchanged.
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace got updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    request_body = UpdateWorkspaceRequest,
    security(("api_key" = []))
)]
#[put("/workspaces/{uuid}")]
pub async fn update_workspace(
    path: Path<PathUuid>,
    req: Json<UpdateWorkspaceRequest>,

    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let req = req.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    let w = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(path.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if *w.owner.key() != user_uuid {
        return Err(ApiError::MissingPrivileges);
    }

    if let Some(name) = &req.name {
        if name.is_empty() {
            return Err(ApiError::InvalidName);
        }
    }

    update!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(w.uuid))
        .begin_dyn_set()
        .set_if(Workspace::F.name, req.name)
        .set_if(Workspace::F.description, req.description)
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .exec()
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// The request to transfer a workspace to another account
#[derive(Debug, ToSchema, Deserialize)]
pub struct TransferWorkspaceRequest {
    /// The uuid of the user that should receive the workspace
    pub user: Uuid,
}

/// Transfer ownership to another account
///
/// You will loose access to the workspace.
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace was transferred"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/transfer")]
pub async fn transfer_ownership(
    req: Json<TransferWorkspaceRequest>,
    path: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let new_owner_uuid = req.into_inner().user;
    let workspace_uuid = path.into_inner().uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;

    let Some(workspace) = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(workspace_uuid))
        .optional()
        .await?
    else {
        return Err(ApiError::MissingPrivileges);
    };

    if *workspace.owner.key() != user_uuid {
        return Err(ApiError::MissingPrivileges);
    }

    update!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(workspace_uuid))
        .set(Workspace::F.owner, ForeignModelByField::Key(new_owner_uuid))
        .exec()
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// The request to invite a user to the workspace
#[derive(Deserialize, Debug, ToSchema)]
pub struct InviteToWorkspace {
    /// The user to invite
    pub user: Uuid,
}

/// Invite a user to the workspace
///
/// This action can only be invoked by the owner of a workspace
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The user was invited."),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    request_body = InviteToWorkspace,
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/invitations")]
pub async fn create_invitation(
    req: Json<InviteToWorkspace>,
    path: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let InviteToWorkspace { user } = req.into_inner();
    let workspace_uuid = path.into_inner().uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;
    let session_user = query!(&mut tx, User)
        .condition(User::F.uuid.equals(user_uuid))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    let invitation_uuid =
        WorkspaceInvitation::insert(&mut tx, workspace_uuid, session_user.uuid, user).await?;

    let workspace = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(workspace_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let (owner,) = query!(&mut tx, (Workspace::F.owner as SimpleUser))
        .condition(Workspace::F.uuid.equals(workspace_uuid))
        .one()
        .await?;

    tx.commit().await?;

    GLOBAL
        .ws
        .message(
            user,
            WsMessage::InvitationToWorkspace {
                invitation_uuid,
                from: SimpleUser {
                    uuid: session_user.uuid,
                    username: session_user.username,
                    display_name: session_user.display_name,
                },
                workspace: SimpleWorkspace {
                    uuid: workspace_uuid,
                    name: workspace.name,
                    description: workspace.description,
                    created_at: workspace.created_at,
                    owner,
                },
            },
        )
        .await;

    Ok(HttpResponse::Ok().finish())
}

/// The url components of an invitation
#[derive(Deserialize, IntoParams)]
pub struct InviteUuid {
    /// The UUID of the workspace
    pub w_uuid: Uuid,
    /// The UUID of the invitation
    pub i_uuid: Uuid,
}

/// Retract an invitation to the workspace
///
/// This action can only be invoked by the owner of a workspace
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The invitation was retracted."),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(InviteUuid),
    security(("api_key" = []))
)]
#[delete("/workspaces/{w_uuid}/invitations/{i_uuid}")]
pub async fn retract_invitation(
    path: Path<InviteUuid>,

    SessionUser(session_user): SessionUser,
) -> ApiResult<HttpResponse> {
    let InviteUuid { w_uuid, i_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    let workspace = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(w_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidWorkspace)?;

    if *workspace.owner.key() != session_user {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (WorkspaceInvitation::F.uuid,))
        .condition(and!(
            WorkspaceInvitation::F.uuid.equals(i_uuid),
            WorkspaceInvitation::F.workspace.equals(w_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidInvitation)?;

    rorm::delete!(&mut tx, WorkspaceInvitation)
        .condition(WorkspaceInvitation::F.uuid.equals(i_uuid))
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// Query all open invitations to a workspace
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns all open invitations to the workspace.", body = WorkspaceInvitationList),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/invitations")]
pub async fn get_all_workspace_invitations(
    path: Path<PathUuid>,

    SessionUser(session_user): SessionUser,
) -> ApiResult<Json<WorkspaceInvitationList>> {
    let workspace_uuid = path.into_inner().uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;

    let workspace = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(workspace_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidWorkspace)?;

    if *workspace.owner.key() != session_user {
        return Err(ApiError::MissingPrivileges);
    }

    let invitations = query!(
        &mut tx,
        (
            WorkspaceInvitation::F.uuid,
            WorkspaceInvitation::F.workspace as Workspace,
            WorkspaceInvitation::F.from as SimpleUser,
            WorkspaceInvitation::F.target as SimpleUser,
            WorkspaceInvitation::F.workspace.owner as SimpleUser
        )
    )
    .condition(WorkspaceInvitation::F.workspace.equals(workspace_uuid))
    .all()
    .await?
    .into_iter()
    .map(
        |(uuid, workspace, from, target, owner)| FullWorkspaceInvitation {
            uuid,
            workspace: SimpleWorkspace {
                uuid: workspace.uuid,
                owner,
                name: workspace.name,
                description: workspace.description,
                created_at: workspace.created_at,
            },
            from,
            target,
        },
    )
    .collect();

    tx.commit().await?;

    Ok(Json(WorkspaceInvitationList { invitations }))
}

/// Retrieve a workspace by id
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns the workspace with the given id", body = FullWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}")]
pub async fn get_workspace_admin(req: Path<PathUuid>) -> ApiResult<Json<FullWorkspace>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let workspace = get_workspace_unchecked(req.uuid, &mut tx).await;

    tx.commit().await?;

    Ok(Json(workspace?))
}

/// Retrieve all workspaces
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns all workspaces", body = GetAllWorkspacesResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub async fn get_all_workspaces_admin() -> ApiResult<Json<GetAllWorkspacesResponse>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let workspaces = query!(
        &mut tx,
        (
            Workspace::F.uuid,
            Workspace::F.name,
            Workspace::F.description,
            Workspace::F.created_at,
            Workspace::F.owner.uuid,
            Workspace::F.owner.username,
            Workspace::F.owner.display_name
        )
    )
    .all()
    .await?;

    tx.commit().await?;

    Ok(Json(GetAllWorkspacesResponse {
        workspaces: workspaces
            .into_iter()
            .map(
                |(uuid, name, description, created_at, by_uuid, username, display_name)| {
                    SimpleWorkspace {
                        uuid,
                        name,
                        description,
                        owner: SimpleUser {
                            uuid: by_uuid,
                            username,
                            display_name,
                        },
                        created_at,
                    }
                },
            )
            .collect(),
    }))
}

/// Request to search the workspace
#[derive(Deserialize, ToSchema)]
pub struct SearchWorkspaceRequest {
    /// the term to search for
    pub(crate) search_term: String,
}

/// Search through a workspaces' data
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Search has been scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    request_body = SearchWorkspaceRequest,
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/search")]
pub async fn search(
    path: Path<PathUuid>,
    request: Json<SearchWorkspaceRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let search_term = request.into_inner().search_term;

    if search_term.is_empty() {
        return Err(ApiError::InvalidSearch);
    }

    let mut db_trx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut db_trx, path.uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    info!("Started workspace search for: '{}'", search_term);

    let search_uuid = insert!(&mut db_trx, Search)
        .return_primary_key()
        .single(&SearchInsert {
            uuid: Uuid::new_v4(),
            started_by: ForeignModelByField::Key(user_uuid),
            workspace: ForeignModelByField::Key(path.uuid),
            search_term: search_term.clone(),
        })
        .await?;

    db_trx.commit().await.map_err(ApiError::DatabaseError)?;

    tokio::spawn({
        let search_term = search_term.clone();
        async move {
            let finished_successful = if let Err(error) =
                run_search(&search_term, path.uuid, search_uuid, user_uuid).await
            {
                if update!(&GLOBAL.db, Search)
                    .condition(Search::F.uuid.equals(search_uuid))
                    .set(Search::F.error, Some(error.to_string()))
                    .set(Search::F.finished_at, Some(Utc::now()))
                    .await
                    .is_err()
                {
                    error!("could not insert error msg into database");
                }
                false
            } else {
                true
            };

            GLOBAL
                .ws
                .message(
                    user_uuid,
                    WsMessage::SearchFinished {
                        search_uuid,
                        finished_successful,
                    },
                )
                .await;

            info!("Finished workspace search for: '{search_term}'");
        }
    });

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: search_uuid }))
}

/// Searched entry
#[derive(Serialize, ToSchema)]
pub struct SearchEntry {
    pub(crate) uuid: Uuid,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) finished_at: Option<DateTime<Utc>>,
    pub(crate) search_term: String,
}

/// Query all searches
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Search results", body = SearchesResultPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/search")]
pub async fn get_searches(
    path: Path<PathUuid>,
    request: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<SearchesResultPage>> {
    let PathUuid { uuid } = path.into_inner();
    let PageParams { limit, offset } = request.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    };

    let (total,) = query!(&mut tx, (Search::F.uuid.count(),)).one().await?;

    let items = query!(&mut tx, Search)
        .condition(Search::F.workspace.equals(uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map_ok(|entry| SearchEntry {
            uuid: entry.uuid,
            created_at: entry.created_at,
            finished_at: entry.finished_at,
            search_term: entry.search_term,
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(Page {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

// TODO: unify with 'InviteUuid'
/// The url components of an search
#[derive(Deserialize, IntoParams)]
pub struct SearchUuid {
    /// The UUID of the workspace
    pub w_uuid: Uuid,
    /// The UUID of the search
    pub s_uuid: Uuid,
}

// pub struct

/// Dynamic result
#[derive(Serialize, ToSchema)]
pub enum SearchResultEntry {
    /// Host Result
    HostEntry(SimpleHost),
    /// Service Result
    ServiceEntry(SimpleService),
    /// Port Result
    PortEntry(SimplePort),
    /// Domain Result
    DomainEntry(SimpleDomain),
    /// DNS Record Result
    DnsRecordResultEntry(SimpleDnsResolutionResult),
    /// TCP Port Result
    TcpPortScanResultEntry(SimpleTcpPortScanResult),
    /// Dehashed Query Result
    DehashedQueryResultEntry(SimpleQueryUnhashedResult),
    /// Certificate Transparency Result
    CertificateTransparencyResultEntry(FullQueryCertificateTransparencyResult),
    /// Host Alive Result
    HostAliveResult(SimpleHostAliveResult),
    /// Service Detection Result
    ServiceDetectionResult(FullServiceDetectionResult),
}

/// Retrieve results for a search by it's uuid
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Search results", body = SearchResultPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(SearchUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/workspaces/{w_uuid}/search/{s_uuid}")]
pub async fn get_search_results(
    path: Path<SearchUuid>,
    request: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<SearchResultPage>> {
    let SearchUuid { w_uuid, s_uuid } = path.into_inner();
    let PageParams { limit, offset } = request.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    };

    let (total,) = query!(&mut tx, (SearchResult::F.uuid.count(),))
        .condition(SearchResult::F.search.equals(s_uuid))
        .one()
        .await?;

    let proxy_items = query!(&mut tx, SearchResult)
        .condition(and!(
            SearchResult::F.search.equals(s_uuid),
            SearchResult::F.search.workspace.equals(w_uuid),
        ))
        .limit(limit)
        .offset(offset)
        .all()
        .await?;

    let mut items = Vec::with_capacity(proxy_items.len());

    for item in proxy_items {
        items.push(match item.ref_type {
            ModelType::Host => {
                let data = query!(&mut tx, Host)
                    .condition(Host::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                SearchResultEntry::HostEntry(SimpleHost {
                    uuid: data.uuid,
                    workspace: *data.workspace.key(),
                    comment: data.comment,
                    os_type: data.os_type,
                    ip_addr: data.ip_addr.to_string(),
                    created_at: data.created_at,
                })
            }
            ModelType::Service => {
                let data = query!(&mut tx, Service)
                    .condition(Service::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                let port = data.port.map(|x| *x.key());

                SearchResultEntry::ServiceEntry(SimpleService {
                    uuid: data.uuid,
                    name: data.name,
                    version: data.version,
                    host: *data.host.key(),
                    comment: data.comment,
                    workspace: *data.workspace.key(),
                    created_at: data.created_at,
                    port,
                })
            }
            ModelType::Port => {
                let data = query!(&mut tx, Port)
                    .condition(Port::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                SearchResultEntry::PortEntry(SimplePort {
                    uuid: data.uuid,
                    comment: data.comment,
                    workspace: *data.workspace.key(),
                    port: data.port as u16,
                    created_at: data.created_at,
                    host: *data.host.key(),
                    protocol: data.protocol,
                })
            }
            ModelType::Domain => {
                let data = query!(&mut tx, Domain)
                    .condition(Domain::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                SearchResultEntry::DomainEntry(SimpleDomain {
                    uuid: data.uuid,
                    comment: data.comment,
                    workspace: *data.workspace.key(),
                    created_at: data.created_at,
                    domain: data.domain,
                })
            }
            ModelType::DnsRecordResult => {
                let data = query!(&mut tx, DnsResolutionResult)
                    .condition(DnsResolutionResult::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                SearchResultEntry::DnsRecordResultEntry(SimpleDnsResolutionResult {
                    uuid: data.uuid,
                    created_at: data.created_at,
                    attack: *data.attack.key(),
                    source: data.source,
                    destination: data.destination,
                    dns_record_type: data.dns_record_type,
                })
            }
            ModelType::TcpPortScanResult => {
                let data = query!(&mut tx, TcpPortScanResult)
                    .condition(TcpPortScanResult::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                SearchResultEntry::TcpPortScanResultEntry(SimpleTcpPortScanResult {
                    uuid: data.uuid,
                    created_at: data.created_at,
                    attack: *data.attack.key(),
                    address: data.address,
                    port: data.port as u16,
                })
            }
            ModelType::DehashedQueryResult => {
                let data = query!(&mut tx, DehashedQueryResult)
                    .condition(DehashedQueryResult::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                SearchResultEntry::DehashedQueryResultEntry(SimpleQueryUnhashedResult {
                    uuid: data.uuid,
                    created_at: data.created_at,
                    attack: *data.attack.key(),
                    address: data.address,
                    phone: data.phone,
                    database_name: data.database_name,
                    dehashed_id: data.dehashed_id,
                    hashed_password: data.hashed_password,
                    ip_address: data.ip_address,
                    email: data.email,
                    password: data.password,
                    username: data.username,
                    vin: data.vin,
                    name: data.name,
                })
            }
            ModelType::CertificateTransparencyResult => {
                let mut data = query!(&mut tx, CertificateTransparencyResult)
                    .condition(CertificateTransparencyResult::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                if CertificateTransparencyResult::F
                    .value_names
                    .populate(&mut tx, &mut data)
                    .await
                    .is_err()
                {
                    error!("could not resolve backref's");
                }

                let names: Vec<CertificateTransparencyValueName> =
                    data.value_names.cached.unwrap_or(vec![]);

                SearchResultEntry::CertificateTransparencyResultEntry(
                    FullQueryCertificateTransparencyResult {
                        uuid: data.uuid,
                        created_at: data.created_at,
                        attack: *data.attack.key(),
                        common_name: data.common_name,
                        value_names: names.into_iter().map(|entry| entry.value_name).collect(),
                        issuer_name: data.issuer_name,
                        serial_number: data.serial_number,
                        not_before: data.not_before,
                        not_after: data.not_after,
                    },
                )
            }
            ModelType::HostAliveResult => {
                let data = query!(&mut tx, HostAliveResult)
                    .condition(HostAliveResult::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                SearchResultEntry::HostAliveResult(SimpleHostAliveResult {
                    uuid: data.uuid,
                    created_at: data.created_at,
                    attack: *data.attack.key(),
                    host: data.host,
                })
            }
            ModelType::ServiceDetectionResult => {
                let data = query!(&mut tx, HostAliveResult)
                    .condition(HostAliveResult::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                SearchResultEntry::HostAliveResult(SimpleHostAliveResult {
                    uuid: data.uuid,
                    created_at: data.created_at,
                    attack: *data.attack.key(),
                    host: data.host,
                })
            }
        })
    }

    tx.commit().await?;

    Ok(Json(Page {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

fn build_query_list() -> Vec<(String, ModelType)> {
    let table_names_no_ref_to_ws = vec![
        ModelType::DnsRecordResult,
        ModelType::TcpPortScanResult,
        ModelType::DehashedQueryResult,
        ModelType::CertificateTransparencyResult,
        ModelType::HostAliveResult,
        ModelType::ServiceDetectionResult,
    ];

    let table_names_ref_to_ws = vec![
        ModelType::Host,
        ModelType::Service,
        ModelType::Port,
        ModelType::Domain,
    ];

    let mut data = Vec::with_capacity(table_names_no_ref_to_ws.len() + table_names_ref_to_ws.len());

    data.extend(table_names_no_ref_to_ws.into_iter().map(|table_entry| {
        (format!(
            "SELECT
                workspace_related_table.uuid
            FROM
                (SELECT t.* FROM \"{table_entry}\" t JOIN attack on t.attack = attack.uuid WHERE attack.workspace = $1) workspace_related_table
            WHERE
                (workspace_related_table.*)::text ILIKE $2;"
        ), table_entry)
    }).collect::<Vec<(String, ModelType)>>());

    data.extend(
        table_names_ref_to_ws
            .into_iter()
            .map(|table_entry| {
                (format!(
                        "SELECT
                            workspace_related_table.uuid
                        FROM
                            (SELECT t.* FROM \"{table_entry}\" t WHERE t.workspace = $1) workspace_related_table
                        WHERE
                            (workspace_related_table.*)::text ILIKE $2;"
                    ),
                    table_entry,
                )
            })
            .collect::<Vec<(String, ModelType)>>(),
    );

    data
}

async fn run_search(
    search_term: &str,
    workspace_uuid: Uuid,
    search_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<(), rorm::Error> {
    let search_term = format!("%{search_term}%");

    for (sql, model_type) in build_query_list() {
        debug!("search sql: {sql}");
        let mut stream = GLOBAL.db.execute::<Stream>(
            sql,
            vec![Value::Uuid(workspace_uuid), Value::String(&search_term)],
        );

        while let Some(row) = stream.try_next().await? {
            let ref_key: Uuid = row.get(0)?;

            debug!("received search result with key: {ref_key:?}");
            let result_uuid = insert!(&GLOBAL.db, SearchResult)
                .return_primary_key()
                .single(&SearchResult {
                    uuid: Uuid::new_v4(),
                    ref_key,
                    ref_type: model_type,
                    search: ForeignModelByField::Key(search_uuid),
                })
                .await?;

            GLOBAL
                .ws
                .message(
                    user_uuid,
                    WsMessage::SearchNotify {
                        search_uuid,
                        result_uuid,
                    },
                )
                .await;
        }
    }

    update!(&GLOBAL.db, Search)
        .condition(Search::F.uuid.equals(search_uuid))
        .set(Search::F.finished_at, Some(Utc::now()))
        .await?;

    Ok(())
}

/// Get a [`FullWorkspace`] by its uuid without permission checks
async fn get_workspace_unchecked(uuid: Uuid, tx: &mut Transaction) -> ApiResult<FullWorkspace> {
    let workspace = query!(&mut *tx, Workspace)
        .condition(Workspace::F.uuid.equals(uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let owner = query!(&mut *tx, SimpleUser)
        .condition(User::F.uuid.equals(*workspace.owner.key()))
        .one()
        .await?;

    let attacks = query!(
        &mut *tx,
        (
            Attack::F.uuid,
            Attack::F.attack_type,
            Attack::F.finished_at,
            Attack::F.created_at,
            Attack::F.started_by as SimpleUser,
            Attack::F.error,
        )
    )
    .condition(Attack::F.workspace.equals(uuid))
    .all()
    .await?
    .into_iter()
    .map(
        |(attack_uuid, attack_type, finished_at, created_at, started_by, error)| SimpleAttack {
            uuid: attack_uuid,
            workspace: SimpleWorkspace {
                uuid: workspace.uuid,
                name: workspace.name.clone(),
                description: workspace.description.clone(),
                created_at: workspace.created_at,
                owner: owner.clone(),
            },
            attack_type,
            started_by,
            finished_at,
            created_at,
            error,
        },
    )
    .collect();

    let members = query!(
        &mut *tx,
        (
            WorkspaceMember::F.member.uuid,
            WorkspaceMember::F.member.username,
            WorkspaceMember::F.member.display_name
        )
    )
    .condition(WorkspaceMember::F.workspace.equals(uuid))
    .all()
    .await?
    .into_iter()
    .map(|(uuid, username, display_name)| SimpleUser {
        uuid,
        username,
        display_name,
    })
    .collect();

    Ok(FullWorkspace {
        uuid: workspace.uuid,
        name: workspace.name,
        description: workspace.description,
        owner,
        attacks,
        members,
        created_at: workspace.created_at,
    })
}
