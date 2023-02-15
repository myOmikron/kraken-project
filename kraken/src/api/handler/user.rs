use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use rorm::Database;
use serde::{Deserialize, Serialize};

use crate::api::handler::ApiResult;
use crate::modules::user::create::create_user_transaction;
use crate::modules::user::delete::delete_user_transaction;

#[derive(Deserialize)]
pub(crate) struct CreateUserRequest {
    pub(crate) username: String,
    pub(crate) display_name: String,
    pub(crate) password: String,
    pub(crate) admin: bool,
}

#[derive(Serialize)]
pub(crate) struct CreateUserResponse {
    pub(crate) uuid: String,
}

pub(crate) async fn create_user(
    req: Json<CreateUserRequest>,
    db: Data<Database>,
) -> ApiResult<Json<CreateUserResponse>> {
    let uuid = create_user_transaction(
        req.username.clone(),
        req.display_name.clone(),
        req.password.clone(),
        req.admin,
        &db,
    )
    .await?;

    Ok(Json(CreateUserResponse {
        uuid: uuid.to_string(),
    }))
}

#[derive(Deserialize)]
pub(crate) struct DeleteUserRequest {
    pub(crate) username: String,
}

pub(crate) async fn delete_user(
    req: Path<DeleteUserRequest>,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    delete_user_transaction(req.username.clone(), &db).await?;

    Ok(HttpResponse::Ok().finish())
}
