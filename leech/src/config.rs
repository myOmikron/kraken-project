//! The configuration definitions of a leech

use std::path::Path;
use std::{fs, io};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::logging::LoggingConfig;

/// The configuration for leech.
///
/// This struct can be parsed from a configuration file
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,

    /// Database configuration
    pub database: DBConfig,

    /// Logging configuration
    ///
    /// Only used in the [`server`](crate::Command::Server) command
    pub logging: LoggingConfig,

    /// Dehashed configuration
    pub dehashed: Option<DehashedConfig>,

    /// The configuration for all kraken related stuff
    pub kraken: KrakenConfig,
}

/// The configuration of the server part
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServerConfig {
    /// Address the gRPC server listens on
    pub listen_address: String,
    /// Port of the gRPC server
    pub listen_port: u16,
}

/// The configuration of the connection to kraken
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct KrakenConfig {
    /// The url to reach kraken's grpc server
    pub kraken_uri: Url,

    /// The fake domain the kraken's cert is valid for
    pub kraken_sni: String,

    /// PEM encoded CA managed by kraken
    pub kraken_ca: String,

    /// PEM encoded certificate to present when communicating with kraken over grpc
    pub leech_cert: String,

    /// PEM encoded private key for the `leech_key`
    pub leech_key: String,
}

/// The configuration of the dehashed API
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DehashedConfig {
    /// The email that is used for login
    pub email: String,
    /// The api key that is used for login
    pub api_key: String,
}

/// The configuration of the database related settings
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DBConfig {
    /// Address of the database
    pub host: String,
    /// Port of the database
    pub port: u16,
    /// Name of the database
    pub name: String,
    /// User to use to connect to the database
    pub user: String,
    /// Password to use to connect to the database
    pub password: String,
}

/// Errors that can occur while retrieving the config file
#[derive(Error, Debug)]
pub enum GetConfigError {
    /// No file exists at the specified path
    #[error("The config file does not exist at the specified path")]
    PathDoesNotExist,
    /// A directory exists at the specified path
    #[error("The config file is a directory")]
    PathIsDirectory,
    /// An io error occurred.
    #[error("io error while reading the config file: {0}")]
    IO(#[from] io::Error),
    /// Invalid toml format found within the file
    #[error("The config file contains invalid TOML: {0}")]
    InvalidToml(#[from] toml::de::Error),
}

impl From<GetConfigError> for String {
    fn from(value: GetConfigError) -> Self {
        value.to_string()
    }
}

/// Retrieve an instance of [Config] from the specified path.
///
/// The file is parsed with a TOML parser.
pub fn get_config(path: &str) -> Result<Config, GetConfigError> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(GetConfigError::PathDoesNotExist);
    }

    if !p.is_file() {
        return Err(GetConfigError::PathIsDirectory);
    }

    let config = toml::from_str(&fs::read_to_string(p)?)?;

    Ok(config)
}
