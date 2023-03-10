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
use rorm::Database;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use webauthn_rs::prelude::{Url, WebauthnError};
use webauthn_rs::WebauthnBuilder;

use crate::api::handler::{
    bruteforce_subdomains, create_leech, create_user, create_workspace, delete_leech, delete_user,
    delete_workspace, finish_auth, finish_register, get_all_leeches, get_all_users,
    get_all_workspaces, get_all_workspaces_admin, get_leech, get_me, get_user, get_workspace,
    get_workspace_admin, login, logout, query_certificate_transparency, scan_tcp_ports,
    set_password, start_auth, start_register, test, update_leech, update_me, update_workspace,
    websocket,
};
use crate::api::middleware::{
    handle_not_found, json_extractor_error, AdminRequired, AuthenticationRequired,
};
use crate::api::swagger::ApiDoc;
use crate::chan::{RpcClients, RpcManagerChannel, WsManagerChan};
use crate::config::Config;

const ORIGIN_NAME: &str = "Kraken";

pub(crate) async fn start_server(
    db: Database,
    config: &Config,
    rpc_manager_chan: RpcManagerChannel,
    rpc_clients: RpcClients,
    ws_manager_chan: WsManagerChan,
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

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .app_data(JsonConfig::default().error_handler(json_extractor_error))
            .app_data(PayloadConfig::default())
            .app_data(webauthn.clone())
            .app_data(Data::new(ws_manager_chan.clone()))
            .app_data(Data::new(rpc_manager_chan.clone()))
            .app_data(rpc_clients.clone())
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
                    .service(get_all_workspaces_admin),
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
                    .service(query_certificate_transparency),
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
