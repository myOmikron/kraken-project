use actix_web::get;
use actix_web::put;
use actix_web::web::Json;
use actix_web::HttpResponse;
use log::error;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::settings::schema::SettingsFull;
use crate::api::handler::settings::schema::UpdateSettingsRequest;
use crate::chan::global::GLOBAL;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;
use crate::models::SettingsInsert;

/// Retrieve the currently active settings
#[utoipa::path(
    tag = "Settings Management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns the currently active settings", body = SettingsFull),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/settings")]
pub async fn get_settings() -> ApiResult<Json<SettingsFull>> {
    let settings = GLOBAL.settings.get_settings();
    Ok(Json(SettingsFull {
        mfa_required: settings.mfa_required,
        created_at: settings.created_at,
        dehashed_email: settings.dehashed_email,
        dehashed_api_key: settings.dehashed_api_key,
        oidc_initial_permission_level: FromDb::from_db(settings.oidc_initial_permission_level),
    }))
}

/// Update the settings
#[utoipa::path(
    tag = "Settings Management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Settings have been updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = UpdateSettingsRequest,
    security(("api_key" = []))
)]
#[put("/settings")]
pub async fn update_settings(req: Json<UpdateSettingsRequest>) -> ApiResult<HttpResponse> {
    let mut req = req.into_inner();

    if let Some(api_key) = &req.dehashed_api_key {
        if api_key.is_empty() {
            req.dehashed_api_key = None;
        }
    }

    if let Some(email) = &req.dehashed_email {
        if email.is_empty() {
            req.dehashed_email = None;
        }
    }

    GLOBAL
        .settings
        .update_settings(&SettingsInsert {
            uuid: Uuid::new_v4(),
            mfa_required: req.mfa_required,
            oidc_initial_permission_level: req.oidc_initial_permission_level.into_db(),
            dehashed_email: req.dehashed_email,
            dehashed_api_key: req.dehashed_api_key,
        })
        .await
        .map_err(|e| {
            error!("Error updating settings: {e}");

            ApiError::InternalServerError
        })?;

    Ok(HttpResponse::Ok().finish())
}
