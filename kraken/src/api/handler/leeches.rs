use std::str::FromStr;

use actix_web::http::Uri;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use rorm::{delete, insert, query, Database, Model};
use serde::{Deserialize, Serialize};

use crate::api::handler::{ApiError, ApiResult, PathId};
use crate::models::{Leech, LeechInsert};

#[derive(Deserialize)]
pub(crate) struct CreateLeechRequest {
    pub(crate) name: String,
    pub(crate) address: String,
}

#[derive(Serialize)]
pub(crate) struct CreateLeechResponse {
    pub(crate) id: i64,
}

pub(crate) async fn create_leech(
    req: Json<CreateLeechRequest>,
    db: Data<Database>,
) -> ApiResult<Json<CreateLeechResponse>> {
    let mut tx = db.start_transaction().await?;

    let address = Uri::from_str(req.address.as_str()).map_err(|_| ApiError::InvalidAddress)?;
    if let Some(scheme) = address.scheme_str() {
        if scheme != "https" && scheme != "http" {
            return Err(ApiError::InvalidAddress);
        }
    } else {
        return Err(ApiError::InvalidAddress);
    }

    if query!(&db, Leech)
        .transaction(&mut tx)
        .condition(Leech::F.address.equals(&address.to_string()))
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
            address: address.to_string(),
        })
        .await?;

    tx.commit().await?;

    Ok(Json(CreateLeechResponse { id }))
}

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

    delete!(&db, Leech)
        .transaction(&mut tx)
        .condition(Leech::F.id.equals(path.id as i64))
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub(crate) struct GetLeechRequest {
    id: Option<u32>,
}

#[derive(Serialize)]
pub(crate) struct GetLeech {
    id: i64,
    name: String,
    address: String,
}

#[derive(Serialize)]
pub(crate) struct GetLeechResponse {
    leeches: Vec<GetLeech>,
}

pub(crate) async fn get_leech(
    req: Path<GetLeechRequest>,
    db: Data<Database>,
) -> ApiResult<Json<GetLeechResponse>> {
    let leeches = if let Some(id) = req.id {
        let leech = query!(&db, Leech)
            .condition(Leech::F.id.equals(id as i64))
            .all()
            .await?;

        if leech.is_empty() {
            return Err(ApiError::InvalidId);
        }

        leech
    } else {
        query!(&db, Leech).all().await?
    };

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
