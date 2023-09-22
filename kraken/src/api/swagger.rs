//! This module holds the swagger definitions.
//!
//! They got created with [utoipa].

use utoipa::openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::api::handler;
use crate::api::handler::{
    api_keys, attacks, auth, global_tags, hosts, leeches, oauth, settings, users, websocket,
    workspace_tags, workspaces,
};
use crate::models;

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

struct SecurityAddon2;

impl Modify for SecurityAddon2 {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_token",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::test,
        auth::login,
        auth::logout,
        auth::start_auth,
        auth::finish_auth,
        auth::start_register,
        auth::finish_register,
        leeches::create_leech,
        leeches::delete_leech,
        leeches::get_all_leeches,
        leeches::get_leech,
        leeches::update_leech,
        websocket::websocket,
        users::create_user,
        users::delete_user,
        users::get_user,
        users::get_all_users,
        users::get_me,
        users::update_me,
        users::set_password,
        workspaces::create_workspace,
        workspaces::delete_workspace,
        workspaces::get_workspace,
        workspaces::get_all_workspaces,
        workspaces::update_workspace,
        workspaces::get_workspace_admin,
        workspaces::get_all_workspaces_admin,
        attacks::bruteforce_subdomains,
        attacks::scan_tcp_ports,
        attacks::query_certificate_transparency,
        attacks::delete_attack,
        attacks::get_attack,
        attacks::get_tcp_port_scan_results,
        oauth::create_oauth_app,
        oauth::get_all_oauth_apps,
        oauth::get_oauth_app,
        oauth::update_oauth_app,
        oauth::delete_oauth_app,
        oauth::info,
        oauth::auth,
        oauth::accept,
        oauth::deny,
        oauth::token,
        settings::get_settings,
        settings::update_settings,
        api_keys::create_api_key,
        api_keys::delete_api_key,
        api_keys::get_api_keys,
        api_keys::update_api_key,
        hosts::get_all_hosts,
        hosts::get_host,
        global_tags::create_global_tag,
        global_tags::get_all_global_tags,
        global_tags::update_global_tag,
        global_tags::delete_global_tag,
        workspace_tags::create_workspace_tag,
        workspace_tags::get_all_workspace_tags,
        workspace_tags::update_workspace_tag,
        workspace_tags::delete_workspace_tag,
    ),
    components(schemas(
        handler::ApiErrorResponse,
        handler::ApiStatusCode,
        handler::UuidResponse,
        auth::LoginRequest,
        auth::FinishRegisterRequest,
        leeches::CreateLeechRequest,
        leeches::GetLeech,
        leeches::GetLeechResponse,
        leeches::UpdateLeechRequest,
        users::CreateUserRequest,
        users::CreateUserResponse,
        users::GetUser,
        users::GetUserResponse,
        users::UserResponse,
        users::UpdateMeRequest,
        users::SetPasswordRequest,
        workspaces::CreateWorkspaceRequest,
        workspaces::SimpleWorkspace,
        workspaces::FullWorkspace,
        workspaces::GetWorkspaceResponse,
        workspaces::UpdateWorkspaceRequest,
        attacks::SimpleAttack,
        attacks::BruteforceSubdomainsRequest,
        attacks::ScanTcpPortsRequest,
        attacks::QueryCertificateTransparencyRequest,
        attacks::PortOrRange,
        attacks::PageParams,
        attacks::TcpPortScanResultsPage,
        attacks::SimpleTcpPortScanResult,
        models::AttackType,
        oauth::CreateAppRequest,
        oauth::SimpleOauthClient,
        oauth::FullOauthClient,
        oauth::GetAppsResponse,
        oauth::UpdateAppRequest,
        oauth::OpenRequestInfo,
        oauth::TokenResponse,
        oauth::TokenErrorResponse,
        oauth::Pkce,
        oauth::TokenType,
        oauth::GrantType,
        oauth::TokenRequest,
        oauth::CodeChallengeMethod,
        settings::SettingsFull,
        settings::UpdateSettingsRequest,
        api_keys::SimpleApiKey,
        api_keys::CreateApiKeyRequest,
        api_keys::GetApiKeysResponse,
        api_keys::UpdateApiKeyRequest,
        hosts::GetAllHostsResponse,
        hosts::SimpleHost,
        models::OsType,
        global_tags::CreateGlobalTagRequest,
        handler::Color,
        global_tags::FullGlobalTag,
        global_tags::GetGlobalTagsResponse,
        global_tags::UpdateGlobalTag,
        models::PortProtocol,
        workspace_tags::FullWorkspaceTag,
        workspace_tags::GetWorkspaceTagsResponse,
        workspace_tags::UpdateWorkspaceTag,
        workspace_tags::CreateWorkspaceTagRequest,
    )),
    modifiers(&SecurityAddon, &SecurityAddon2),
)]
pub(crate) struct ApiDoc;
