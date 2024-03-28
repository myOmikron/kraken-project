use std::str::FromStr;

use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use log::error;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;
use url::Url;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::leeches::schema::CreateLeechRequest;
use crate::api::handler::leeches::schema::LeechConfig;
use crate::api::handler::leeches::schema::ListLeeches;
use crate::api::handler::leeches::schema::SimpleLeech;
use crate::api::handler::leeches::schema::UpdateLeechRequest;
use crate::chan::global::GLOBAL;
use crate::models::Leech;
use crate::modules::uri::check_leech_address;

/// Create a leech
///
/// The `name` parameter must be unique.
///
/// `address` must be a valid address including a scheme and port.
/// Currently only https and http are supported as scheme.
#[swaggapi::post("/leeches")]
pub async fn create_leech(req: Json<CreateLeechRequest>) -> ApiResult<Json<UuidResponse>> {
    let req = req.into_inner();

    let uuid = Leech::insert(&GLOBAL.db, req.name, req.address, req.description).await?;

    GLOBAL.leeches.created_leech(uuid).await;

    Ok(Json(UuidResponse { uuid }))
}

/// Delete a leech by its uuid
#[swaggapi::delete("/leeches/{uuid}")]
pub async fn delete_leech(path: Path<PathUuid>) -> ApiResult<HttpResponse> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    query!(&mut tx, (Leech::F.uuid,))
        .condition(Leech::F.uuid.equals(path.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    rorm::delete!(&mut tx, Leech)
        .condition(Leech::F.uuid.equals(path.uuid))
        .await?;

    tx.commit().await?;

    GLOBAL.leeches.deleted_leech(path.uuid).await;

    Ok(HttpResponse::Ok().finish())
}

/// Generate a new config for the leech
#[swaggapi::get("/leeches/{uuid}/cert")]
pub async fn gen_leech_config(req: Path<PathUuid>) -> ApiResult<Json<LeechConfig>> {
    let (secret, address) = query!(&GLOBAL.db, (Leech::F.secret, Leech::F.address))
        .condition(Leech::F.uuid.equals(req.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;
    Ok(Json(LeechConfig {
        tls: GLOBAL
            .tls
            .gen_leech_cert(Url::from_str(&address).map_err(|_| {
                error!("The leech {} doesn't have a valid address", req.uuid);
                ApiError::InternalServerError
            })?)?,
        secret,
    }))
}

/// Update a leech by its id
///
/// All parameter are optional, but at least one of them must be specified.
///
/// `address` must be a valid address including a scheme and port.
/// Currently only https and http are supported as scheme.
#[swaggapi::put("/leeches/{uuid}")]
pub async fn update_leech(
    path: Path<PathUuid>,
    req: Json<UpdateLeechRequest>,
) -> ApiResult<HttpResponse> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let req = req.into_inner();

    query!(&mut tx, (Leech::F.uuid,))
        .condition(Leech::F.uuid.equals(path.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if let Some(address) = &req.address {
        if !check_leech_address(address) {
            return Err(ApiError::InvalidAddress);
        }
    }

    update!(&mut tx, Leech)
        .begin_dyn_set()
        .set_if(Leech::F.name, req.name)
        .set_if(Leech::F.address, req.address.map(|x| x.to_string()))
        .set_if(Leech::F.description, req.description)
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .condition(Leech::F.uuid.equals(path.uuid))
        .exec()
        .await?;

    tx.commit().await?;

    GLOBAL.leeches.updated_leech(path.uuid).await;

    Ok(HttpResponse::Ok().finish())
}

/// Retrieve a leech by its id
#[swaggapi::get("/leeches/{uuid}")]
pub async fn get_leech(req: Path<PathUuid>) -> ApiResult<Json<SimpleLeech>> {
    let leech = query!(&GLOBAL.db, Leech)
        .condition(Leech::F.uuid.equals(req.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    Ok(Json(SimpleLeech {
        uuid: leech.uuid,
        name: leech.name,
        address: Url::parse(&leech.address).map_err(|e| {
            error!("Invalid URL received from database: {e}");
            ApiError::InternalServerError
        })?,
    }))
}

/// Retrieve all leeches
#[swaggapi::get("/leeches")]
pub async fn get_all_leeches() -> ApiResult<Json<ListLeeches>> {
    let leeches = query!(&GLOBAL.db, Leech).all().await?;

    let mut leech_list = vec![];

    for l in leeches {
        leech_list.push(SimpleLeech {
            uuid: l.uuid,
            name: l.name,
            address: Url::parse(&l.address).map_err(|e| {
                error!("Invalid URL received from database: {e}");
                ApiError::InternalServerError
            })?,
        })
    }

    Ok(Json(ListLeeches {
        leeches: leech_list,
    }))
}
