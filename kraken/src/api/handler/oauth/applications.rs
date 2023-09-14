use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use rand::distributions::{Alphanumeric, DistString};
use rorm::prelude::*;
use rorm::{insert, query, update, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{ApiError, ApiResult, PathUuid, UuidResponse};
use crate::models::OauthClient;

#[derive(Deserialize, ToSchema)]
pub(crate) struct CreateAppRequest {
    #[schema(example = "Trustworthy application")]
    pub(crate) name: String,

    #[schema(example = "http://127.0.0.1:8080")]
    pub(crate) redirect_uri: String,
}

/// Create a new application
#[utoipa::path(
    tag = "OAuth Application",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Application was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateAppRequest,
    security(("api_key" = []))
)]
#[post("/applications")]
pub(crate) async fn create_oauth_app(
    db: Data<Database>,
    request: Json<CreateAppRequest>,
) -> ApiResult<Json<UuidResponse>> {
    let CreateAppRequest { name, redirect_uri } = request.into_inner();
    let uuid = insert!(db.as_ref(), OauthClient)
        .return_primary_key()
        .single(&OauthClient {
            uuid: Uuid::new_v4(),
            name,
            secret: Alphanumeric.sample_string(&mut rand::thread_rng(), 32),
            redirect_uri,
        })
        .await?;
    Ok(Json(UuidResponse { uuid }))
}

/// A simple (secret-less) version of a workspace
#[derive(Serialize, ToSchema, Patch)]
#[rorm(model = "OauthClient")]
pub(crate) struct SimpleOauthClient {
    pub(crate) uuid: Uuid,
    #[schema(example = "Trustworthy application")]
    pub(crate) name: String,
    #[schema(example = "http://127.0.0.1:8080")]
    pub(crate) redirect_uri: String,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct GetAppsResponse {
    pub(crate) apps: Vec<FullOauthClient>,
}

#[utoipa::path(
    tag = "OAuth Application",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns all oauth applications", body = GetAppsResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/applications")]
pub(crate) async fn get_all_oauth_apps(db: Data<Database>) -> ApiResult<Json<GetAppsResponse>> {
    Ok(Json(GetAppsResponse {
        apps: query!(db.as_ref(), FullOauthClient).all().await?,
    }))
}

/// A complete version of a workspace
#[derive(Serialize, ToSchema, Patch)]
#[rorm(model = "OauthClient")]
pub(crate) struct FullOauthClient {
    pub(crate) uuid: Uuid,
    #[schema(example = "Trustworthy application")]
    pub(crate) name: String,
    #[schema(example = "http://127.0.0.1:8080")]
    pub(crate) redirect_uri: String,
    #[schema(example = "IPSPL29BSDw5HFir5LYamdlm6SiaBdwx")]
    pub(crate) secret: String,
}

#[utoipa::path(
    tag = "OAuth Application",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns an oauth applications", body = FullOauthClient),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/applications/{uuid}")]
pub(crate) async fn get_oauth_app(
    db: Data<Database>,
    path: Path<PathUuid>,
) -> ApiResult<Json<FullOauthClient>> {
    let OauthClient {
        uuid,
        name,
        redirect_uri,
        secret,
    } = query!(db.as_ref(), OauthClient)
        .condition(OauthClient::F.uuid.equals(path.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;
    Ok(Json(FullOauthClient {
        uuid,
        name,
        redirect_uri,
        secret,
    }))
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct UpdateAppRequest {
    #[schema(example = "Trustworthy application")]
    pub(crate) name: Option<String>,

    #[schema(example = "http://127.0.0.1:8080")]
    pub(crate) redirect_uri: Option<String>,
}

/// Update an application
#[utoipa::path(
    tag = "OAuth Application",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Application got updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    request_body = UpdateAppRequest,
    security(("api_key" = []))
)]
#[put("/applications/{uuid}")]
pub(crate) async fn update_oauth_app(
    path: Path<PathUuid>,
    db: Data<Database>,
    request: Json<UpdateAppRequest>,
) -> ApiResult<HttpResponse> {
    let UpdateAppRequest { name, redirect_uri } = request.into_inner();

    let affected = update!(db.as_ref(), OauthClient)
        .condition(OauthClient::F.uuid.equals(path.uuid))
        .begin_dyn_set()
        .set_if(OauthClient::F.name, name)
        .set_if(OauthClient::F.redirect_uri, redirect_uri)
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .await?;

    if affected == 0 {
        Err(ApiError::InvalidUuid)
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

/// Delete an application
#[utoipa::path(
    tag = "OAuth Application",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Application was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/applications/{uuid}")]
pub(crate) async fn delete_oauth_app(
    path: Path<PathUuid>,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let affected = rorm::delete!(db.as_ref(), OauthClient)
        .condition(OauthClient::F.uuid.equals(path.uuid))
        .await?;

    if affected == 0 {
        Err(ApiError::InvalidUuid)
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}
