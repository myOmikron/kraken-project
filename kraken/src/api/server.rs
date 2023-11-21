use std::fmt::{Display, Formatter};
use std::io;

use actix_toolbox::tb_middleware::{
    setup_logging_mw, DBSessionStore, LoggingMiddlewareConfig, PersistentSession, SessionMiddleware,
};
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Key, KeyError};
use actix_web::http::StatusCode;
use actix_web::middleware::{Compress, ErrorHandlers};
use actix_web::web::{scope, Data, JsonConfig, PayloadConfig};
use actix_web::{App, HttpServer};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use webauthn_rs::prelude::{Url, WebauthnError};
use webauthn_rs::WebauthnBuilder;

use crate::api::handler::{
    api_keys, attack_results, attacks, auth, data_export, domains, global_tags, hosts, leeches,
    oauth, oauth_applications, ports, services, settings, users, websocket, wordlists,
    workspace_invitations, workspace_tags, workspaces,
};
use crate::api::middleware::{
    handle_not_found, json_extractor_error, AdminRequired, AuthenticationRequired,
};
use crate::api::swagger::{ExternalApi, FrontendApi};
use crate::chan::GLOBAL;
use crate::config::Config;
use crate::modules::oauth::OauthManager;

const ORIGIN_NAME: &str = "Kraken";

pub(crate) async fn start_server(config: &Config) -> Result<(), StartServerError> {
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
        )
        .unwrap()
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
                    .session_lifecycle(PersistentSession::session_ttl(
                        PersistentSession::default(),
                        Duration::hours(1),
                    ))
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
                    .service(auth::test)
                    .service(auth::login)
                    .service(auth::logout)
                    .service(auth::start_register)
                    .service(auth::finish_register)
                    .service(auth::start_auth)
                    .service(auth::finish_auth),
            )
            .service(
                scope("/api/v1/oauth")
                    .service(oauth::info)
                    .service(oauth::auth)
                    .service(oauth::accept)
                    .service(oauth::deny),
            )
            .service(scope("/api/v1/oauth-server").service(oauth::token))
            .service(scope("/api/v1/export").service(data_export::export_workspace))
            .service(
                scope("/api/v1/admin")
                    .wrap(AdminRequired)
                    .service(leeches::get_leech)
                    .service(leeches::get_all_leeches)
                    .service(leeches::create_leech)
                    .service(leeches::delete_leech)
                    .service(leeches::update_leech)
                    .service(leeches::gen_leech_config)
                    .service(users::create_user)
                    .service(users::delete_user)
                    .service(users::get_user)
                    .service(users::get_all_users_admin)
                    .service(workspaces::get_workspace_admin)
                    .service(workspaces::get_all_workspaces_admin)
                    .service(oauth_applications::create_oauth_app)
                    .service(oauth_applications::get_all_oauth_apps)
                    .service(oauth_applications::get_oauth_app)
                    .service(oauth_applications::update_oauth_app)
                    .service(oauth_applications::delete_oauth_app)
                    .service(settings::get_settings)
                    .service(settings::update_settings)
                    .service(global_tags::create_global_tag)
                    .service(global_tags::update_global_tag)
                    .service(global_tags::delete_global_tag)
                    .service(wordlists::create_wordlist_admin)
                    .service(wordlists::get_all_wordlists_admin)
                    .service(wordlists::update_wordlist_admin)
                    .service(wordlists::delete_wordlist_admin),
            )
            .service(
                scope("/api/v1")
                    .wrap(AuthenticationRequired)
                    .service(websocket::websocket)
                    .service(users::get_me)
                    .service(users::update_me)
                    .service(users::set_password)
                    .service(users::get_all_users)
                    .service(workspaces::get_workspace)
                    .service(workspaces::get_all_workspaces)
                    .service(workspaces::create_workspace)
                    .service(workspaces::delete_workspace)
                    .service(workspaces::update_workspace)
                    .service(workspaces::transfer_ownership)
                    .service(workspaces::create_invitation)
                    .service(workspaces::retract_invitation)
                    .service(workspaces::get_all_workspace_invitations)
                    .service(workspaces::search)
                    .service(workspaces::get_search_results)
                    .service(workspaces::get_searches)
                    .service(attacks::bruteforce_subdomains)
                    .service(attacks::scan_tcp_ports)
                    .service(attacks::query_certificate_transparency)
                    .service(attacks::delete_attack)
                    .service(attacks::get_attack)
                    .service(attacks::get_workspace_attacks)
                    .service(attacks::hosts_alive_check)
                    .service(attacks::query_dehashed)
                    .service(attacks::service_detection)
                    .service(attacks::dns_resolution)
                    .service(attack_results::get_bruteforce_subdomains_results)
                    .service(attack_results::get_tcp_port_scan_results)
                    .service(attack_results::get_query_certificate_transparency_results)
                    .service(attack_results::get_query_unhashed_results)
                    .service(attack_results::get_host_alive_results)
                    .service(attack_results::get_service_detection_results)
                    .service(attack_results::get_dns_resolution_results)
                    .service(api_keys::create_api_key)
                    .service(api_keys::get_api_keys)
                    .service(api_keys::update_api_key)
                    .service(api_keys::delete_api_key)
                    .service(global_tags::get_all_global_tags)
                    .service(workspace_tags::create_workspace_tag)
                    .service(workspace_tags::get_all_workspace_tags)
                    .service(workspace_tags::update_workspace_tag)
                    .service(workspace_tags::delete_workspace_tag)
                    .service(hosts::get_all_hosts)
                    .service(hosts::get_host)
                    .service(hosts::create_host)
                    .service(hosts::update_host)
                    .service(ports::get_all_ports)
                    .service(ports::get_port)
                    .service(ports::create_port)
                    .service(ports::update_port)
                    .service(services::get_all_services)
                    .service(services::get_service)
                    .service(services::create_service)
                    .service(services::update_service)
                    .service(domains::get_all_domains)
                    .service(domains::get_domain)
                    .service(domains::create_domain)
                    .service(domains::update_domain)
                    .service(wordlists::get_all_wordlists)
                    .service(workspace_invitations::get_all_invitations)
                    .service(workspace_invitations::accept_invitation)
                    .service(workspace_invitations::decline_invitation),
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

#[derive(Debug)]
pub(crate) enum StartServerError {
    IO(io::Error),
    Webauthn(WebauthnError),
    InvalidSecretKey,
    InvalidOrigin,
}

impl Display for StartServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StartServerError::IO(err) => write!(f, "Error starting server: {err}"),
            StartServerError::Webauthn(err) => {
                write!(f, "Error while constructing Webauthn: {err}")
            }
            StartServerError::InvalidSecretKey => write!(
                f,
                "Invalid parameter SecretKey.\
                    Consider using the subcommand keygen and update your configuration file"
            ),
            StartServerError::InvalidOrigin => write!(f, "invalid origin specified"),
        }
    }
}

impl From<io::Error> for StartServerError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<base64::DecodeError> for StartServerError {
    fn from(_value: base64::DecodeError) -> Self {
        Self::InvalidSecretKey
    }
}

impl From<WebauthnError> for StartServerError {
    fn from(value: WebauthnError) -> Self {
        Self::Webauthn(value)
    }
}

impl From<KeyError> for StartServerError {
    fn from(_value: KeyError) -> Self {
        Self::InvalidSecretKey
    }
}

impl From<StartServerError> for String {
    fn from(value: StartServerError) -> Self {
        value.to_string()
    }
}
