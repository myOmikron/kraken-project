//! Definitions of the configuration file

use actix_toolbox::logging::LoggingConfig;
use serde::{Deserialize, Serialize};

/// Server related configuration.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServerConfig {
    /// Address the API server should bind to
    pub api_listen_address: String,
    /// Port the API server should bind to
    pub api_listen_port: u16,
    /// Address the gRPC server should bind to
    pub rpc_listen_address: String,
    /// Port the gRPC server should bind to
    pub rpc_listen_port: u16,
    /// The secret key is used for sessions signing / encryption.
    pub secret_key: String,
    /// The bearer token that is used to validate calls to reporting endpoints
    pub reporting_key: String,
    /// The URI this server is reachable by.
    ///
    /// Scheme and port must be included as well
    pub origin_uri: String,
}

/// Database related configuration.
///
/// As the only supported database is postgres, no driver configuration is needed
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DBConfig {
    /// The address of the database server
    pub host: String,
    /// The port of the database server
    pub port: u16,
    /// The database name
    pub name: String,
    /// The user to use for the database connection
    pub user: String,
    /// Password for the user
    pub password: String,
}

/// Definition of the main configuration.
///
/// This model can be parsed from the config.toml
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Database configuration
    pub database: DBConfig,
}
