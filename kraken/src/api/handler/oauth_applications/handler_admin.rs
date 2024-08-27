use actix_web::delete;
use actix_web::get;
use actix_web::post;
use actix_web::put;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use rorm::insert;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::oauth_applications::schema::CreateAppRequest;
use crate::api::handler::oauth_applications::schema::FullOauthClient;
use crate::api::handler::oauth_applications::schema::ListOauthApplications;
use crate::api::handler::oauth_applications::schema::UpdateAppRequest;
use crate::chan::global::GLOBAL;
use crate::models::OauthClient;

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
    request: Json<CreateAppRequest>,
) -> ApiResult<Json<UuidResponse>> {
    let CreateAppRequest { name, redirect_uri } = request.into_inner();
    let uuid = insert!(&GLOBAL.db, OauthClient)
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

#[utoipa::path(
    tag = "OAuth Application",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns all oauth applications", body = ListOauthApplications),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/applications")]
pub(crate) async fn get_all_oauth_apps() -> ApiResult<Json<ListOauthApplications>> {
    Ok(Json(ListOauthApplications {
        apps: query!(&GLOBAL.db, FullOauthClient).all().await?,
    }))
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
pub(crate) async fn get_oauth_app(path: Path<PathUuid>) -> ApiResult<Json<FullOauthClient>> {
    let OauthClient {
        uuid,
        name,
        redirect_uri,
        secret,
    } = query!(&GLOBAL.db, OauthClient)
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

    request: Json<UpdateAppRequest>,
) -> ApiResult<HttpResponse> {
    let UpdateAppRequest { name, redirect_uri } = request.into_inner();

    let affected = update!(&GLOBAL.db, OauthClient)
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
pub(crate) async fn delete_oauth_app(path: Path<PathUuid>) -> ApiResult<HttpResponse> {
    let affected = rorm::delete!(&GLOBAL.db, OauthClient)
        .condition(OauthClient::F.uuid.equals(path.uuid))
        .await?;

    if affected == 0 {
        Err(ApiError::InvalidUuid)
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}
