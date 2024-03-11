use actix_web::delete;
use actix_web::get;
use actix_web::post;
use actix_web::put;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::web::Query;
use actix_web::HttpResponse;
use chrono::Utc;
use futures::TryStreamExt;
use log::debug;
use log::error;
use log::info;
use rorm::and;
use rorm::db::executor;
use rorm::db::sql::value::Value;
use rorm::db::Executor;
use rorm::field;
use rorm::insert;
use rorm::internal::field::Field;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::attack_results::schema::FullQueryCertificateTransparencyResult;
use crate::api::handler::attack_results::schema::FullServiceDetectionResult;
use crate::api::handler::attack_results::schema::FullUdpServiceDetectionResult;
use crate::api::handler::attack_results::schema::SimpleDnsResolutionResult;
use crate::api::handler::attack_results::schema::SimpleDnsTxtScanResult;
use crate::api::handler::attack_results::schema::SimpleHostAliveResult;
use crate::api::handler::attack_results::schema::SimpleQueryUnhashedResult;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::Page;
use crate::api::handler::common::schema::PageParams;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::SearchResultPage;
use crate::api::handler::common::schema::SearchesResultPage;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::domains::schema::SimpleDomain;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::api::handler::ports::schema::SimplePort;
use crate::api::handler::services::schema::SimpleService;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspace_invitations::schema::FullWorkspaceInvitation;
use crate::api::handler::workspace_invitations::schema::WorkspaceInvitationList;
use crate::api::handler::workspaces::schema::CreateWorkspaceRequest;
use crate::api::handler::workspaces::schema::FullWorkspace;
use crate::api::handler::workspaces::schema::InviteToWorkspaceRequest;
use crate::api::handler::workspaces::schema::InviteUuid;
use crate::api::handler::workspaces::schema::ListWorkspaces;
use crate::api::handler::workspaces::schema::SearchEntry;
use crate::api::handler::workspaces::schema::SearchResultEntry;
use crate::api::handler::workspaces::schema::SearchUuid;
use crate::api::handler::workspaces::schema::SearchWorkspaceRequest;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::api::handler::workspaces::schema::TransferWorkspaceRequest;
use crate::api::handler::workspaces::schema::UpdateWorkspaceRequest;
use crate::api::handler::workspaces::utils::get_workspace_unchecked;
use crate::api::handler::workspaces::utils::run_search;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::Attack;
use crate::models::CertificateTransparencyResult;
use crate::models::CertificateTransparencyValueName;
use crate::models::DehashedQueryResult;
use crate::models::DnsResolutionResult;
use crate::models::DnsTxtScanAttackResult;
use crate::models::Domain;
use crate::models::Host;
use crate::models::HostAliveResult;
use crate::models::ModelType;
use crate::models::Port;
use crate::models::Search;
use crate::models::SearchInsert;
use crate::models::SearchResult;
use crate::models::Service;
use crate::models::ServiceDetectionName;
use crate::models::ServiceDetectionResult;
use crate::models::UdpServiceDetectionName;
use crate::models::UdpServiceDetectionResult;
use crate::models::UserPermission;
use crate::models::Workspace;
use crate::models::WorkspaceInvitation;
use crate::models::WorkspaceMember;
use crate::modules::cache::EditorCached;

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

    let executing_user = GLOBAL
        .user_cache
        .get_full_user(user_uuid)
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

        const ATTACK_TABLE: &str = Attack::TABLE;
        const ATTACK_UUID: &str = <field!(Attack::F.uuid)>::NAME;
        const ATTACK_WORKSPACE: &str = <field!(Attack::F.workspace)>::NAME;
        const RESULT_TABLE: &str = CertificateTransparencyResult::TABLE;
        const RESULT_UUID: &str = <field!(CertificateTransparencyResult::F.uuid)>::NAME;
        const RESULT_ATTACK: &str = <field!(CertificateTransparencyResult::F.attack)>::NAME;
        const VALUE_TABLE: &str = CertificateTransparencyValueName::TABLE;
        const VALUE_RESULT: &str = <field!(CertificateTransparencyValueName::F.ct_result)>::NAME;
        tx.execute::<executor::AffectedRows>(
            format!(r#"DELETE FROM "{VALUE_TABLE}" USING "{RESULT_TABLE}", "{ATTACK_TABLE}" WHERE "{VALUE_TABLE}"."{VALUE_RESULT}" = "{RESULT_TABLE}"."{RESULT_UUID}" AND "{RESULT_TABLE}"."{RESULT_ATTACK}" = "{ATTACK_TABLE}"."{ATTACK_UUID}" AND "{ATTACK_TABLE}"."{ATTACK_WORKSPACE}" = $1;"#),
            vec![Value::Uuid(workspace.uuid)],
        )
            .await?;
        rorm::delete!(&mut tx, Workspace).single(&workspace).await?;
    } else {
        debug!(
            "User {} does not have the privileges to delete the workspace {}",
            executing_user.username, workspace.uuid
        );

        return Err(ApiError::MissingPrivileges);
    }

    tx.commit().await?;

    // Remove entry from the cache
    GLOBAL.editor_cache.ws_notes.delete(req.uuid);

    Ok(HttpResponse::Ok().finish())
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

/// Retrieve all workspaces that the executing user has access to
///
/// For administration access, look at the `/admin/workspaces` endpoint.
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns all workspaces that the executing user has access to", body = ListWorkspaces),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub async fn get_all_workspaces(
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<ListWorkspaces>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let session_user = GLOBAL
        .user_cache
        .get_simple_user(user_uuid)
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    let mut workspaces: Vec<(Workspace, SimpleUser)> = query!(&mut tx, Workspace)
        .condition(Workspace::F.owner.equals(session_user.uuid))
        .stream()
        .map_ok(|x| (x, session_user.clone()))
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

    Ok(Json(ListWorkspaces {
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

/// Transfer ownership to another account
///
/// You will lose access to the workspace.
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

    // Check if the new user exists
    if GLOBAL
        .user_cache
        .get_simple_user(new_owner_uuid)
        .await?
        .is_none()
    {
        return Err(ApiError::InvalidUuid);
    }

    update!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(workspace_uuid))
        .set(Workspace::F.owner, ForeignModelByField::Key(new_owner_uuid))
        .exec()
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
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
    request_body = InviteToWorkspaceRequest,
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/invitations")]
pub async fn create_invitation(
    req: Json<InviteToWorkspaceRequest>,
    path: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let InviteToWorkspaceRequest { user } = req.into_inner();
    let workspace_uuid = path.into_inner().uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;
    let session_user = GLOBAL
        .user_cache
        .get_simple_user(user_uuid)
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
                from: session_user,
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

/// Retrieve results for a search by its uuid
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
                    response_time: data.response_time,
                    certainty: data.certainty,
                    ip_addr: data.ip_addr.ip(),
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
                    certainty: data.certainty,
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
                    certainty: data.certainty,
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
                    certainty: data.certainty,
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
            ModelType::DnsTxtScanResult => {
                let data = query!(&mut tx, DnsTxtScanAttackResult)
                    .condition(DnsTxtScanAttackResult::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                SearchResultEntry::DnsTxtScanResultEntry(SimpleDnsTxtScanResult {
                    uuid: data.uuid,
                    created_at: data.created_at,
                    attack: *data.attack.key(),
                    domain: data.domain,
                    collection_type: data.collection_type,
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
                let data = query!(&mut tx, ServiceDetectionResult)
                    .condition(ServiceDetectionResult::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                let service_names = query!(&mut tx, (ServiceDetectionName::F.name,))
                    .condition(ServiceDetectionName::F.result.equals(item.ref_key))
                    .stream()
                    .map_ok(|x| x.0)
                    .try_collect()
                    .await?;

                SearchResultEntry::ServiceDetectionResult(FullServiceDetectionResult {
                    uuid: data.uuid,
                    created_at: data.created_at,
                    attack: *data.attack.key(),
                    host: data.host,
                    port: data.port as u16,
                    certainty: data.certainty,
                    service_names,
                })
            }
            ModelType::UdpServiceDetectionResult => {
                let data = query!(&mut tx, UdpServiceDetectionResult)
                    .condition(UdpServiceDetectionResult::F.uuid.equals(item.ref_key))
                    .one()
                    .await?;

                let service_names = query!(&mut tx, (UdpServiceDetectionName::F.name,))
                    .condition(UdpServiceDetectionName::F.result.equals(item.ref_key))
                    .stream()
                    .map_ok(|x| x.0)
                    .try_collect()
                    .await?;

                SearchResultEntry::UdpServiceDetectionResult(FullUdpServiceDetectionResult {
                    uuid: data.uuid,
                    created_at: data.created_at,
                    attack: *data.attack.key(),
                    host: data.host,
                    port: data.port as u16,
                    certainty: data.certainty,
                    service_names,
                })
            }
            ModelType::TcpPortScanResult => continue,
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
