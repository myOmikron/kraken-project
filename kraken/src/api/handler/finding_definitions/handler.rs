use actix_web::get;
use actix_web::post;
use actix_web::web::Json;
use actix_web::web::Path;
use futures::TryStreamExt;
use rorm::insert;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::finding_definitions::schema::CreateFindingDefinitionRequest;
use crate::api::handler::finding_definitions::schema::FullFindingDefinition;
use crate::api::handler::finding_definitions::schema::ListFindingDefinitions;
use crate::api::handler::finding_definitions::schema::SimpleFindingDefinition;
use crate::chan::global::GLOBAL;
use crate::models::FindingDefinition;
use crate::models::InsertFindingDefinition;
use crate::modules::cache::EditorCached;

/// Add a definition for a finding
///
/// These definition serve as reference and knowledge base in kraken.
/// They can be used to create a finding that references a definition and links it to one or
/// multiple aggregations.
#[utoipa::path(
    tag = "Knowledge Base",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Finding definition was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateFindingDefinitionRequest,
    security(("api_key" = []))
)]
#[post("/findingDefinitions")]
pub async fn create_finding_definition(
    req: Json<CreateFindingDefinitionRequest>,
) -> ApiResult<Json<UuidResponse>> {
    let CreateFindingDefinitionRequest {
        name,
        summary,
        severity,
        cve,
        description,
        impact,
        remediation,
        references,
    } = req.into_inner();

    let uuid = Uuid::new_v4();

    insert!(&GLOBAL.db, FindingDefinition)
        .single(&InsertFindingDefinition {
            uuid,
            name,
            summary,
            severity,
            cve,
            description,
            impact,
            remediation,
            references,
        })
        .await?;

    GLOBAL.editor_cache.fd_summary.invalidate_not_found();
    GLOBAL.editor_cache.fd_description.invalidate_not_found();
    GLOBAL.editor_cache.fd_impact.invalidate_not_found();
    GLOBAL.editor_cache.fd_remediation.invalidate_not_found();
    GLOBAL.editor_cache.fd_references.invalidate_not_found();

    Ok(Json(UuidResponse { uuid }))
}

/// Retrieve a specific finding definition
#[utoipa::path(
    tag = "Knowledge Base",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieved a specific finding definition", body = FullFindingDefinition),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/findingDefinitions/{uuid}")]
pub async fn get_finding_definition(
    path: Path<PathUuid>,
) -> ApiResult<Json<FullFindingDefinition>> {
    let uuid = path.into_inner().uuid;

    let finding_definition = query!(&GLOBAL.db, FindingDefinition)
        .condition(FindingDefinition::F.uuid.equals(uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    Ok(Json(FullFindingDefinition {
        uuid: finding_definition.uuid,
        name: finding_definition.name,
        #[rustfmt::skip]
        summary: GLOBAL.editor_cache.fd_summary.get(uuid).await?.ok_or(ApiError::InvalidUuid)?,
        severity: finding_definition.severity,
        cve: finding_definition.cve,
        #[rustfmt::skip]
        description: GLOBAL.editor_cache.fd_description.get(uuid).await?.ok_or(ApiError::InvalidUuid)?,
        #[rustfmt::skip]
        impact: GLOBAL.editor_cache.fd_impact.get(uuid).await?.ok_or(ApiError::InvalidUuid)?,
        #[rustfmt::skip]
        remediation: GLOBAL.editor_cache.fd_remediation.get(uuid).await?.ok_or(ApiError::InvalidUuid)?,
        #[rustfmt::skip]
        references: GLOBAL.editor_cache.fd_references.get(uuid).await?.ok_or(ApiError::InvalidUuid)?,
        created_at: finding_definition.created_at,
    }))
}

/// Retrieve all finding definitions
#[utoipa::path(
    tag = "Knowledge Base",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieved a list of all finding definitions", body = ListFindingDefinitions),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/findingDefinitions")]
pub async fn get_all_finding_definitions() -> ApiResult<Json<ListFindingDefinitions>> {
    let mut finding_definitions: Vec<SimpleFindingDefinition> =
        query!(&GLOBAL.db, FindingDefinition)
            .stream()
            .map_ok(|fd| SimpleFindingDefinition {
                uuid: fd.uuid,
                name: fd.name,
                summary: fd.summary,
                severity: fd.severity,
                created_at: fd.created_at,
            })
            .try_collect()
            .await?;

    for fd in &mut finding_definitions {
        fd.summary = GLOBAL
            .editor_cache
            .fd_summary
            .get(fd.uuid)
            .await?
            .ok_or(ApiError::InternalServerError)?;
    }

    Ok(Json(ListFindingDefinitions {
        finding_definitions,
    }))
}
