use std::collections::HashMap;

use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use futures::TryStreamExt;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::finding_definitions::schema::FindingDefinitionUsage;
use crate::api::handler::finding_definitions::schema::ListFindingDefinitionUsages;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::Finding;
use crate::models::FindingAffected;
use crate::models::FindingDefinition;
use crate::modules::cache::EditorCached;

/// Get all findings using the finding definition
#[swaggapi::get("/findingDefinitions/{uuid}/usages")]
pub async fn get_finding_definition_usage(
    path: Path<PathUuid>,
) -> ApiResult<Json<ListFindingDefinitionUsages>> {
    let uuid = path.into_inner().uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;

    let mut affected_counts: HashMap<Uuid, [usize; 4]> = HashMap::new();
    let mut stream = query!(&mut tx, FindingAffected)
        .condition(FindingAffected::F.finding.definition.equals(uuid))
        .stream();
    while let Some(affected) = stream.try_next().await? {
        let [d, h, p, s] = affected_counts
            .entry(*affected.finding.key())
            .or_insert([0; 4]);
        *d += affected.domain.is_some() as usize;
        *h += affected.host.is_some() as usize;
        *p += affected.port.is_some() as usize;
        *s += affected.service.is_some() as usize;
    }
    drop(stream);

    let usages = query!(
        &mut tx,
        (
            Finding::F.uuid,
            Finding::F.severity,
            Finding::F.created_at,
            Finding::F.workspace.uuid,
            Finding::F.workspace.name,
            Finding::F.workspace.description,
            Finding::F.workspace.archived,
            Finding::F.workspace.created_at,
            Finding::F.workspace.owner.uuid,
            Finding::F.workspace.owner.username,
            Finding::F.workspace.owner.display_name,
        )
    )
    .condition(Finding::F.definition.equals(uuid))
    .stream()
    .map_ok(
        |(
            f_uuid,
            f_severity,
            f_created_at,
            w_uuid,
            w_name,
            w_description,
            w_archived,
            w_created_at,
            o_uuid,
            o_username,
            o_display_name,
        )| {
            let [affected_domains, affected_hosts, affected_ports, affected_services] =
                affected_counts.get(&f_uuid).copied().unwrap_or([0; 4]);
            FindingDefinitionUsage {
                uuid: f_uuid,
                severity: f_severity,
                created_at: f_created_at,
                workspace: SimpleWorkspace {
                    uuid: w_uuid,
                    name: w_name,
                    description: w_description,
                    owner: SimpleUser {
                        uuid: o_uuid,
                        username: o_username,
                        display_name: o_display_name,
                    },
                    archived: w_archived,
                    created_at: w_created_at,
                },
                affected_domains,
                affected_hosts,
                affected_ports,
                affected_services,
            }
        },
    )
    .try_collect()
    .await?;

    tx.commit().await?;

    Ok(Json(ListFindingDefinitionUsages { usages }))
}

/// Delete a finding definition
#[swaggapi::delete("/findingDefinitions/{uuid}")]
pub async fn delete_finding_definition(path: Path<PathUuid>) -> ApiResult<HttpResponse> {
    let uuid = path.into_inner().uuid;

    let deleted = rorm::delete!(&GLOBAL.db, FindingDefinition)
        .condition(FindingDefinition::F.uuid.equals(uuid))
        .await?;

    if deleted == 0 {
        return Err(ApiError::InvalidUuid);
    }

    GLOBAL.editor_cache.fd_summary.delete(uuid);
    GLOBAL.editor_cache.fd_description.delete(uuid);
    GLOBAL.editor_cache.fd_impact.delete(uuid);
    GLOBAL.editor_cache.fd_remediation.delete(uuid);
    GLOBAL.editor_cache.fd_references.delete(uuid);

    // Notify every user about deleted finding definition
    GLOBAL
        .ws
        .message_all(WsMessage::DeletedFindingDefinition { uuid })
        .await;

    Ok(HttpResponse::Ok().finish())
}
