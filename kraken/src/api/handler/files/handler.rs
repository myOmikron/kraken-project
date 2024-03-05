use std::fs::File;

use actix_files::NamedFile;
use actix_web::get;
use actix_web::post;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::web::Payload;
use actix_web::web::Query;
use log::error;
use rorm::and;
use rorm::prelude::*;
use rorm::query;
use rorm::FieldAccess;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::files::schema::PathFile;
use crate::api::handler::files::schema::UploadQuery;
use crate::api::handler::files::utils::stream_into_file;
use crate::api::handler::files::utils::StreamIntoFileError;
use crate::chan::global::GLOBAL;
use crate::config::VAR_DIR;
use crate::models::MediaFile;
use crate::models::Workspace;

/// Uploads a file to the workspace
///
/// The returned uuid can be used to attach the file for example to a finding.
#[utoipa::path(
    tag = "Files",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "File has been uploaded successfully", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = Vec<u8>,
    params(PathUuid, UploadQuery),
    security(("api_key" = []))
)]
#[post("/workspace/{uuid}/files")]
pub async fn upload_file(
    path: Path<PathUuid>,
    Query(query): Query<UploadQuery>,
    SessionUser(user_uuid): SessionUser,
    body: Payload,
) -> ApiResult<Json<UuidResponse>> {
    let workspace_uuid = path.into_inner().uuid;
    let file_uuid = Uuid::new_v4();

    let file_path = format!("{VAR_DIR}/media/{file_uuid}");
    let (delete_file_guard, sha256) = stream_into_file::<sha2::Sha256>(file_path.as_ref(), body)
        .await
        .map_err(|err| match err {
            StreamIntoFileError::Actix(err) => ApiError::PayloadError(err),
            StreamIntoFileError::FileCreate(err)
            | StreamIntoFileError::FileWrite(err)
            | StreamIntoFileError::FileClose(err) => {
                error!("Failed to write uploaded file to tmp: {err}");
                ApiError::InternalServerError
            }
        })?;

    let mut tx = GLOBAL.db.start_transaction().await?;
    if let Some((uuid,)) = query!(&mut tx, (MediaFile::F.uuid,))
        .condition(and![
            MediaFile::F.workspace.equals(workspace_uuid),
            MediaFile::F.user.equals(user_uuid),
            MediaFile::F.name.equals(&query.filename),
            MediaFile::F.sha256.equals(&sha256)
        ])
        .optional()
        .await?
    {
        return Ok(Json(UuidResponse { uuid }));
    }

    MediaFile::insert(
        &mut tx,
        file_uuid,
        query.filename,
        sha256,
        false,
        user_uuid,
        workspace_uuid,
    )
    .await?;
    tx.commit().await?;

    delete_file_guard.dont();
    Ok(Json(UuidResponse { uuid: file_uuid }))
}

/// Downloads a file from the workspace
#[utoipa::path(
    tag = "Files",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "File has been downloaded successfully", body = Vec<u8>),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathFile),
    security(("api_key" = []))
)]
#[get("/workspace/{w_uuid}/files/{f_uuid}")]
pub async fn download_file(
    path: Path<PathFile>,
    SessionUser(u_uuid): SessionUser,
) -> ApiResult<NamedFile> {
    let PathFile { w_uuid, f_uuid } = path.into_inner();

    if !Workspace::is_user_member_or_owner(&GLOBAL.db, w_uuid, u_uuid).await? {
        return Err(ApiError::NotFound);
    }

    let file = query!(&GLOBAL.db, MediaFile)
        .condition(MediaFile::F.uuid.equals(f_uuid))
        .optional()
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(File::open(format!("{VAR_DIR}/media/{f_uuid}"))
        .and_then(|x| NamedFile::from_file(x, file.name))
        .map(|file| file.use_etag(true).use_last_modified(true))
        .map_err(|err| {
            error!("Failed to open file for download: {err}");
            ApiError::InternalServerError
        })?)
}
