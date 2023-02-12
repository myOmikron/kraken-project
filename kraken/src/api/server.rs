use std::fmt::{Display, Formatter};
use std::io;

use actix_toolbox::tb_middleware::{setup_logging_mw, LoggingMiddlewareConfig};
use actix_web::web::{Data, JsonConfig, PayloadConfig};
use actix_web::{App, HttpServer};
use rorm::Database;

use crate::config::Config;

pub(crate) async fn start_server(db: Database, config: &Config) -> Result<(), StartServerError> {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .app_data(JsonConfig::default())
            .app_data(PayloadConfig::default())
            .wrap(setup_logging_mw(LoggingMiddlewareConfig::default()))
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
}

impl Display for StartServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StartServerError::IO(err) => write!(f, "Error starting server: {err}"),
        }
    }
}

impl From<io::Error> for StartServerError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<StartServerError> for String {
    fn from(value: StartServerError) -> Self {
        value.to_string()
    }
}
