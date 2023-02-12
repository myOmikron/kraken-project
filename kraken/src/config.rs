//! Definitions of the configuration file

use actix_toolbox::logging::LoggingConfig;
use serde::{Deserialize, Serialize};

/// Definition of the main configuration.
///
/// This model can be parsed from the config.toml
#[derive(Deserialize, Serialize)]
pub struct Config {
    /// Logging configuration
    pub logging: LoggingConfig,
}
