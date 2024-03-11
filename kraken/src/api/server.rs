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
use actix_web::web::scope;
use actix_web::web::Data;
use actix_web::web::JsonConfig;
use actix_web::web::PayloadConfig;
use actix_web::App;
use actix_web::HttpServer;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use thiserror::Error;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
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
use crate::api::handler::finding_definitions;
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
use crate::api::swagger::ExternalApi;
use crate::api::swagger::FrontendApi;
use crate::chan::global::GLOBAL;
use crate::config::Config;
use crate::modules::oauth::OauthManager;

const ORIGIN_NAME: &str = "Kraken";

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
            .service(SwaggerUi::new("/docs/{_:.*}").urls(vec![
                (
                    utoipa_swagger_ui::Url::new("frontend-api", "/api-doc/frontend-api.json"),
                    FrontendApi::openapi(),
                ),
                (
                    utoipa_swagger_ui::Url::new("external-api", "/api-doc/external-api.json"),
                    ExternalApi::openapi(),
                ),
            ]))
            .service(
                scope("/api/v1/auth")
                    .service(auth::handler::test)
                    .service(auth::handler::login)
                    .service(auth::handler::logout)
                    .service(auth::handler::start_register)
                    .service(auth::handler::finish_register)
                    .service(auth::handler::start_auth)
                    .service(auth::handler::finish_auth),
            )
            .service(
                scope("/api/v1/oauth")
                    .service(oauth::handler::info)
                    .service(oauth::handler::auth)
                    .service(oauth::handler::accept)
                    .service(oauth::handler::deny),
            )
            .service(scope("/api/v1/oauth-server").service(oauth::handler::token))
            .service(scope("/api/v1/export").service(data_export::handler::export_workspace))
            .service(
                scope("/api/v1/admin")
                    .wrap(AdminRequired)
                    .service(leeches::handler_admin::get_leech)
                    .service(leeches::handler_admin::get_all_leeches)
                    .service(leeches::handler_admin::create_leech)
                    .service(leeches::handler_admin::delete_leech)
                    .service(leeches::handler_admin::update_leech)
                    .service(leeches::handler_admin::gen_leech_config)
                    .service(users::handler_admin::create_user)
                    .service(users::handler_admin::delete_user)
                    .service(users::handler_admin::get_user)
                    .service(users::handler_admin::get_all_users_admin)
                    .service(workspaces::handler_admin::get_workspace_admin)
                    .service(workspaces::handler_admin::get_all_workspaces_admin)
                    .service(files::handler_admin::get_all_files_admin)
                    .service(files::handler_admin::download_file_admin)
                    .service(files::handler_admin::delete_file_admin)
                    .service(oauth_applications::handler_admin::create_oauth_app)
                    .service(oauth_applications::handler_admin::get_all_oauth_apps)
                    .service(oauth_applications::handler_admin::get_oauth_app)
                    .service(oauth_applications::handler_admin::update_oauth_app)
                    .service(oauth_applications::handler_admin::delete_oauth_app)
                    .service(settings::handler_admin::get_settings)
                    .service(settings::handler_admin::update_settings)
                    .service(global_tags::handler_admin::create_global_tag)
                    .service(global_tags::handler_admin::update_global_tag)
                    .service(global_tags::handler_admin::delete_global_tag)
                    .service(wordlists::handler_admin::create_wordlist_admin)
                    .service(wordlists::handler_admin::get_all_wordlists_admin)
                    .service(wordlists::handler_admin::update_wordlist_admin)
                    .service(wordlists::handler_admin::delete_wordlist_admin)
                    .service(finding_definitions::handler_admin::delete_finding_definition),
            )
            .service(
                scope("/api/v1")
                    .wrap(AuthenticationRequired)
                    .service(websocket::websocket)
                    .service(users::handler::get_me)
                    .service(users::handler::update_me)
                    .service(users::handler::set_password)
                    .service(users::handler::get_all_users)
                    .service(workspaces::handler::get_workspace)
                    .service(workspaces::handler::get_all_workspaces)
                    .service(workspaces::handler::create_workspace)
                    .service(workspaces::handler::delete_workspace)
                    .service(workspaces::handler::update_workspace)
                    .service(workspaces::handler::transfer_ownership)
                    .service(workspaces::handler::create_invitation)
                    .service(workspaces::handler::retract_invitation)
                    .service(workspaces::handler::get_all_workspace_invitations)
                    .service(workspaces::handler::search)
                    .service(workspaces::handler::get_search_results)
                    .service(workspaces::handler::get_searches)
                    .service(files::handler::upload_file)
                    .service(files::handler::upload_image)
                    .service(files::handler::download_thumbnail)
                    .service(files::handler::download_file)
                    .service(attacks::handler::bruteforce_subdomains)
                    .service(attacks::handler::query_certificate_transparency)
                    .service(attacks::handler::delete_attack)
                    .service(attacks::handler::get_attack)
                    .service(attacks::handler::get_workspace_attacks)
                    .service(attacks::handler::get_all_attacks)
                    .service(attacks::handler::hosts_alive_check)
                    .service(attacks::handler::query_dehashed)
                    .service(attacks::handler::service_detection)
                    .service(attacks::handler::udp_service_detection)
                    .service(attacks::handler::dns_resolution)
                    .service(attacks::handler::dns_txt_scan)
                    .service(attacks::handler::os_detection)
                    .service(attack_results::handler::get_bruteforce_subdomains_results)
                    .service(attack_results::handler::get_query_certificate_transparency_results)
                    .service(attack_results::handler::get_query_unhashed_results)
                    .service(attack_results::handler::get_host_alive_results)
                    .service(attack_results::handler::get_service_detection_results)
                    .service(attack_results::handler::get_udp_service_detection_results)
                    .service(attack_results::handler::get_dns_resolution_results)
                    .service(attack_results::handler::get_dns_txt_scan_results)
                    .service(attack_results::handler::get_os_detection_results)
                    .service(api_keys::handler::create_api_key)
                    .service(api_keys::handler::get_api_keys)
                    .service(api_keys::handler::update_api_key)
                    .service(api_keys::handler::delete_api_key)
                    .service(global_tags::handler::get_all_global_tags)
                    .service(workspace_tags::handler::create_workspace_tag)
                    .service(workspace_tags::handler::get_all_workspace_tags)
                    .service(workspace_tags::handler::update_workspace_tag)
                    .service(workspace_tags::handler::delete_workspace_tag)
                    .service(hosts::handler::get_all_hosts)
                    .service(hosts::handler::get_host)
                    .service(hosts::handler::create_host)
                    .service(hosts::handler::update_host)
                    .service(hosts::handler::delete_host)
                    .service(hosts::handler::get_host_sources)
                    .service(hosts::handler::get_host_relations)
                    .service(ports::handler::get_all_ports)
                    .service(ports::handler::get_port)
                    .service(ports::handler::create_port)
                    .service(ports::handler::update_port)
                    .service(ports::handler::delete_port)
                    .service(ports::handler::get_port_sources)
                    .service(ports::handler::get_port_relations)
                    .service(services::handler::get_all_services)
                    .service(services::handler::get_service)
                    .service(services::handler::create_service)
                    .service(services::handler::update_service)
                    .service(services::handler::delete_service)
                    .service(services::handler::get_service_sources)
                    .service(services::handler::get_service_relations)
                    .service(domains::handler::get_all_domains)
                    .service(domains::handler::get_domain)
                    .service(domains::handler::create_domain)
                    .service(domains::handler::update_domain)
                    .service(domains::handler::delete_domain)
                    .service(domains::handler::get_domain_sources)
                    .service(domains::handler::get_domain_relations)
                    .service(wordlists::handler::get_all_wordlists)
                    .service(workspace_invitations::handler::get_all_invitations)
                    .service(workspace_invitations::handler::accept_invitation)
                    .service(workspace_invitations::handler::decline_invitation)
                    .service(finding_definitions::handler::create_finding_definition)
                    .service(finding_definitions::handler::get_finding_definition)
                    .service(finding_definitions::handler::get_all_finding_definitions),
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
