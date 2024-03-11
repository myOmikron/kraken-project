use std::fs::File;

use actix_files::NamedFile;
use actix_web::delete;
use actix_web::get;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::web::Query;
use actix_web::HttpResponse;
use futures::TryStreamExt;
use log::error;
use rorm::conditions::Condition;
use rorm::conditions::DynamicCollection;
use rorm::prelude::*;
use rorm::query;
use tokio::fs;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::Page;
use crate::api::handler::common::schema::PageParams;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::utils::get_page_params;
use crate::api::handler::files::schema::FullFile;
use crate::api::handler::files::schema::GetAllFilesQuery;
use crate::api::handler::files::utils::media_file_path;
use crate::api::handler::files::utils::media_thumbnail_path;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::chan::global::GLOBAL;
use crate::models::MediaFile;
use crate::models::User;
use crate::models::Workspace;

/// Retrieve all files
#[utoipa::path(
    tag = "Admin Files",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "The requested page of files", body = FullFilesPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
        ),
    params(PageParams, GetAllFilesQuery),
    security(("api_key" = []))
)]
#[get("/files")]
pub async fn get_all_files_admin(
    Query(page): Query<PageParams>,
    Query(filter): Query<GetAllFilesQuery>,
) -> ApiResult<Json<Page<FullFile>>> {
    let (limit, offset) = get_page_params(page).await?;

    let workspace = filter
        .workspace
        .map(|uuid| MediaFile::F.workspace.equals(uuid));
    let user = filter.user.map(|uuid| MediaFile::F.user.equals(uuid));

    let mut tx = GLOBAL.db.start_transaction().await?;

    let mut conditions = Vec::with_capacity(3);
    conditions.push(rorm::conditions::Value::Bool(true).boxed());
    conditions.extend(workspace.clone().map(Condition::boxed));
    conditions.extend(user.clone().map(Condition::boxed));
    let conditions = DynamicCollection::and(conditions);

    let (total,) = query!(&mut tx, (MediaFile::F.uuid.count(),))
        .condition(conditions)
        .one()
        .await?;

    let mut conditions = Vec::with_capacity(3);
    conditions.push(rorm::conditions::Value::Bool(true).boxed());
    conditions.extend(workspace.map(Condition::boxed));
    conditions.extend(user.map(Condition::boxed));
    let conditions = DynamicCollection::and(conditions);

    let items = query!(
        &mut tx,
        (
            MediaFile::F.uuid,
            MediaFile::F.name,
            MediaFile::F.sha256,
            MediaFile::F.is_image,
            MediaFile::F.user as User,
            MediaFile::F.workspace as Workspace,
            MediaFile::F.workspace.owner as User,
            MediaFile::F.created_at,
        )
    )
    .condition(conditions)
    .limit(limit)
    .offset(offset)
    .stream()
    .map_ok(
        |(uuid, name, sha256, is_image, user, workspace, owner, uploaded_at)| FullFile {
            uuid,
            name,
            sha256,
            is_image,
            user: SimpleUser {
                uuid: user.uuid,
                username: user.username,
                display_name: user.display_name,
            },
            workspace: SimpleWorkspace {
                uuid: workspace.uuid,
                name: workspace.name,
                description: workspace.description,
                owner: SimpleUser {
                    uuid: owner.uuid,
                    username: owner.username,
                    display_name: owner.display_name,
                },
                created_at: workspace.created_at,
            },
            uploaded_at,
        },
    )
    .try_collect()
    .await?;

    tx.commit().await?;

    Ok(Json(Page {
        items,
        total: total as u64,
        offset: page.offset,
        limit: page.limit,
    }))
}

/// Downloads a file
#[utoipa::path(
    tag = "Admin Files",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "File has been downloaded successfully", body = Vec<u8>),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/files/{uuid}")]
pub async fn download_file_admin(path: Path<PathUuid>) -> ApiResult<NamedFile> {
    let uuid = path.into_inner().uuid;

    let file = query!(&GLOBAL.db, MediaFile)
        .condition(MediaFile::F.uuid.equals(uuid))
        .optional()
        .await?
        .ok_or(ApiError::NotFound)?;

    File::open(media_file_path(uuid))
        .and_then(|x| NamedFile::from_file(x, file.name))
        .map(|file| file.use_etag(true).use_last_modified(true))
        .map_err(|err| {
            error!("Failed to open file for download: {err}");
            ApiError::InternalServerError
        })
}

/// Deletes a file
#[utoipa::path(
    tag = "Admin Files",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "File has been deleted successfully"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/files/{uuid}")]
pub async fn delete_file_admin(path: Path<PathUuid>) -> ApiResult<HttpResponse> {
    let uuid = path.into_inner().uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;
    let (is_image,) = query!(&mut tx, (MediaFile::F.is_image,))
        .condition(MediaFile::F.uuid.equals(uuid))
        .optional()
        .await?
        .ok_or(ApiError::NotFound)?;
    rorm::delete!(&mut tx, MediaFile)
        .condition(MediaFile::F.uuid.equals(uuid))
        .await?;
    fs::remove_file(media_file_path(uuid))
        .await
        .map_err(|err| {
            error!("Failed to delete file: {err}");
            ApiError::InternalServerError
        })?;
    if is_image {
        fs::remove_file(media_thumbnail_path(uuid))
            .await
            .map_err(|err| {
                error!("Failed to delete thumbnail: {err}");
                ApiError::InternalServerError
            })?;
    }
    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
