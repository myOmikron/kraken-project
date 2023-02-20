use std::fmt::{Display, Formatter};
use std::io;

use actix_toolbox::tb_middleware::{
    setup_logging_mw, DBSessionStore, LoggingMiddlewareConfig, PersistentSession, SessionMiddleware,
};
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Key, KeyError};
use actix_web::http::StatusCode;
use actix_web::middleware::{Compress, ErrorHandlers};
use actix_web::web::{delete, get, post, scope, Data, JsonConfig, PayloadConfig};
use actix_web::{App, HttpServer};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use rorm::Database;
use webauthn_rs::prelude::{Url, WebauthnError};
use webauthn_rs::WebauthnBuilder;

use crate::api::handler;
use crate::api::middleware::{
    handle_not_found, json_extractor_error, AdminRequired, AuthenticationRequired,
};
use crate::config::Config;

const ORIGIN_NAME: &str = "Kraken";

pub(crate) async fn start_server(db: Database, config: &Config) -> Result<(), StartServerError> {
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
            .service(
                scope("api/v1/auth")
                    .route("login", post().to(handler::login))
                    .route("logout", get().to(handler::logout))
                    .route("start_auth", post().to(handler::start_auth))
                    .route("finish_auth", post().to(handler::finish_auth))
                    .route("start_register", post().to(handler::start_register))
                    .route("finish_register", post().to(handler::finish_register)),
            )
            .service(
                scope("api/v1/admin")
                    .wrap(AdminRequired)
                    .route("users", get().to(handler::get_user))
                    .route("users/{username}", get().to(handler::get_user))
                    .route("users", post().to(handler::create_user))
                    .route("users/{username}", delete().to(handler::delete_user))
                    .route("leeches", get().to(handler::get_leech))
                    .route("leeches/{id}", get().to(handler::get_leech))
                    .route("leeches", post().to(handler::create_leech))
                    .route("leeches/{id}", delete().to(handler::delete_leech))
                    .route("workspaces", get().to(handler::get_workspaces_admin))
                    .route("workspaces/{id}", get().to(handler::get_workspaces_admin)),
            )
            .service(
                scope("api/v1")
                    .wrap(AuthenticationRequired)
                    .route("test", get().to(handler::test))
                    .route("users/me", get().to(handler::get_me))
                    .route("workspaces", get().to(handler::get_workspaces))
                    .route("workspaces/{id}", get().to(handler::get_workspaces))
                    .route("workspaces", post().to(handler::create_workspace))
                    .route("workspaces/{id}", delete().to(handler::delete_workspace)),
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
