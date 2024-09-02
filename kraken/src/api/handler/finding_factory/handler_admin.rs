use actix_web::get;
use actix_web::put;
use actix_web::web::Json;
use actix_web::HttpResponse;
use futures::TryStreamExt;
use log::warn;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::handler::common::error::ApiResult;
use crate::api::handler::finding_definitions::schema::SimpleFindingDefinition;
use crate::api::handler::finding_factory::schema::GetFindingFactoryEntriesResponse;
use crate::api::handler::finding_factory::schema::SetFindingFactoryEntryRequest;
use crate::chan::global::GLOBAL;
use crate::models::convert::FromDb;
use crate::models::FindingDefinition;
use crate::models::FindingFactoryEntry;
use crate::modules::finding_factory::schema::FindingFactoryIdentifier;

#[utoipa::path(
    tag = "Finding Factory",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Retrieved the list of all finding factory entries", body = GetFindingFactoryEntriesResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/finding-factory/entries")]
pub async fn get_finding_factory_entries() -> ApiResult<Json<GetFindingFactoryEntriesResponse>> {
    // TODO query categories
    let entries = query!(
        &GLOBAL.db,
        (
            FindingFactoryEntry::F.identifier,
            FindingFactoryEntry::F.finding as FindingDefinition
        )
    )
    .stream()
    .try_filter_map(|(identifier, finding)| async move {
        if let Ok(identifier) = identifier.parse::<FindingFactoryIdentifier>() {
            Ok(Some((
                identifier,
                SimpleFindingDefinition {
                    uuid: finding.uuid,
                    name: finding.name,
                    cve: finding.cve,
                    severity: FromDb::from_db(finding.severity),
                    summary: finding.summary,
                    created_at: finding.created_at,
                    categories: Vec::new(),
                },
            )))
        } else {
            warn!("Found invalid `FindingFactoryIdentifier` in db: {identifier}");
            Ok(None)
        }
    })
    .try_collect()
    .await?;
    Ok(Json(GetFindingFactoryEntriesResponse { entries }))
}

#[utoipa::path(
    tag = "Finding Factory",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "The entry has been set"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[put("/finding-factory/entry")]
pub async fn set_finding_factory_entry(
    request: Json<SetFindingFactoryEntryRequest>,
) -> ApiResult<HttpResponse> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let SetFindingFactoryEntryRequest {
        identifier,
        finding_definition,
    } = request.into_inner();
    let identifier = identifier.to_string();

    if let Some(finding) = finding_definition {
        let updated = update!(&mut tx, FindingFactoryEntry)
            .set(
                FindingFactoryEntry::F.finding,
                ForeignModelByField::Key(finding),
            )
            .condition(FindingFactoryEntry::F.identifier.equals(&identifier))
            .await?;

        if updated < 1 {
            insert!(&mut tx, FindingFactoryEntry)
                .single(&FindingFactoryEntry {
                    uuid: Uuid::new_v4(),
                    identifier,
                    finding: ForeignModelByField::Key(finding),
                })
                .await?;
        }
    } else {
        rorm::delete!(&mut tx, FindingFactoryEntry)
            .condition(FindingFactoryEntry::F.identifier.equals(&identifier))
            .await?;
    }

    tx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}
