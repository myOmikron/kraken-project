//! The management endpoints of the leech are located here

use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use log::error;
use rorm::{insert, query, update, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{ApiError, ApiResult, PathUuid, UuidResponse};
use crate::chan::{RpcManagerChannel, RpcManagerEvent};
use crate::models::{Leech, LeechInsert};
use crate::modules::uri::check_leech_address;

/// The request to create a new leech
#[derive(Deserialize, ToSchema)]
pub struct CreateLeechRequest {
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
pub async fn create_leech(
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
pub async fn delete_leech(
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

/// T
#[derive(Serialize, ToSchema)]
pub struct SimpleLeech {
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
        (status = 200, description = "Matched leeches", body = SimpleLeech),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/leeches/{uuid}")]
pub async fn get_leech(req: Path<PathUuid>, db: Data<Database>) -> ApiResult<Json<SimpleLeech>> {
    let leech = query!(db.as_ref(), Leech)
        .condition(Leech::F.uuid.equals(req.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    Ok(Json(SimpleLeech {
        uuid: leech.uuid,
        name: leech.name,
        address: leech.address,
    }))
}

/// The response that hold all leeches
#[derive(Serialize, ToSchema)]
pub struct GetAllLeechesResponse {
    leeches: Vec<SimpleLeech>,
}

/// Retrieve all leeches
#[utoipa::path(
    tag = "Leech management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Matched leeches", body = GetAllLeechesResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    security(("api_key" = []))
)]
#[get("/leeches")]
pub async fn get_all_leeches(db: Data<Database>) -> ApiResult<Json<GetAllLeechesResponse>> {
    let leeches = query!(db.as_ref(), Leech).all().await?;

    Ok(Json(GetAllLeechesResponse {
        leeches: leeches
            .into_iter()
            .map(|l| SimpleLeech {
                uuid: l.uuid,
                name: l.name,
                address: l.address,
            })
            .collect(),
    }))
}

/// The request to update a leech
#[derive(Deserialize, ToSchema)]
pub struct UpdateLeechRequest {
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
pub async fn update_leech(
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
