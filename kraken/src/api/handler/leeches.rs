use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use log::error;
use rorm::{insert, query, update, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::users::PathUuid;
use crate::api::handler::{ApiError, ApiResult, UuidResponse};
use crate::chan::{RpcManagerChannel, RpcManagerEvent};
use crate::models::{Leech, LeechInsert};
use crate::modules::uri::check_leech_address;

#[derive(Deserialize, ToSchema)]
pub(crate) struct CreateLeechRequest {
    #[schema(example = "leech-01")]
    pub(crate) name: String,
    #[schema(example = "https://10.13.37:8081")]
    pub(crate) address: String,
    #[schema(example = "The first leech in a private net")]
    pub(crate) description: Option<String>,
}

/// Create a leech
///
/// The `name` parameter must be unique.
///
/// `address` must be a valid address including a scheme and port.
/// Currently only https and http are supported as scheme.
#[utoipa::path(
    tag = "Leech management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Leech got created successfully", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = CreateLeechRequest,
    security(("api_key" = []))
)]
#[post("/leeches")]
pub(crate) async fn create_leech(
    req: Json<CreateLeechRequest>,
    db: Data<Database>,
    rpc_manager_channel: Data<RpcManagerChannel>,
) -> ApiResult<Json<UuidResponse>> {
    let mut tx = db.start_transaction().await?;

    if !check_leech_address(&req.address) {
        return Err(ApiError::InvalidAddress);
    }

    if query!(&mut tx, Leech)
        .condition(Leech::F.address.equals(&req.address))
        .optional()
        .await?
        .is_some()
    {
        return Err(ApiError::AddressAlreadyExists);
    }

    if query!(&mut tx, Leech)
        .condition(Leech::F.name.equals(&req.name))
        .optional()
        .await?
        .is_some()
    {
        return Err(ApiError::NameAlreadyExists);
    }

    let uuid = insert!(&mut tx, LeechInsert)
        .return_primary_key()
        .single(&LeechInsert {
            uuid: Uuid::new_v4(),
            name: req.name.clone(),
            address: req.address.clone(),
            description: req.description.clone(),
        })
        .await?;

    tx.commit().await?;

    // Notify rpc manager about new leech
    if let Err(err) = rpc_manager_channel
        .send(RpcManagerEvent::Created(uuid))
        .await
    {
        error!("Error sending to rpc manager: {err}");
    }

    Ok(Json(UuidResponse { uuid }))
}

/// Delete a leech by its uuid
#[utoipa::path(
    tag = "Leech management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Leech got deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/leeches/{uuid}")]
pub(crate) async fn delete_leech(
    path: Path<PathUuid>,
    db: Data<Database>,
    rpc_manager_channel: Data<RpcManagerChannel>,
) -> ApiResult<HttpResponse> {
    let mut tx = db.start_transaction().await?;

    query!(&mut tx, (Leech::F.uuid,))
        .condition(Leech::F.uuid.equals(path.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    rorm::delete!(&mut tx, Leech)
        .condition(Leech::F.uuid.equals(path.uuid))
        .await?;

    tx.commit().await?;

    // Notify rpc manager about deleted leech
    if let Err(err) = rpc_manager_channel
        .send(RpcManagerEvent::Deleted(path.uuid))
        .await
    {
        error!("Error sending to rpc manager: {err}");
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetLeech {
    uuid: Uuid,
    #[schema(example = "leech-01")]
    name: String,
    #[schema(example = "https://10.13.37.1:8081")]
    address: String,
}

/// Retrieve a leech by its id
#[utoipa::path(
    tag = "Leech management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Matched leeches", body = GetLeech),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/leeches/{uuid}")]
pub(crate) async fn get_leech(
    req: Path<PathUuid>,
    db: Data<Database>,
) -> ApiResult<Json<GetLeech>> {
    let leech = query!(db.as_ref(), Leech)
        .condition(Leech::F.uuid.equals(req.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    Ok(Json(GetLeech {
        uuid: leech.uuid,
        name: leech.name,
        address: leech.address,
    }))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetLeechResponse {
    leeches: Vec<GetLeech>,
}

/// Retrieve all leeches
#[utoipa::path(
    tag = "Leech management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Matched leeches", body = GetLeechResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    security(("api_key" = []))
)]
#[get("/leeches")]
pub(crate) async fn get_all_leeches(db: Data<Database>) -> ApiResult<Json<GetLeechResponse>> {
    let leeches = query!(db.as_ref(), Leech).all().await?;

    Ok(Json(GetLeechResponse {
        leeches: leeches
            .into_iter()
            .map(|l| GetLeech {
                uuid: l.uuid,
                name: l.name,
                address: l.address,
            })
            .collect(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct UpdateLeechRequest {
    #[schema(example = "leech-01")]
    name: Option<String>,
    #[schema(example = "https://10.13.37.1:8081")]
    address: Option<String>,
    #[schema(example = "First leech in a private network")]
    #[serde(default)]
    #[serde(deserialize_with = "crate::api::handler::de_optional")]
    description: Option<Option<String>>,
}

/// Update a leech by its id
///
/// All parameter are optional, but at least one of them must be specified.
///
/// `address` must be a valid address including a scheme and port.
/// Currently only https and http are supported as scheme.
#[utoipa::path(
    tag = "Leech management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Leech got updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    request_body = UpdateLeechRequest,
    security(("api_key" = []))
)]
#[put("/leeches/{uuid}")]
pub(crate) async fn update_leech(
    path: Path<PathUuid>,
    req: Json<UpdateLeechRequest>,
    db: Data<Database>,
    rpc_manager_channel: Data<RpcManagerChannel>,
) -> ApiResult<HttpResponse> {
    let mut tx = db.start_transaction().await?;

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
        .set_if(Leech::F.address, req.address)
        .set_if(Leech::F.description, req.description)
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .condition(Leech::F.uuid.equals(path.uuid))
        .exec()
        .await?;

    tx.commit().await?;

    // Notify rpc manager about updated leech
    if let Err(err) = rpc_manager_channel
        .send(RpcManagerEvent::Updated(path.uuid))
        .await
    {
        error!("Error sending to rpc manager: {err}");
    }

    Ok(HttpResponse::Ok().finish())
}
