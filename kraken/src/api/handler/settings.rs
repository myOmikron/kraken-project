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
use crate::models::{Settings, SettingsInsert};

/// The live settings of kraken
#[derive(Serialize, Clone, ToSchema, Debug)]
pub struct SettingsFull {
    pub(crate) created_at: DateTime<Utc>,
    #[schema(example = "foo@example.com")]
    pub(crate) dehashed_email: Option<String>,
    #[schema(example = "1231kb3kkb51kj31kjb231kj3b1jk23bkj123")]
    pub(crate) dehashed_api_key: Option<String>,
}

impl From<Settings> for SettingsFull {
    fn from(value: Settings) -> Self {
        Self {
            created_at: value.created_at,
            dehashed_email: value.dehashed_email,
            dehashed_api_key: value.dehashed_api_key,
        }
    }
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
    Ok(Json(settings.into()))
}

/// The request to update the settings
#[derive(Deserialize, Clone, Debug, ToSchema)]
pub struct UpdateSettingsRequest {
    dehashed_email: Option<String>,
    dehashed_api_key: Option<String>,
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
