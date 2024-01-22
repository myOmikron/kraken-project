//! This module holds the swagger definitions.
//!
//! They got created with [utoipa].

use utoipa::openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::api::handler::{
    aggregation_source, api_keys, attack_results, attacks, auth, common, data_export, domains,
    global_tags, hosts, leeches, oauth, oauth_applications, oauth_decisions, ports, services,
    settings, users, websocket, wordlists, workspace_invitations, workspace_tags, workspaces,
};
use crate::modules::oauth::schemas as oauth_schemas;
use crate::{chan, models};

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
        auth::handler::test,
        auth::handler::login,
        auth::handler::logout,
        auth::handler::start_auth,
        auth::handler::finish_auth,
        auth::handler::start_register,
        auth::handler::finish_register,
        leeches::handler_admin::create_leech,
        leeches::handler_admin::delete_leech,
        leeches::handler_admin::get_all_leeches,
        leeches::handler_admin::get_leech,
        leeches::handler_admin::update_leech,
        leeches::handler_admin::gen_leech_config,
        websocket::websocket,
        users::handler_admin::create_user,
        users::handler_admin::delete_user,
        users::handler_admin::get_user,
        users::handler_admin::get_all_users_admin,
        users::handler::get_me,
        users::handler::update_me,
        users::handler::set_password,
        users::handler::get_all_users,
        workspaces::handler::create_workspace,
        workspaces::handler::delete_workspace,
        workspaces::handler::get_workspace,
        workspaces::handler::get_all_workspaces,
        workspaces::handler::update_workspace,
        workspaces::handler_admin::get_workspace_admin,
        workspaces::handler_admin::get_all_workspaces_admin,
        workspaces::handler::transfer_ownership,
        workspaces::handler::create_invitation,
        workspaces::handler::retract_invitation,
        workspaces::handler::get_all_workspace_invitations,
        workspaces::handler::search,
        workspaces::handler::get_searches,
        workspaces::handler::get_search_results,
        attacks::handler::bruteforce_subdomains,
        attacks::handler::scan_tcp_ports,
        attacks::handler::query_certificate_transparency,
        attacks::handler::delete_attack,
        attacks::handler::get_attack,
        attacks::handler::get_all_attacks,
        attacks::handler::get_workspace_attacks,
        attacks::handler::query_dehashed,
        attacks::handler::hosts_alive_check,
        attacks::handler::service_detection,
        attacks::handler::udp_service_detection,
        attacks::handler::dns_resolution,
        attacks::handler::testssl,
        attack_results::handler::get_bruteforce_subdomains_results,
        attack_results::handler::get_tcp_port_scan_results,
        attack_results::handler::get_query_certificate_transparency_results,
        attack_results::handler::get_query_unhashed_results,
        attack_results::handler::get_host_alive_results,
        attack_results::handler::get_service_detection_results,
        attack_results::handler::get_udp_service_detection_results,
        attack_results::handler::get_dns_resolution_results,
        attack_results::handler::get_testssl_results,
        oauth_applications::handler_admin::create_oauth_app,
        oauth_applications::handler_admin::get_all_oauth_apps,
        oauth_applications::handler_admin::get_oauth_app,
        oauth_applications::handler_admin::update_oauth_app,
        oauth_applications::handler_admin::delete_oauth_app,
        oauth::handler::info,
        oauth::handler::accept,
        oauth::handler::deny,
        oauth_decisions::handler::get_decisions,
        oauth_decisions::handler::revoke_decision,
        settings::handler_admin::get_settings,
        settings::handler_admin::update_settings,
        api_keys::handler::create_api_key,
        api_keys::handler::delete_api_key,
        api_keys::handler::get_api_keys,
        api_keys::handler::update_api_key,
        global_tags::handler_admin::create_global_tag,
        global_tags::handler::get_all_global_tags,
        global_tags::handler_admin::update_global_tag,
        global_tags::handler_admin::delete_global_tag,
        workspace_tags::handler::create_workspace_tag,
        workspace_tags::handler::get_all_workspace_tags,
        workspace_tags::handler::update_workspace_tag,
        workspace_tags::handler::delete_workspace_tag,
        hosts::handler::get_all_hosts,
        hosts::handler::get_host,
        hosts::handler::create_host,
        hosts::handler::update_host,
        hosts::handler::delete_host,
        hosts::handler::get_host_sources,
        hosts::handler::get_host_relations,
        ports::handler::get_all_ports,
        ports::handler::get_port,
        ports::handler::create_port,
        ports::handler::update_port,
        ports::handler::delete_port,
        ports::handler::get_port_sources,
        ports::handler::get_port_relations,
        services::handler::get_all_services,
        services::handler::get_service,
        services::handler::create_service,
        services::handler::update_service,
        services::handler::delete_service,
        services::handler::get_service_sources,
        services::handler::get_service_relations,
        domains::handler::get_all_domains,
        domains::handler::get_domain,
        domains::handler::create_domain,
        domains::handler::update_domain,
        domains::handler::delete_domain,
        domains::handler::get_domain_sources,
        domains::handler::get_domain_relations,
        wordlists::handler::get_all_wordlists,
        wordlists::handler_admin::create_wordlist_admin,
        wordlists::handler_admin::get_all_wordlists_admin,
        wordlists::handler_admin::update_wordlist_admin,
        wordlists::handler_admin::delete_wordlist_admin,
        workspace_invitations::handler::get_all_invitations,
        workspace_invitations::handler::accept_invitation,
        workspace_invitations::handler::decline_invitation,
    ),
    components(schemas(
        common::schema::ApiErrorResponse,
        common::schema::ApiStatusCode,
        common::schema::UuidResponse,
        common::schema::SimpleTag,
        common::schema::TagType,
        common::schema::PageParams,
        common::schema::TcpPortScanResultsPage,
        aggregation_source::schema::SimpleAggregationSource,
        aggregation_source::schema::FullAggregationSource,
        aggregation_source::schema::ManualInsert,
        aggregation_source::schema::SourceAttack,
        aggregation_source::schema::SourceAttackResult,
        auth::schema::LoginRequest,
        auth::schema::FinishRegisterRequest,
        leeches::schema::CreateLeechRequest,
        leeches::schema::SimpleLeech,
        leeches::schema::ListLeeches,
        leeches::schema::UpdateLeechRequest,
        leeches::schema::LeechConfig,
        leeches::schema::LeechTlsConfig,
        users::schema::CreateUserRequest,
        users::schema::SimpleUser,
        users::schema::FullUser,
        users::schema::ListFullUsers,
        users::schema::UpdateMeRequest,
        users::schema::SetPasswordRequest,
        users::schema::SimpleUser,
        users::schema::ListUsers,
        workspaces::schema::CreateWorkspaceRequest,
        workspaces::schema::SimpleWorkspace,
        workspaces::schema::FullWorkspace,
        workspaces::schema::ListWorkspaces,
        workspaces::schema::UpdateWorkspaceRequest,
        workspaces::schema::TransferWorkspaceRequest,
        workspaces::schema::InviteToWorkspaceRequest,
        workspaces::schema::SearchWorkspaceRequest,
        workspaces::schema::SearchEntry,
        workspaces::schema::SearchResultEntry,
        attacks::schema::SimpleAttack,
        attacks::schema::ListAttacks,
        attacks::schema::BruteforceSubdomainsRequest,
        attacks::schema::HostsAliveRequest,
        attacks::schema::ScanTcpPortsRequest,
        attacks::schema::QueryCertificateTransparencyRequest,
        attacks::schema::PortOrRange,
        attacks::schema::ServiceDetectionRequest,
        attacks::schema::UdpServiceDetectionRequest,
        attacks::schema::DnsResolutionRequest,
        attacks::schema::TestSSLRequest,
        attacks::schema::DomainOrNetwork,
        attacks::schema::StartTLSProtocol,
        attack_results::schema::SimpleBruteforceSubdomainsResult,
        attack_results::schema::SimpleTcpPortScanResult,
        attack_results::schema::FullQueryCertificateTransparencyResult,
        attack_results::schema::SimpleQueryUnhashedResult,
        attack_results::schema::SimpleHostAliveResult,
        attack_results::schema::FullServiceDetectionResult,
        attack_results::schema::FullUdpServiceDetectionResult,
        attack_results::schema::SimpleDnsResolutionResult,
        attack_results::schema::FullTestSSLResult,
        attack_results::schema::TestSSLFinding,
        dehashed_rs::Query,
        dehashed_rs::SearchType,
        attacks::schema::QueryDehashedRequest,
        models::AttackType,
        oauth_applications::schema::CreateAppRequest,
        oauth_applications::schema::SimpleOauthClient,
        oauth_applications::schema::FullOauthClient,
        oauth_applications::schema::ListOauthApplications,
        oauth_applications::schema::UpdateAppRequest,
        oauth::schema::OpenRequestInfo,
        oauth_decisions::schema::ListOauthDecisions,
        oauth_decisions::schema::FullOauthDecision,
        settings::schema::SettingsFull,
        settings::schema::UpdateSettingsRequest,
        api_keys::schema::FullApiKey,
        api_keys::schema::CreateApiKeyRequest,
        api_keys::schema::ListApiKeys,
        api_keys::schema::UpdateApiKeyRequest,
        hosts::schema::SimpleHost,
        hosts::schema::FullHost,
        hosts::schema::UpdateHostRequest,
        hosts::schema::CreateHostRequest,
        hosts::schema::GetAllHostsQuery,
        hosts::schema::HostRelations,
        ports::schema::SimplePort,
        ports::schema::FullPort,
        ports::schema::UpdatePortRequest,
        ports::schema::CreatePortRequest,
        ports::schema::GetAllPortsQuery,
        ports::schema::PortRelations,
        services::schema::SimpleService,
        services::schema::FullService,
        services::schema::UpdateServiceRequest,
        services::schema::CreateServiceRequest,
        services::schema::GetAllServicesQuery,
        services::schema::ServiceRelations,
        domains::schema::SimpleDomain,
        domains::schema::FullDomain,
        domains::schema::UpdateDomainRequest,
        domains::schema::GetAllDomainsQuery,
        domains::schema::CreateDomainRequest,
        domains::schema::GetAllDomainsQuery,
        domains::schema::DomainRelations,
        common::schema::HostResultsPage,
        common::schema::DomainResultsPage,
        common::schema::PortResultsPage,
        common::schema::ServiceResultsPage,
        models::OsType,
        models::Color,
        models::DomainCertainty,
        models::HostCertainty,
        models::PortCertainty,
        models::ServiceCertainty,
        models::UserPermission,
        models::ManualHostCertainty,
        models::ManualPortCertainty,
        models::ManualServiceCertainty,
        models::TestSSLSection,
        models::TestSSLSeverity,
        global_tags::schema::CreateGlobalTagRequest,
        global_tags::schema::FullGlobalTag,
        global_tags::schema::ListGlobalTags,
        global_tags::schema::UpdateGlobalTag,
        models::PortProtocol,
        workspace_tags::schema::FullWorkspaceTag,
        workspace_tags::schema::ListWorkspaceTags,
        workspace_tags::schema::UpdateWorkspaceTag,
        workspace_tags::schema::CreateWorkspaceTagRequest,
        wordlists::schema::ListWordlists,
        wordlists::schema::SimpleWordlist,
        wordlists::schema::CreateWordlistRequest,
        wordlists::schema::ListWordlistsAdmin,
        wordlists::schema::FullWordlist,
        wordlists::schema::UpdateWordlistRequest,
        workspace_invitations::schema::FullWorkspaceInvitation,
        workspace_invitations::schema::WorkspaceInvitationList,
        chan::ws_manager::schema::WsMessage,
        chan::ws_manager::schema::AggregationType,
        chan::ws_manager::schema::CertificateTransparencyEntry,
    )),
    modifiers(&SecurityAddon)
)]
pub(crate) struct FrontendApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        oauth::handler::auth,
        oauth::handler::token,
        data_export::handler::export_workspace,
    ),
    components(schemas(
        models::OsType,
        models::PortProtocol,
        models::DomainCertainty,
        models::HostCertainty,
        models::PortCertainty,
        models::ServiceCertainty,
        common::schema::ApiErrorResponse,
        common::schema::ApiStatusCode,
        oauth_schemas::TokenRequest,
        oauth_schemas::TokenResponse,
        oauth_schemas::TokenError,
        oauth_schemas::TokenErrorType,
        data_export::schema::AggregatedWorkspace,
        data_export::schema::AggregatedHost,
        data_export::schema::AggregatedPort,
        data_export::schema::AggregatedService,
        data_export::schema::AggregatedDomain,
        data_export::schema::AggregatedTags,
        data_export::schema::AggregatedRelation,
    )),
    modifiers(&SecurityAddon2)
)]
pub(crate) struct ExternalApi;
