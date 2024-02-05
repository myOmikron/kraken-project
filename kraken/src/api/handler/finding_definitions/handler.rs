use actix_web::web::{Json, Path};
use actix_web::{get, post};

use crate::api::handler::common::error::{ApiError, ApiResult};
use crate::api::handler::common::schema::{PathUuid, UuidResponse};
use crate::api::handler::finding_definitions::schema::{
    CreateFindingDefinitionRequest, FullFindingDefinition, ListFindingDefinitions,
    SimpleFindingDefinition,
};
use crate::chan::global::GLOBAL;

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

    Ok(Json(UuidResponse {
        uuid: GLOBAL
            .finding_definition_cache
            .insert(
                name,
                summary,
                severity,
                cve,
                description,
                impact,
                remediation,
                references,
            )
            .await?,
    }))
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

    let finding_definition = GLOBAL
        .finding_definition_cache
        .get(uuid)
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    Ok(Json(FullFindingDefinition {
        uuid: finding_definition.uuid,
        name: finding_definition.name,
        summary: finding_definition.summary,
        severity: finding_definition.severity,
        cve: finding_definition.cve,
        description: finding_definition.description,
        impact: finding_definition.impact,
        remediation: finding_definition.remediation,
        references: finding_definition.references,
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
    let finding_definitions = GLOBAL
        .finding_definition_cache
        .get_all()
        .await?
        .into_iter()
        .map(|finding| SimpleFindingDefinition {
            uuid: finding.uuid,
            name: finding.name,
            summary: finding.summary,
            severity: finding.severity,
            created_at: finding.created_at,
        })
        .collect();

    Ok(Json(ListFindingDefinitions {
        finding_definitions,
    }))
}
