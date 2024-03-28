//! [`start_server`] which build and runs the actix server

use std::io;

use actix_toolbox::tb_middleware::actix_session::config::TtlExtensionPolicy;
use actix_toolbox::tb_middleware::setup_logging_mw;
use actix_toolbox::tb_middleware::DBSessionStore;
use actix_toolbox::tb_middleware::LoggingMiddlewareConfig;
use actix_toolbox::tb_middleware::PersistentSession;
use actix_toolbox::tb_middleware::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::cookie::KeyError;
use actix_web::http::StatusCode;
use actix_web::middleware::Compress;
use actix_web::middleware::ErrorHandlers;
use actix_web::web::Data;
use actix_web::web::JsonConfig;
use actix_web::web::PayloadConfig;
use actix_web::App;
use actix_web::HttpServer;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use swaggapi::ApiContext;
use swaggapi::SwaggapiPage;
use swaggapi::SwaggerUi;
use thiserror::Error;
use webauthn_rs::prelude::Url;
use webauthn_rs::prelude::WebauthnError;
use webauthn_rs::WebauthnBuilder;

use crate::api::handler::api_keys;
use crate::api::handler::attack_results;
use crate::api::handler::attacks;
use crate::api::handler::auth;
use crate::api::handler::data_export;
use crate::api::handler::domains;
use crate::api::handler::files;
use crate::api::handler::finding_affected;
use crate::api::handler::finding_definitions;
use crate::api::handler::findings;
use crate::api::handler::global_tags;
use crate::api::handler::hosts;
use crate::api::handler::leeches;
use crate::api::handler::oauth;
use crate::api::handler::oauth_applications;
use crate::api::handler::ports;
use crate::api::handler::services;
use crate::api::handler::settings;
use crate::api::handler::users;
use crate::api::handler::websocket;
use crate::api::handler::wordlists;
use crate::api::handler::workspace_invitations;
use crate::api::handler::workspace_tags;
use crate::api::handler::workspaces;
use crate::api::middleware::handle_not_found;
use crate::api::middleware::json_extractor_error;
use crate::api::middleware::AdminRequired;
use crate::api::middleware::AuthenticationRequired;
use crate::chan::global::GLOBAL;
use crate::config::Config;
use crate::modules::oauth::OauthManager;

const ORIGIN_NAME: &str = "Kraken";

#[derive(SwaggapiPage)]
struct ExternalApi;
#[derive(SwaggapiPage)]
struct FrontendApi;

/// Start the web server
pub async fn start_server(config: &Config) -> Result<(), StartServerError> {
    let key = Key::try_from(
        BASE64_STANDARD
            .decode(&config.server.secret_key)?
            .as_slice(),
    )?;

    let rp_origin =
        Url::parse(&config.server.origin_uri).map_err(|_| StartServerError::InvalidOrigin)?;

    let webauthn = Data::new(
        WebauthnBuilder::new(
            rp_origin.domain().ok_or(StartServerError::InvalidOrigin)?,
            &rp_origin,
        )?
        .rp_name(ORIGIN_NAME)
        .build()?,
    );

    let oauth = Data::new(OauthManager::default());

    HttpServer::new(move || {
        App::new()
            .app_data(JsonConfig::default().error_handler(json_extractor_error))
            .app_data(PayloadConfig::default())
            .app_data(webauthn.clone())
            .app_data(oauth.clone())
            .wrap(setup_logging_mw(LoggingMiddlewareConfig::default()))
            .wrap(
                SessionMiddleware::builder(DBSessionStore::new(GLOBAL.db.clone()), key.clone())
                    .session_lifecycle(
                        PersistentSession::session_ttl(
                            PersistentSession::default(),
                            Duration::hours(1),
                        )
                        .session_ttl_extension_policy(TtlExtensionPolicy::OnEveryRequest),
                    )
                    .build(),
            )
            .wrap(Compress::default())
            .wrap(ErrorHandlers::new().handler(StatusCode::NOT_FOUND, handle_not_found))
            .service({
                // TODO prettier swaggapi api
                let mut s = SwaggerUi::without_everything();
                s.path = "/docs";
                s.page("Frontend API", FrontendApi)
                    .page("External API", ExternalApi)
            })
            .service(
                ApiContext::new("/api/v1/auth")
                    .page(FrontendApi)
                    .handler(auth::handler::test)
                    .handler(auth::handler::login)
                    .handler(auth::handler::logout)
                    .handler(auth::handler::start_register)
                    .handler(auth::handler::finish_register)
                    .handler(auth::handler::start_auth)
                    .handler(auth::handler::finish_auth),
            )
            .service(
                ApiContext::new("/api/v1/oauth")
                    .page(FrontendApi)
                    .handler(oauth::handler::info)
                    .handler(oauth::handler::accept)
                    .handler(oauth::handler::deny),
            )
            .service(
                ApiContext::new("/api/v1/oauth")
                    .page(ExternalApi)
                    .handler(oauth::handler::auth),
            )
            .service(
                ApiContext::new("/api/v1/oauth-server")
                    .page(ExternalApi)
                    .handler(oauth::handler::token),
            )
            .service(
                ApiContext::new("/api/v1/export")
                    .page(ExternalApi)
                    .handler(data_export::handler::export_workspace),
            )
            .service(
                ApiContext::new("/api/v1/admin")
                    .page(FrontendApi)
                    .wrap(AdminRequired)
                    .handler(leeches::handler_admin::get_leech)
                    .handler(leeches::handler_admin::get_all_leeches)
                    .handler(leeches::handler_admin::create_leech)
                    .handler(leeches::handler_admin::delete_leech)
                    .handler(leeches::handler_admin::update_leech)
                    .handler(leeches::handler_admin::gen_leech_config)
                    .handler(users::handler_admin::create_user)
                    .handler(users::handler_admin::delete_user)
                    .handler(users::handler_admin::get_user)
                    .handler(users::handler_admin::get_all_users_admin)
                    .handler(workspaces::handler_admin::get_workspace_admin)
                    .handler(workspaces::handler_admin::get_all_workspaces_admin)
                    .handler(files::handler_admin::get_all_files_admin)
                    .handler(files::handler_admin::download_file_admin)
                    .handler(files::handler_admin::delete_file_admin)
                    .handler(oauth_applications::handler_admin::create_oauth_app)
                    .handler(oauth_applications::handler_admin::get_all_oauth_apps)
                    .handler(oauth_applications::handler_admin::get_oauth_app)
                    .handler(oauth_applications::handler_admin::update_oauth_app)
                    .handler(oauth_applications::handler_admin::delete_oauth_app)
                    .handler(settings::handler_admin::get_settings)
                    .handler(settings::handler_admin::update_settings)
                    .handler(global_tags::handler_admin::create_global_tag)
                    .handler(global_tags::handler_admin::update_global_tag)
                    .handler(global_tags::handler_admin::delete_global_tag)
                    .handler(wordlists::handler_admin::create_wordlist_admin)
                    .handler(wordlists::handler_admin::get_all_wordlists_admin)
                    .handler(wordlists::handler_admin::update_wordlist_admin)
                    .handler(wordlists::handler_admin::delete_wordlist_admin)
                    .handler(finding_definitions::handler_admin::get_finding_definition_usage)
                    .handler(finding_definitions::handler_admin::delete_finding_definition),
            )
            .service(
                ApiContext::new("/api/v1")
                    .page(FrontendApi)
                    .wrap(AuthenticationRequired)
                    .handler(websocket::websocket)
                    .handler(users::handler::get_me)
                    .handler(users::handler::update_me)
                    .handler(users::handler::set_password)
                    .handler(users::handler::get_all_users)
                    .handler(workspaces::handler::get_workspace)
                    .handler(workspaces::handler::get_all_workspaces)
                    .handler(workspaces::handler::create_workspace)
                    .handler(workspaces::handler::delete_workspace)
                    .handler(workspaces::handler::update_workspace)
                    .handler(workspaces::handler::transfer_ownership)
                    .handler(workspaces::handler::create_invitation)
                    .handler(workspaces::handler::retract_invitation)
                    .handler(workspaces::handler::get_all_workspace_invitations)
                    .handler(workspaces::handler::search)
                    .handler(workspaces::handler::get_search_results)
                    .handler(workspaces::handler::get_searches)
                    .handler(workspaces::handler::archive_workspace)
                    .handler(workspaces::handler::unarchive_workspace)
                    .handler(files::handler::upload_file)
                    .handler(files::handler::upload_image)
                    .handler(files::handler::download_thumbnail)
                    .handler(files::handler::download_file)
                    .handler(attacks::handler::bruteforce_subdomains)
                    .handler(attacks::handler::query_certificate_transparency)
                    .handler(attacks::handler::delete_attack)
                    .handler(attacks::handler::get_attack)
                    .handler(attacks::handler::get_workspace_attacks)
                    .handler(attacks::handler::get_all_attacks)
                    .handler(attacks::handler::hosts_alive_check)
                    .handler(attacks::handler::query_dehashed)
                    .handler(attacks::handler::service_detection)
                    .handler(attacks::handler::udp_service_detection)
                    .handler(attacks::handler::dns_resolution)
                    .handler(attacks::handler::dns_txt_scan)
                    .handler(attacks::handler::os_detection)
                    .handler(attack_results::handler::get_bruteforce_subdomains_results)
                    .handler(attack_results::handler::get_query_certificate_transparency_results)
                    .handler(attack_results::handler::get_query_unhashed_results)
                    .handler(attack_results::handler::get_host_alive_results)
                    .handler(attack_results::handler::get_service_detection_results)
                    .handler(attack_results::handler::get_udp_service_detection_results)
                    .handler(attack_results::handler::get_dns_resolution_results)
                    .handler(attack_results::handler::get_dns_txt_scan_results)
                    .handler(attack_results::handler::get_os_detection_results)
                    .handler(api_keys::handler::create_api_key)
                    .handler(api_keys::handler::get_api_keys)
                    .handler(api_keys::handler::update_api_key)
                    .handler(api_keys::handler::delete_api_key)
                    .handler(global_tags::handler::get_all_global_tags)
                    .handler(workspace_tags::handler::create_workspace_tag)
                    .handler(workspace_tags::handler::get_all_workspace_tags)
                    .handler(workspace_tags::handler::update_workspace_tag)
                    .handler(workspace_tags::handler::delete_workspace_tag)
                    .handler(hosts::handler::get_all_hosts)
                    .handler(hosts::handler::get_host)
                    .handler(hosts::handler::create_host)
                    .handler(hosts::handler::update_host)
                    .handler(hosts::handler::delete_host)
                    .handler(hosts::handler::get_host_sources)
                    .handler(hosts::handler::get_host_relations)
                    .handler(hosts::handler::get_host_findings)
                    .handler(ports::handler::get_all_ports)
                    .handler(ports::handler::get_port)
                    .handler(ports::handler::create_port)
                    .handler(ports::handler::update_port)
                    .handler(ports::handler::delete_port)
                    .handler(ports::handler::get_port_sources)
                    .handler(ports::handler::get_port_relations)
                    .handler(ports::handler::get_port_findings)
                    .handler(services::handler::get_all_services)
                    .handler(services::handler::get_service)
                    .handler(services::handler::create_service)
                    .handler(services::handler::update_service)
                    .handler(services::handler::delete_service)
                    .handler(services::handler::get_service_sources)
                    .handler(services::handler::get_service_relations)
                    .handler(services::handler::get_service_findings)
                    .handler(domains::handler::get_all_domains)
                    .handler(domains::handler::get_domain)
                    .handler(domains::handler::create_domain)
                    .handler(domains::handler::update_domain)
                    .handler(domains::handler::delete_domain)
                    .handler(domains::handler::get_domain_sources)
                    .handler(domains::handler::get_domain_relations)
                    .handler(domains::handler::get_domain_findings)
                    .handler(wordlists::handler::get_all_wordlists)
                    .handler(workspace_invitations::handler::get_all_invitations)
                    .handler(workspace_invitations::handler::accept_invitation)
                    .handler(workspace_invitations::handler::decline_invitation)
                    .handler(findings::handler::create_finding)
                    .handler(findings::handler::get_all_findings)
                    .handler(findings::handler::get_finding)
                    .handler(findings::handler::update_finding)
                    .handler(findings::handler::delete_finding)
                    .handler(finding_affected::handler::create_finding_affected)
                    .handler(finding_affected::handler::get_finding_affected)
                    .handler(finding_affected::handler::update_finding_affected)
                    .handler(finding_affected::handler::delete_finding_affected)
                    .handler(finding_definitions::handler::create_finding_definition)
                    .handler(finding_definitions::handler::get_finding_definition)
                    .handler(finding_definitions::handler::get_all_finding_definitions)
                    .handler(finding_definitions::handler::update_finding_definition),
            )
    })
    .bind((
        config.server.api_listen_address.as_str(),
        config.server.api_listen_port,
    ))?
    .run()
    .await?;

    Ok(())
}

/// Error type produced by [`start_server`]
#[derive(Debug, Error)]
pub enum StartServerError {
    /// An [`io::Error`]
    #[error("Error starting server: {0}")]
    IO(#[from] io::Error),

    /// A [`WebauthnError`]
    #[error("Error while constructing Webauthn: {0}")]
    Webauthn(#[from] WebauthnError),

    /// Config contains an invalid secret key
    #[error("Invalid parameter SecretKey.\nConsider using the subcommand keygen and update your configuration file")]
    #[from(base64::DecodeError)]
    InvalidSecretKey,

    /// Config contains an invalid origin
    #[error("invalid origin specified")]
    InvalidOrigin,
}

impl From<base64::DecodeError> for StartServerError {
    fn from(_value: base64::DecodeError) -> Self {
        Self::InvalidSecretKey
    }
}

impl From<KeyError> for StartServerError {
    fn from(_value: KeyError) -> Self {
        Self::InvalidSecretKey
    }
}
