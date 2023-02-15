use actix_web::web::{Data, Json};
use rorm::Database;
use serde::{Deserialize, Serialize};

use crate::api::handler::ApiResult;
use crate::modules::user;

#[derive(Deserialize)]
pub(crate) struct CreateUserRequest {
    pub(crate) username: String,
    pub(crate) display_name: String,
    pub(crate) password: String,
    pub(crate) admin: bool,
}

#[derive(Serialize)]
pub(crate) struct CreateUserResponse {}

pub(crate) async fn create_user(
    req: Json<CreateUserRequest>,
    db: Data<Database>,
) -> ApiResult<Json<CreateUserResponse>> {
    user::create_user_transaction(
        req.username.clone(),
        req.display_name.clone(),
        req.password.clone(),
        req.admin,
        &db,
    )
    .await?;

    Ok(Json(CreateUserResponse {}))
}
