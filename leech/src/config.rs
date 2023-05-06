//! The configuration definitions of a leech

use std::fmt::{Display, Formatter};
use std::path::Path;
use std::{fs, io};

use serde::{Deserialize, Serialize};

use crate::logging::LoggingConfig;

/// The configuration of the server part
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServerConfig {
    /// Address the gRPC server listens on
    pub listen_address: String,
    /// Port of the gRPC server
    pub listen_port: u16,
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
}

/// Errors that can occur while
pub enum GetConfigError {
    /// No file exists at the specified path
    PathDoesNotExist,
    /// A directory exists at the specified path
    PathIsDirectory,
    /// An io error occurred.
    IO(io::Error),
    /// Invalid toml format found within the file
    InvalidToml(toml::de::Error),
}

impl Display for GetConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GetConfigError::PathDoesNotExist => {
                write!(f, "The config file does not exist at the specified path")
            }
            GetConfigError::PathIsDirectory => {
                write!(f, "The config file is a directory")
            }
            GetConfigError::IO(err) => write!(f, "io error while reading the config file: {err}"),
            GetConfigError::InvalidToml(err) => {
                write!(f, "The config file contains invalid TOML: {err}")
            }
        }
    }
}

impl From<io::Error> for GetConfigError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<toml::de::Error> for GetConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::InvalidToml(value)
    }
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
