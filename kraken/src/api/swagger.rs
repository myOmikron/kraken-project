//! This module holds the swagger definitions.
//!
//! They got created with [utoipa].

use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::api::handler;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("id"))),
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handler::test,
        handler::login,
        handler::logout,
        handler::start_auth,
        handler::finish_auth,
        handler::start_register,
        handler::finish_register,
        handler::create_leech,
        handler::delete_leech,
        handler::get_all_leeches,
        handler::get_leech,
        handler::update_leech,
        handler::websocket,
        handler::create_user,
        handler::delete_user,
        handler::get_user,
        handler::get_all_users,
        handler::get_me,
        handler::set_password,
        handler::create_workspace,
        handler::delete_workspace,
        handler::get_workspace,
        handler::get_all_workspaces,
        handler::get_workspace_admin,
        handler::get_all_workspaces_admin,
    ),
    components(schemas(
        handler::ApiErrorResponse,
        handler::ApiStatusCode,
        handler::LoginRequest,
        handler::FinishRegisterRequest,
        handler::CreateLeechResponse,
        handler::CreateLeechRequest,
        handler::GetLeech,
        handler::GetLeechResponse,
        handler::UpdateLeechRequest,
        handler::CreateUserRequest,
        handler::CreateUserResponse,
        handler::GetUser,
        handler::GetUserResponse,
        handler::SetPasswordRequest,
        handler::CreateWorkspaceRequest,
        handler::CreateWorkspaceResponse,
        handler::GetWorkspace,
        handler::GetWorkspaceResponse,
    )),
    modifiers(&SecurityAddon),
)]
pub(crate) struct ApiDoc;
