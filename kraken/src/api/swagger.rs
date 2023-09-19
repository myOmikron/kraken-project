//! This module holds the swagger definitions.
//!
//! They got created with [utoipa].

use utoipa::openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::api::handler;
use crate::api::handler::{apikeys, oauth};
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
        handler::update_me,
        handler::set_password,
        handler::create_workspace,
        handler::delete_workspace,
        handler::get_workspace,
        handler::get_all_workspaces,
        handler::update_workspace,
        handler::get_workspace_admin,
        handler::get_all_workspaces_admin,
        handler::bruteforce_subdomains,
        handler::scan_tcp_ports,
        handler::query_certificate_transparency,
        handler::report_workspace_results,
        handler::delete_attack,
        handler::get_attack,
        handler::get_tcp_port_scan_results,
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
        handler::get_settings,
        handler::update_settings,
        apikeys::create_api_key,
        apikeys::delete_api_key,
        apikeys::get_api_keys,
        apikeys::update_leech,
    ),
    components(schemas(
        handler::ApiErrorResponse,
        handler::ApiStatusCode,
        handler::LoginRequest,
        handler::FinishRegisterRequest,
        handler::CreateLeechRequest,
        handler::GetLeech,
        handler::GetLeechResponse,
        handler::UpdateLeechRequest,
        handler::CreateUserRequest,
        handler::CreateUserResponse,
        handler::GetUser,
        handler::GetUserResponse,
        handler::UserResponse,
        handler::UpdateMeRequest,
        handler::SetPasswordRequest,
        handler::CreateWorkspaceRequest,
        handler::SimpleWorkspace,
        handler::FullWorkspace,
        handler::SimpleAttack,
        handler::GetWorkspaceResponse,
        handler::UpdateWorkspaceRequest,
        handler::BruteforceSubdomainsRequest,
        handler::ScanTcpPortsRequest,
        handler::QueryCertificateTransparencyRequest,
        handler::ReportingWorkspaceResults,
        handler::ReportingTcpPortScanAttack,
        handler::ReportingUser,
        handler::ReportingIpPort,
        handler::PortOrRange,
        handler::PageParams,
        handler::TcpPortScanResultsPage,
        handler::SimpleTcpPortScanResult,
        handler::UuidResponse,
        models::AttackType,
        oauth::CreateAppRequest,
        oauth::SimpleOauthClient,
        oauth::FullOauthClient,
        oauth::GetAppsResponse,
        oauth::UpdateAppRequest,
        oauth::OpenRequestInfo,
        oauth::TokenResponse,
        oauth::TokenErrorResponse,
        handler::SettingsFull,
        handler::UpdateSettingsRequest,
        apikeys::SimpleApiKey,
        apikeys::CreateApiKeyRequest,
        apikeys::GetApiKeysResponse,
        apikeys::UpdateApiKeyRequest,
    )),
    modifiers(&SecurityAddon, &SecurityAddon2),
)]
pub(crate) struct ApiDoc;
