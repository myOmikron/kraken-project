use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{Arc, RwLock};

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
use dehashed_rs::Scheduler;
use rorm::Database;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use webauthn_rs::prelude::{Url, WebauthnError};
use webauthn_rs::WebauthnBuilder;

use crate::api::handler::{
    apikeys, bruteforce_subdomains, create_leech, create_user, create_workspace, delete_attack,
    delete_leech, delete_user, delete_workspace, finish_auth, finish_register, get_all_leeches,
    get_all_users, get_all_workspaces, get_all_workspaces_admin, get_attack, get_leech, get_me,
    get_settings, get_tcp_port_scan_results, get_user, get_workspace, get_workspace_admin, login,
    logout, oauth, query_certificate_transparency, report_workspace_results, scan_tcp_ports,
    set_password, start_auth, start_register, test, update_leech, update_me, update_settings,
    update_workspace, websocket,
};
use crate::api::middleware::{
    handle_not_found, json_extractor_error, AdminRequired, AuthenticationRequired, TokenRequired,
};
use crate::api::swagger::ApiDoc;
use crate::chan::{RpcClients, RpcManagerChannel, SettingsManagerChan, WsManagerChan};
use crate::config::Config;

const ORIGIN_NAME: &str = "Kraken";

/// A type alias for the scheduler of the dehashed api
///
/// It consists of an rwlock with an option that is either None, if no scheduler is
/// available (due to missing credentials) or the scheduler.
pub type DehashedScheduler = Data<RwLock<Option<Scheduler>>>;

pub(crate) async fn start_server(
    db: Database,
    config: &Config,
    rpc_manager_chan: RpcManagerChannel,
    rpc_clients: RpcClients,
    ws_manager_chan: WsManagerChan,
    setting_manager_chan: Arc<SettingsManagerChan>,
    dehashed_scheduler: Option<Scheduler>,
) -> Result<(), StartServerError> {
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

    let oauth = Data::new(oauth::OauthManager::default());

    let reporting_key = config.server.reporting_key.clone();
    let dehashed = Data::new(RwLock::new(dehashed_scheduler));

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .app_data(JsonConfig::default().error_handler(json_extractor_error))
            .app_data(PayloadConfig::default())
            .app_data(webauthn.clone())
            .app_data(oauth.clone())
            .app_data(Data::new(ws_manager_chan.clone()))
            .app_data(Data::new(rpc_manager_chan.clone()))
            .app_data(rpc_clients.clone())
            .app_data(Data::new(setting_manager_chan.clone()))
            .app_data(dehashed.clone())
            .wrap(setup_logging_mw(LoggingMiddlewareConfig::default()))
            .wrap(
                SessionMiddleware::builder(DBSessionStore::new(db.clone()), key.clone())
                    .session_lifecycle(PersistentSession::session_ttl(
                        PersistentSession::default(),
                        Duration::hours(1),
                    ))
                    .build(),
            )
            .wrap(Compress::default())
            .wrap(ErrorHandlers::new().handler(StatusCode::NOT_FOUND, handle_not_found))
            .service(SwaggerUi::new("/docs/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()))
            .service(
                scope("/api/v1/reporting")
                    .wrap(TokenRequired(reporting_key.clone()))
                    .service(report_workspace_results),
            )
            .service(
                scope("/api/v1/auth")
                    .service(test)
                    .service(login)
                    .service(logout)
                    .service(start_register)
                    .service(finish_register)
                    .service(start_auth)
                    .service(finish_auth),
            )
            .service(
                scope("/api/v1/oauth")
                    .service(oauth::info)
                    .service(oauth::auth)
                    .service(oauth::accept)
                    .service(oauth::deny),
            )
            .service(scope("/api/v1/oauth-server").service(oauth::token))
            .service(
                scope("/api/v1/admin")
                    .wrap(AdminRequired)
                    .service(get_leech)
                    .service(get_all_leeches)
                    .service(create_leech)
                    .service(delete_leech)
                    .service(update_leech)
                    .service(create_user)
                    .service(delete_user)
                    .service(get_user)
                    .service(get_all_users)
                    .service(get_workspace_admin)
                    .service(get_all_workspaces_admin)
                    .service(oauth::create_oauth_app)
                    .service(oauth::get_all_oauth_apps)
                    .service(oauth::get_oauth_app)
                    .service(oauth::update_oauth_app)
                    .service(oauth::delete_oauth_app)
                    .service(get_settings)
                    .service(update_settings),
            )
            .service(
                scope("/api/v1")
                    .wrap(AuthenticationRequired)
                    .service(websocket)
                    .service(get_me)
                    .service(update_me)
                    .service(set_password)
                    .service(get_workspace)
                    .service(get_all_workspaces)
                    .service(create_workspace)
                    .service(delete_workspace)
                    .service(update_workspace)
                    .service(bruteforce_subdomains)
                    .service(scan_tcp_ports)
                    .service(query_certificate_transparency)
                    .service(delete_attack)
                    .service(get_tcp_port_scan_results)
                    .service(get_attack)
                    .service(apikeys::create_api_key)
                    .service(apikeys::get_api_keys)
                    .service(apikeys::update_leech)
                    .service(apikeys::delete_api_key),
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
