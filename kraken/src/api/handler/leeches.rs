use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use rorm::{insert, query, update, Database, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::handler::{ApiError, ApiResult, PathId};
use crate::models::{Leech, LeechInsert};
use crate::modules::uri::check_leech_address;

#[derive(Deserialize, ToSchema)]
pub(crate) struct CreateLeechRequest {
    pub(crate) name: String,
    pub(crate) address: String,
    pub(crate) description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct CreateLeechResponse {
    pub(crate) id: i64,
}

#[utoipa::path(
    tag = "Leech management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Leech got created successfully", body = CreateLeechResponse),
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
) -> ApiResult<Json<CreateLeechResponse>> {
    let mut tx = db.start_transaction().await?;

    if !check_leech_address(&req.address) {
        return Err(ApiError::InvalidAddress);
    }

    if query!(&db, Leech)
        .transaction(&mut tx)
        .condition(Leech::F.address.equals(&req.address))
        .optional()
        .await?
        .is_some()
    {
        return Err(ApiError::AddressAlreadyExists);
    }

    if query!(&db, Leech)
        .transaction(&mut tx)
        .condition(Leech::F.name.equals(&req.name))
        .optional()
        .await?
        .is_some()
    {
        return Err(ApiError::NameAlreadyExists);
    }

    let id = insert!(&db, LeechInsert)
        .transaction(&mut tx)
        .single(&LeechInsert {
            name: req.name.clone(),
            address: req.address.clone(),
            description: req.description.clone(),
        })
        .await?;

    tx.commit().await?;

    Ok(Json(CreateLeechResponse { id }))
}

#[utoipa::path(
    tag = "Leech management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Leech got deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathId),
    security(("api_key" = []))
)]
#[delete("/leeches/{id}")]
pub(crate) async fn delete_leech(
    path: Path<PathId>,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let mut tx = db.start_transaction().await?;

    query!(&db, (Leech::F.id,))
        .transaction(&mut tx)
        .condition(Leech::F.id.equals(path.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    rorm::delete!(&db, Leech)
        .transaction(&mut tx)
        .condition(Leech::F.id.equals(path.id as i64))
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetLeech {
    id: i64,
    name: String,
    address: String,
}

#[utoipa::path(
    tag = "Leech management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Matched leeches", body = GetLeech),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathId),
    security(("api_key" = []))
)]
#[get("/leeches/{id}")]
pub(crate) async fn get_leech(req: Path<PathId>, db: Data<Database>) -> ApiResult<Json<GetLeech>> {
    let leech = query!(&db, Leech)
        .condition(Leech::F.id.equals(req.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    Ok(Json(GetLeech {
        id: leech.id,
        name: leech.name,
        address: leech.address,
    }))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetLeechResponse {
    leeches: Vec<GetLeech>,
}

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
    let leeches = query!(&db, Leech).all().await?;

    Ok(Json(GetLeechResponse {
        leeches: leeches
            .into_iter()
            .map(|l| GetLeech {
                id: l.id,
                name: l.name,
                address: l.address,
            })
            .collect(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct UpdateLeechRequest {
    name: Option<String>,
    address: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "crate::api::handler::de_optional")]
    description: Option<Option<String>>,
}

#[utoipa::path(
    tag = "Leech management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Leech got updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathId),
    request_body = UpdateLeechRequest,
    security(("api_key" = []))
)]
#[put("/leeches/{id}")]
pub(crate) async fn update_leech(
    path: Path<PathId>,
    req: Json<UpdateLeechRequest>,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let mut tx = db.start_transaction().await?;

    query!(&db, (Leech::F.id,))
        .transaction(&mut tx)
        .condition(Leech::F.id.equals(path.id as i64))
        .optional()
        .await?
        .ok_or(ApiError::InvalidId)?;

    let mut ub = update!(&db, Leech).begin_dyn_set();

    if let Some(name) = &req.name {
        ub = ub.set(Leech::F.name, name);
    }

    if let Some(address) = &req.address {
        if !check_leech_address(address) {
            return Err(ApiError::InvalidAddress);
        }
        ub = ub.set(Leech::F.address, address);
    }

    if let Some(description) = &req.description {
        ub = ub.set(Leech::F.description, description.as_ref());
    }

    ub.transaction(&mut tx)
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .exec()
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
