//! Endpoints for listing and managing wordlists

use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use rorm::prelude::*;
use rorm::{query, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{ApiError, ApiResult, PathUuid, UuidResponse};
use crate::models::WordList;

/// Response containing all wordlists
#[derive(Serialize, ToSchema)]
pub struct GetAllWordlistsResponse {
    /// List of all wordlists
    pub wordlists: Vec<SimpleWordlist>,
}

/// A wordlist without its `path` field
#[derive(Serialize, ToSchema, Patch)]
#[rorm(model = "WordList")]
pub struct SimpleWordlist {
    /// The primary key of the wordlist
    pub uuid: Uuid,

    /// The wordlist's name to be displayed select buttons
    #[schema(example = "subdomains-top1million-5000.txt")]
    pub name: String,

    /// A description explaining the wordlist's intended use case
    #[schema(example = "List of 5000 subdomains")]
    pub description: String,
}

/// Get a list of all wordlist for the user to select from when starting an bruteforce subdomains attack
#[utoipa::path(
    tag = "Wordlist",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Matched leeches", body = GetAllWordlistsResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    security(("api_key" = []))
)]
#[get("/wordlists")]
pub async fn get_all_wordlists(db: Data<Database>) -> ApiResult<Json<GetAllWordlistsResponse>> {
    Ok(Json(GetAllWordlistsResponse {
        wordlists: query!(db.as_ref(), SimpleWordlist).all().await?,
    }))
}

/// Arguments for creating a new wordlist
#[derive(Deserialize, ToSchema)]
pub struct CreateWordlistRequest {
    /// The wordlist's name to be displayed select buttons
    #[schema(example = "subdomains-top1million-5000.txt")]
    pub name: String,

    /// A description explaining the wordlist's intended use case
    #[schema(example = "List of 5000 subdomains")]
    pub description: String,

    /// The file path the wordlist is deployed under on each leech
    #[schema(example = "/opt/wordlists/Discovery/DNS/subdomains-top1million-5000.txt")]
    pub path: String,
}

/// Create a new wordlist
#[utoipa::path(
    tag = "Wordlist management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Wordlist got created successfully", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = CreateWordlistRequest,
    security(("api_key" = []))
)]
#[post("/wordlists")]
pub async fn create_wordlist_admin(
    db: Data<Database>,
    req: Json<CreateWordlistRequest>,
) -> ApiResult<Json<UuidResponse>> {
    let CreateWordlistRequest {
        name,
        description,
        path,
    } = req.into_inner();
    Ok(Json(UuidResponse {
        uuid: WordList::insert(db.as_ref(), name, description, path).await?,
    }))
}

/// Response containing all wordlists including their `path` fields
#[derive(Serialize, ToSchema)]
pub struct GetAllWordlistsAdminResponse {
    /// List of all wordlists including their `path` fields
    pub wordlists: Vec<FullWordlist>,
}

/// A wordlist including its `path` field only meant for admins
#[derive(Serialize, ToSchema, Patch)]
#[rorm(model = "WordList")]
pub struct FullWordlist {
    /// The primary key of the wordlist
    pub uuid: Uuid,

    /// The wordlist's name to be displayed select buttons
    #[schema(example = "subdomains-top1million-5000.txt")]
    pub name: String,

    /// A description explaining the wordlist's intended use case
    #[schema(example = "List of 5000 subdomains")]
    pub description: String,

    /// The file path the wordlist is deployed under on each leech
    #[schema(example = "/opt/wordlists/Discovery/DNS/subdomains-top1million-5000.txt")]
    pub path: String,
}

/// Get a list of all wordlists including their paths
#[utoipa::path(
    tag = "Wordlist management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "List of all wordlists", body = GetAllWordlistsAdminResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/wordlists")]
pub async fn get_all_wordlists_admin(
    db: Data<Database>,
) -> ApiResult<Json<GetAllWordlistsAdminResponse>> {
    Ok(Json(GetAllWordlistsAdminResponse {
        wordlists: query!(db.as_ref(), FullWordlist).all().await?,
    }))
}

/// Arguments for updating an existing wordlist
#[derive(Deserialize, ToSchema)]
pub struct UpdateWordlistRequest {
    /// The primary key of the wordlist to update
    pub uuid: Uuid,

    /// The wordlist's name to be displayed select buttons
    #[schema(example = "subdomains-top1million-5000.txt")]
    pub name: Option<String>,

    /// A description explaining the wordlist's intended use case
    #[schema(example = "List of 5000 subdomains")]
    pub description: Option<String>,

    /// The file path the wordlist is deployed under on each leech
    #[schema(example = "/opt/wordlists/Discovery/DNS/subdomains-top1million-5000.txt")]
    pub path: Option<String>,
}

/// Update an existing wordlist
#[utoipa::path(
    tag = "Wordlist management",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Wordlist got updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    request_body = UpdateWordlistRequest,
    security(("api_key" = []))
)]
#[put("/wordlists/{uuid}")]
pub async fn update_wordlist_admin(
    db: Data<Database>,
    req: Json<UpdateWordlistRequest>,
) -> ApiResult<HttpResponse> {
    let UpdateWordlistRequest {
        uuid,
        name,
        description,
        path,
    } = req.into_inner();
    WordList::update(db.as_ref(), uuid, name, description, path).await?;
    Ok(HttpResponse::Ok().finish())
}

/// Delete an existing wordlist
#[utoipa::path(
    tag = "Wordlist management",
    context_path = "/api/v1/admin",
        responses(
        (status = 200, description = "Wordlist got deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/wordlists/{uuid}")]
pub async fn delete_wordlist_admin(
    db: Data<Database>,
    path: Path<PathUuid>,
) -> ApiResult<HttpResponse> {
    let deleted = rorm::delete!(db.as_ref(), WordList)
        .condition(WordList::F.uuid.equals(path.uuid))
        .await?;
    if deleted > 0 {
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(ApiError::InvalidUuid)
    }
}
