use std::collections::HashMap;

use actix_web::delete;
use actix_web::get;
use actix_web::post;
use actix_web::put;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use futures::TryStreamExt;
use rorm::insert;
use rorm::prelude::*;
use rorm::query;
use rorm::update;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::finding_categories::schema::SimpleFindingCategory;
use crate::api::handler::finding_definitions::schema::SimpleFindingDefinition;
use crate::api::handler::findings::schema::CreateFindingRequest;
use crate::api::handler::findings::schema::FullFinding;
use crate::api::handler::findings::schema::ListFindings;
use crate::api::handler::findings::schema::PathFinding;
use crate::api::handler::findings::schema::SimpleFinding;
use crate::api::handler::findings::schema::UpdateFindingRequest;
use crate::api::handler::findings::utils::finding_affected_into_simple;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;
use crate::models::Finding;
use crate::models::FindingAffected;
use crate::models::FindingCategory;
use crate::models::FindingDefinition;
use crate::models::FindingDefinitionCategoryRelation;
use crate::models::FindingDetails;
use crate::models::FindingFindingCategoryRelation;
use crate::models::Workspace;

/// Creates a new finding
#[utoipa::path(
    tag = "Findings",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Finding created successfully", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateFindingRequest,
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspace/{uuid}/findings")]
pub async fn create_finding(
    path: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
    Json(request): Json<CreateFindingRequest>,
) -> ApiResult<Json<UuidResponse>> {
    let workspace_uuid = path.into_inner().uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, workspace_uuid, user_uuid).await? {
        return Err(ApiError::NotFound);
    }

    FindingCategory::exist_all(&mut tx, request.categories.iter().copied())
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let uuid = Finding::insert(
        &mut tx,
        workspace_uuid,
        request.definition,
        request.severity.into_db(),
        request.remediation_duration,
        request.export_details,
        request.user_details,
        None,
        request.screenshot,
        request.log_file,
    )
    .await?;

    insert!(&mut tx, FindingFindingCategoryRelation)
        .return_nothing()
        .bulk(
            request
                .categories
                .into_iter()
                .map(|cat| FindingFindingCategoryRelation {
                    uuid: Uuid::new_v4(),
                    finding: ForeignModelByField::Key(uuid),
                    category: ForeignModelByField::Key(cat),
                }),
        )
        .await?;

    for affected in request.affected {
        FindingAffected::insert(
            &mut tx,
            uuid,
            affected.uuid,
            affected.r#type,
            workspace_uuid,
            affected.export_details,
            affected.user_details,
            None,
            affected.screenshot,
            affected.log_file,
        )
        .await?;
    }

    tx.commit().await?;
    Ok(Json(UuidResponse { uuid }))
}

/// Gets a workspace's findings
#[utoipa::path(
    tag = "Findings",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The list of findings", body = ListFindings),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspace/{uuid}/findings")]
pub async fn get_all_findings(
    path: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<ListFindings>> {
    let workspace_uuid = path.into_inner().uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, workspace_uuid, user_uuid).await? {
        return Err(ApiError::NotFound);
    }

    let mut affected_lookup = HashMap::new();
    let affected = query!(
        &mut tx,
        (FindingAffected::F.uuid, FindingAffected::F.finding)
    )
    .condition(FindingAffected::F.workspace.equals(workspace_uuid))
    .all()
    .await?;

    for (_, finding) in affected {
        affected_lookup
            .entry(*finding.key())
            .and_modify(|x| *x += 1)
            .or_insert(1);
    }

    let mut categories: HashMap<_, Vec<_>> = HashMap::new();
    let mut stream = query!(
        &mut tx,
        (
            FindingFindingCategoryRelation::F.finding.uuid,
            FindingFindingCategoryRelation::F.category.uuid,
            FindingFindingCategoryRelation::F.category.name,
            FindingFindingCategoryRelation::F.category.color
        )
    )
    .condition(
        FindingFindingCategoryRelation::F
            .finding
            .workspace
            .equals(workspace_uuid),
    )
    .stream();
    while let Some((finding, uuid, name, color)) = stream.try_next().await? {
        categories
            .entry(finding)
            .or_default()
            .push(SimpleFindingCategory {
                uuid,
                name,
                color: FromDb::from_db(color),
            });
    }
    drop(stream);

    let findings = query!(
        &mut tx,
        (
            Finding::F.uuid,
            Finding::F.definition.uuid,
            Finding::F.definition.name,
            Finding::F.definition.cve,
            Finding::F.severity,
            Finding::F.sorting_weight,
            Finding::F.created_at,
        )
    )
    .condition(Finding::F.workspace.equals(workspace_uuid))
    .order_desc(Finding::F.severity)
    .order_desc(Finding::F.sorting_weight)
    .stream()
    .map_ok(
        |(uuid, definition, name, cve, severity, sorting_weight, created_at)| SimpleFinding {
            uuid,
            definition,
            name,
            cve,
            severity: FromDb::from_db(severity),
            sorting_weight,
            affected_count: *affected_lookup.get(&uuid).unwrap_or(&0),
            created_at,
            categories: categories.remove(&uuid).unwrap_or_default(),
        },
    )
    .try_collect()
    .await?;

    tx.commit().await?;
    Ok(Json(ListFindings { findings }))
}

/// Gets a single finding
#[utoipa::path(
    tag = "Findings",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "A full finding and its affected", body = FullFinding),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathFinding),
    security(("api_key" = []))
)]
#[get("/workspace/{w_uuid}/findings/{f_uuid}")]
pub async fn get_finding(
    path: Path<PathFinding>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<Json<FullFinding>> {
    let PathFinding { w_uuid, f_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::NotFound);
    }

    let finding = query!(&mut tx, Finding)
        .condition(Finding::F.uuid.equals(f_uuid))
        .optional()
        .await?
        .ok_or(ApiError::NotFound)?;

    let details = query!(&mut tx, FindingDetails)
        .condition(FindingDetails::F.uuid.equals(*finding.details.key()))
        .one()
        .await?;

    let definition = query!(&mut tx, FindingDefinition)
        .condition(FindingDefinition::F.uuid.equals(*finding.definition.key()))
        .one()
        .await?;

    let affected = query!(&mut tx, FindingAffected)
        .condition(FindingAffected::F.finding.equals(f_uuid))
        .stream()
        .map_err(ApiError::DatabaseError)
        .and_then(|x| std::future::ready(finding_affected_into_simple(x)))
        .try_collect()
        .await?;

    let categories = query!(
        &mut tx,
        (
            FindingFindingCategoryRelation::F.category.uuid,
            FindingFindingCategoryRelation::F.category.name,
            FindingFindingCategoryRelation::F.category.color,
        )
    )
    .condition(
        FindingFindingCategoryRelation::F
            .finding
            .equals(finding.uuid),
    )
    .stream()
    .map_ok(|(uuid, name, color)| SimpleFindingCategory {
        uuid,
        name,
        color: FromDb::from_db(color),
    })
    .try_collect()
    .await?;

    let definition_categories = SimpleFindingCategory::query_for_single(
        &mut tx,
        FindingDefinitionCategoryRelation::F.category,
        FindingDefinitionCategoryRelation::F.definition,
        definition.uuid,
    )
    .await?;

    tx.commit().await?;
    Ok(Json(FullFinding {
        uuid: finding.uuid,
        definition: SimpleFindingDefinition {
            uuid: definition.uuid,
            name: definition.name,
            cve: definition.cve,
            severity: FromDb::from_db(definition.severity),
            #[rustfmt::skip]
            summary: GLOBAL.editor_cache.fd_summary.get(*finding.definition.key()).await?.ok_or(ApiError::InvalidUuid)?.0,
            remediation_duration: definition.remediation_duration,
            created_at: definition.created_at,
            categories: definition_categories,
        },
        severity: FromDb::from_db(finding.severity),
        remediation_duration: finding.remediation_duration,
        sorting_weight: finding.sorting_weight,
        affected,
        #[rustfmt::skip]
        export_details: GLOBAL.editor_cache.finding_export_details.get(finding.uuid).await?.unwrap_or_default().0,
        #[rustfmt::skip]
        user_details: GLOBAL.editor_cache.finding_user_details.get(finding.uuid).await?.unwrap_or_default().0,
        tool_details: details.tool_details,
        screenshot: details.screenshot.map(|x| *x.key()),
        log_file: details.log_file.map(|x| *x.key()),
        created_at: finding.created_at,
        categories,
    }))
}

/// Updates a finding
#[utoipa::path(
    tag = "Findings",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Finding has been updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = UpdateFindingRequest,
    params(PathFinding),
    security(("api_key" = []))
)]
#[put("/workspace/{w_uuid}/findings/{f_uuid}")]
pub async fn update_finding(
    path: Path<PathFinding>,
    SessionUser(u_uuid): SessionUser,
    Json(request): Json<UpdateFindingRequest>,
) -> ApiResult<HttpResponse> {
    let PathFinding { w_uuid, f_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::NotFound);
    }

    if matches!(
        &request,
        UpdateFindingRequest {
            definition: None,
            severity: None,
            remediation_duration: None,
            sorting_weight: None,
            screenshot: None,
            log_file: None,
            categories: None,
        }
    ) {
        return Err(ApiError::EmptyJson);
    }

    // Check finding's existence and get the details uuid
    let (d_uuid,) = query!(&mut tx, (Finding::F.details,))
        .condition(Finding::F.uuid.equals(f_uuid))
        .optional()
        .await?
        .ok_or(ApiError::NotFound)?;

    // Update the parts which are stored in Finding
    if let Ok(update) = update!(&mut tx, Finding)
        .condition(Finding::F.uuid.equals(f_uuid))
        .begin_dyn_set()
        .set_if(
            Finding::F.definition,
            request.definition.map(ForeignModelByField::Key),
        )
        .set_if(Finding::F.severity, request.severity.map(IntoDb::into_db))
        .set_if(
            Finding::F.remediation_duration,
            request.remediation_duration.clone(),
        )
        .set_if(Finding::F.sorting_weight, request.sorting_weight)
        .finish_dyn_set()
    {
        update.await?;
    }

    // Update the parts which are stored in FindingDetails
    FindingDetails::update(
        &mut tx,
        *d_uuid.key(),
        None,
        request.screenshot,
        request.log_file,
    )
    .await?;

    if let Some(categories) = request.categories.clone() {
        FindingCategory::exist_all(&mut tx, categories.iter().copied())
            .await?
            .ok_or(ApiError::InvalidUuid)?;

        rorm::delete!(&mut tx, FindingFindingCategoryRelation)
            .condition(FindingFindingCategoryRelation::F.finding.equals(f_uuid))
            .await?;

        insert!(&mut tx, FindingFindingCategoryRelation)
            .return_nothing()
            .bulk(
                categories
                    .into_iter()
                    .map(|cat| FindingFindingCategoryRelation {
                        uuid: Uuid::new_v4(),
                        finding: ForeignModelByField::Key(f_uuid),
                        category: ForeignModelByField::Key(cat),
                    }),
            )
            .await?;
    }

    tx.commit().await?;
    GLOBAL
        .ws
        .message_workspace(
            w_uuid,
            WsMessage::UpdatedFinding {
                workspace: w_uuid,
                finding: f_uuid,
                update: request,
            },
        )
        .await;
    Ok(HttpResponse::Ok().finish())
}

/// Deletes a finding
#[utoipa::path(
    tag = "Findings",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Finding has been deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathFinding),
    security(("api_key" = []))
)]
#[delete("/workspace/{w_uuid}/findings/{f_uuid}")]
pub async fn delete_finding(
    path: Path<PathFinding>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let PathFinding { w_uuid, f_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::NotFound);
    }

    let deleted = Finding::delete(&mut tx, f_uuid).await?;
    GLOBAL.editor_cache.finding_export_details.delete(f_uuid);
    GLOBAL.editor_cache.finding_user_details.delete(f_uuid);

    tx.commit().await?;
    // Notify workspace members about deleted finding
    GLOBAL
        .ws
        .message_workspace(
            w_uuid,
            WsMessage::DeletedFinding {
                workspace: w_uuid,
                finding: f_uuid,
            },
        )
        .await;
    if deleted {
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(ApiError::NotFound)
    }
}
