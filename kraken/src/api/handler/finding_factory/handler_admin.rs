use std::collections::HashMap;

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

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::finding_definitions::schema::SimpleFindingDefinition;
use crate::api::handler::finding_factory::schema::FullFindingFactoryEntry;
use crate::api::handler::finding_factory::schema::GetFindingFactoryEntriesResponse;
use crate::api::handler::finding_factory::schema::UpdateFindingFactoryEntryRequest;
use crate::chan::global::GLOBAL;
use crate::models::convert::FromDb;
use crate::models::FindingDefinition;
use crate::models::FindingFactoryEntry;
use crate::modules::finding_factory::schema::FindingFactoryIdentifier;

/// Retrieves the current mapping between finding factory identifiers and finding definitions
///
/// An identifier is an enum variant which identifies one kind of issue,
/// the finding factory might create a finding for.
///
/// If the finding factory detects an issue it will look up its identifier's finding definition
/// and create a finding using this definition (if it found any).
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
    let mut tx = GLOBAL.db.start_transaction().await?;

    // TODO query categories

    let entries: HashMap<_, _> = query!(
        &mut tx,
        (
            FindingFactoryEntry::F.identifier,
            FindingFactoryEntry::F.finding as FindingDefinition,
        )
    )
    .stream()
    .try_filter_map(|(identifier, finding)| async move {
        let Ok(identifier) = identifier.parse::<FindingFactoryIdentifier>() else {
            warn!("Found invalid `FindingFactoryIdentifier` in db: {identifier}");
            return Ok(None);
        };

        Ok(Some((
            identifier,
            FullFindingFactoryEntry {
                identifier,
                finding: Some(SimpleFindingDefinition {
                    uuid: finding.uuid,
                    name: finding.name,
                    cve: finding.cve,
                    severity: FromDb::from_db(finding.severity),
                    summary: finding.summary,
                    created_at: finding.created_at,
                    categories: Vec::new(),
                }),
            },
        )))
    })
    .try_collect()
    .await?;

    tx.commit().await?;
    Ok(Json(GetFindingFactoryEntriesResponse { entries }))
}

/// Updates a single finding factory identifier
#[utoipa::path(
    tag = "Finding Factory",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "The entry has been updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[put("/finding-factory/entry")]
pub async fn update_finding_factory_entry(
    request: Json<UpdateFindingFactoryEntryRequest>,
) -> ApiResult<HttpResponse> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let UpdateFindingFactoryEntryRequest {
        identifier,
        finding_definition,
    } = request.into_inner();
    let identifier = identifier.to_string();
    let finding = finding_definition.map(|opt| opt.map(ForeignModelByField::Key));

    let updated = update!(&mut tx, FindingFactoryEntry)
        .begin_dyn_set()
        .set_if(FindingFactoryEntry::F.finding, finding.clone())
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .condition(FindingFactoryEntry::F.identifier.equals(&identifier))
        .await?;

    if updated < 1 {
        insert!(&mut tx, FindingFactoryEntry)
            .single(&FindingFactoryEntry {
                uuid: Uuid::new_v4(),
                identifier,
                finding: finding.unwrap_or(None),
            })
            .await?;
    }

    tx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}
