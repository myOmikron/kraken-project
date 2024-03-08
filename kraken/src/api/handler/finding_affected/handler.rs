use actix_web::delete;
use actix_web::get;
use actix_web::post;
use actix_web::put;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use futures::TryStreamExt;
use rorm::prelude::*;
use rorm::query;
use rorm::update;

use crate::api::extractors::SessionUser;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::domains::schema::SimpleDomain;
use crate::api::handler::finding_affected::schema::CreateFindingAffectedRequest;
use crate::api::handler::finding_affected::schema::FindingAffectedObject;
use crate::api::handler::finding_affected::schema::FullFindingAffected;
use crate::api::handler::finding_affected::schema::PathFindingAffected;
use crate::api::handler::finding_affected::schema::UpdateFindingAffectedRequest;
use crate::api::handler::finding_affected::utils::query_finding_affected;
use crate::api::handler::finding_definitions::schema::SimpleFindingDefinition;
use crate::api::handler::findings::schema::FullFinding;
use crate::api::handler::findings::schema::PathFinding;
use crate::api::handler::findings::utils::finding_affected_into_simple;
use crate::api::handler::hosts::schema::SimpleHost;
use crate::api::handler::ports::schema::SimplePort;
use crate::api::handler::services::schema::SimpleService;
use crate::chan::global::GLOBAL;
use crate::models::Domain;
use crate::models::Finding;
use crate::models::FindingAffected;
use crate::models::FindingDetails;
use crate::models::Host;
use crate::models::Port;
use crate::models::Service;
use crate::models::Workspace;

/// Add a new affected object to a finding
#[utoipa::path(
    tag = "Findings",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Affected object was added successfully"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateFindingAffectedRequest,
    params(PathFinding),
    security(("api_key" = []))
)]
#[post("/workspace/{w_uuid}/findings/{f_uuid}/affected")]
pub async fn create_finding_affected(
    path: Path<PathFinding>,
    Json(request): Json<CreateFindingAffectedRequest>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let PathFinding { w_uuid, f_uuid } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::NotFound);
    }

    query!(&mut tx, (Finding::F.uuid,))
        .condition(Finding::F.uuid.equals(f_uuid))
        .optional()
        .await?
        .ok_or(ApiError::NotFound)?;

    FindingAffected::insert(
        &mut tx,
        f_uuid,
        request.uuid,
        request.r#type,
        w_uuid,
        request.details,
        None,
        request.screenshot,
        request.log_file,
    )
    .await?;

    tx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}

/// Get an object affected by a finding
#[utoipa::path(
    tag = "Findings",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "A full finding and the affected object", body = FullFindingAffected),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathFindingAffected),
    security(("api_key" = []))
)]
#[get("/workspace/{w_uuid}/findings/{f_uuid}/affected/{a_uuid}")]
pub async fn get_finding_affected(
    path: Path<PathFindingAffected>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<Json<FullFindingAffected>> {
    #[rustfmt::skip]
    let PathFindingAffected { w_uuid, f_uuid, a_uuid, } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::NotFound);
    }

    let (
        finding,
        finding_details,
        finding_definition_uuid,
        finding_definition_name,
        finding_definition_severity,
        finding_definition_summary,
        finding_definition_created_at,
        domain,
        host,
        port,
        service,
        details,
        created_at,
    ) = query_finding_affected(
        &mut tx,
        (
            FindingAffected::F.finding.select_as::<Finding>(),
            FindingAffected::F
                .finding
                .details
                .select_as::<FindingDetails>(),
            FindingAffected::F.finding.definition.uuid,
            FindingAffected::F.finding.definition.name,
            FindingAffected::F.finding.definition.severity,
            FindingAffected::F.finding.definition.summary,
            FindingAffected::F.finding.definition.created_at,
            FindingAffected::F.domain,
            FindingAffected::F.host,
            FindingAffected::F.port,
            FindingAffected::F.service,
            FindingAffected::F.details,
            FindingAffected::F.created_at,
        ),
        f_uuid,
        a_uuid,
    )
    .await?;

    let mut details = if let Some(details) = details {
        Some(
            query!(&mut tx, FindingDetails)
                .condition(FindingDetails::F.uuid.equals(*details.key()))
                .one()
                .await?,
        )
    } else {
        None
    };

    let finding_affected = query!(&mut tx, FindingAffected)
        .condition(FindingAffected::F.finding.equals(f_uuid))
        .stream()
        .map_err(ApiError::DatabaseError)
        .and_then(|x| std::future::ready(finding_affected_into_simple(x)))
        .try_collect()
        .await?;

    let affected = match (domain, host, port, service) {
        (Some(fm), None, None, None) => {
            let domain = query!(&mut tx, Domain)
                .condition(Domain::F.uuid.equals(*fm.key()))
                .one()
                .await?;
            FindingAffectedObject::Domain(SimpleDomain {
                uuid: domain.uuid,
                domain: domain.domain,
                comment: domain.comment,
                workspace: *domain.workspace.key(),
                created_at: domain.created_at,
                certainty: domain.certainty,
            })
        }
        (None, Some(fm), None, None) => {
            let host = query!(&mut tx, Host)
                .condition(Host::F.uuid.equals(*fm.key()))
                .one()
                .await?;
            FindingAffectedObject::Host(SimpleHost {
                uuid: host.uuid,
                ip_addr: host.ip_addr.ip(),
                os_type: host.os_type,
                response_time: host.response_time,
                comment: host.comment,
                workspace: *host.workspace.key(),
                created_at: host.created_at,
                certainty: host.certainty,
            })
        }
        (None, None, Some(fm), None) => {
            let port = query!(&mut tx, Port)
                .condition(Port::F.uuid.equals(*fm.key()))
                .one()
                .await?;
            FindingAffectedObject::Port(SimplePort {
                uuid: port.uuid,
                port: port.port as u16,
                protocol: port.protocol,
                certainty: port.certainty,
                host: *port.host.key(),
                comment: port.comment,
                workspace: *port.workspace.key(),
                created_at: port.created_at,
            })
        }
        (None, None, None, Some(fm)) => {
            let service = query!(&mut tx, Service)
                .condition(Service::F.uuid.equals(*fm.key()))
                .one()
                .await?;
            FindingAffectedObject::Service(SimpleService {
                uuid: service.uuid,
                name: service.name,
                version: service.version,
                certainty: service.certainty,
                host: *service.host.key(),
                port: service.port.map(|fm| *fm.key()),
                comment: service.comment,
                workspace: *service.workspace.key(),
                created_at: service.created_at,
            })
        }
        _ => return Err(ApiError::InternalServerError),
    };

    tx.commit().await?;
    Ok(Json(FullFindingAffected {
        finding: FullFinding {
            uuid: finding.uuid,
            definition: SimpleFindingDefinition {
                uuid: finding_definition_uuid,
                name: finding_definition_name,
                severity: finding_definition_severity,
                summary: finding_definition_summary,
                created_at: finding_definition_created_at,
            },
            severity: finding.severity,
            affected: finding_affected,
            user_details: finding_details.user_details,
            tool_details: finding_details.tool_details,
            screenshot: finding_details.screenshot.map(|fm| *fm.key()),
            log_file: finding_details.log_file.map(|fm| *fm.key()),
            created_at: finding.created_at,
        },
        affected,
        user_details: details.as_mut().and_then(|d| d.user_details.take()),
        tool_details: details.as_mut().and_then(|d| d.tool_details.take()),
        screenshot: details
            .as_mut()
            .and_then(|d| d.screenshot.take().map(|fm| *fm.key())),
        log_file: details
            .as_mut()
            .and_then(|d| d.log_file.take().map(|fm| *fm.key())),
        created_at,
    }))
}

/// Update the details of an affected object
#[utoipa::path(
    tag = "Findings",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Affected object has been updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = UpdateFindingAffectedRequest,
    params(PathFindingAffected),
    security(("api_key" = []))
)]
#[put("/workspace/{w_uuid}/findings/{f_uuid}/affected/{a_uuid}")]
pub async fn update_finding_affected(
    path: Path<PathFindingAffected>,
    Json(request): Json<UpdateFindingAffectedRequest>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    #[rustfmt::skip]
    let PathFindingAffected { w_uuid, f_uuid, a_uuid, } = path.into_inner();

    if matches!(
        &request,
        UpdateFindingAffectedRequest {
            details: None,
            screenshot: None,
            log_file: None
        }
    ) {
        return Err(ApiError::EmptyJson);
    }

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::NotFound);
    }

    let (details,) =
        query_finding_affected(&mut tx, (FindingAffected::F.details,), f_uuid, a_uuid).await?;

    if let Some(details) = details {
        FindingDetails::update(
            &mut tx,
            *details.key(),
            request.details,
            None,
            request.screenshot,
            request.log_file,
        )
        .await?;
    } else {
        let details = request.details.flatten();
        let screenshot = request.screenshot.flatten();
        let log_file = request.log_file.flatten();
        if details.is_some() || screenshot.is_some() || log_file.is_some() {
            let uuid = FindingDetails::insert(&mut tx, details, None, screenshot, log_file).await?;
            update!(&mut tx, FindingAffected)
                .set(
                    FindingAffected::F.details,
                    Some(ForeignModelByField::Key(uuid)),
                )
                .await?;
        }
    };

    tx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}

/// Remove an affected object from a finding
#[utoipa::path(
    tag = "Findings",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Affected object has been removed"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathFindingAffected),
    security(("api_key" = []))
)]
#[delete("/workspace/{w_uuid}/findings/{f_uuid}/affected/{a_uuid}")]
pub async fn delete_finding_affected(
    path: Path<PathFindingAffected>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    #[rustfmt::skip]
    let PathFindingAffected { w_uuid, f_uuid, a_uuid, } = path.into_inner();

    let mut tx = GLOBAL.db.start_transaction().await?;
    if !Workspace::is_user_member_or_owner(&mut tx, w_uuid, u_uuid).await? {
        return Err(ApiError::NotFound);
    }

    let (uuid,) =
        query_finding_affected(&mut tx, (FindingAffected::F.uuid,), f_uuid, a_uuid).await?;
    FindingAffected::delete(&mut tx, uuid).await?;

    tx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}
