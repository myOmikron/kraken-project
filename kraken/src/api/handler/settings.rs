//! This module holds the handler to retrieve and update settings

use std::sync::Arc;

use actix_web::web::{Data, Json};
use actix_web::{get, put, HttpResponse};
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{ApiError, ApiResult};
use crate::chan::SettingsManagerChan;
use crate::models::{SettingsInsert, UserPermission};

/// The live settings of kraken
#[derive(Serialize, Clone, ToSchema, Debug)]
pub struct SettingsFull {
    /// Require mfa for local users
    pub mfa_required: bool,

    /// The default permission a user from oidc is set to
    pub oidc_initial_permission_level: UserPermission,

    /// The email for the dehashed account
    #[schema(example = "foo@example.com")]
    pub dehashed_email: Option<String>,

    /// The api key for the dehashed account
    #[schema(example = "1231kb3kkb51kj31kjb231kj3b1jk23bkj123")]
    pub dehashed_api_key: Option<String>,

    /// The point in time the settings were created
    pub created_at: DateTime<Utc>,
}

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
pub async fn get_settings(
    settings_chan: Data<Arc<SettingsManagerChan>>,
) -> ApiResult<Json<SettingsFull>> {
    let settings = settings_chan.get_settings();
    Ok(Json(SettingsFull {
        mfa_required: settings.mfa_required,
        created_at: settings.created_at,
        dehashed_email: settings.dehashed_email,
        dehashed_api_key: settings.dehashed_api_key,
        oidc_initial_permission_level: settings.oidc_initial_permission_level,
    }))
}

/// The request to update the settings
#[derive(Deserialize, Clone, Debug, ToSchema)]
pub struct UpdateSettingsRequest {
    /// Require mfa for local users
    pub mfa_required: bool,

    /// The default permission a user from oidc is set to
    pub oidc_initial_permission_level: UserPermission,

    /// The email for the dehashed account
    #[schema(example = "foo@example.com")]
    pub dehashed_email: Option<String>,

    /// The api key for the dehashed account
    #[schema(example = "1231kb3kkb51kj31kjb231kj3b1jk23bkj123")]
    pub dehashed_api_key: Option<String>,
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
pub async fn update_settings(
    req: Json<UpdateSettingsRequest>,
    settings_chan: Data<Arc<SettingsManagerChan>>,
) -> ApiResult<HttpResponse> {
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

    settings_chan
        .update_settings(&SettingsInsert {
            uuid: Uuid::new_v4(),
            mfa_required: req.mfa_required,
            oidc_initial_permission_level: req.oidc_initial_permission_level,
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
