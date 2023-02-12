//! A scalable pen-testing platform.
//!
//! # Kraken
//! The core of the kraken project.
//!
//! It provides an API for accessing and retrieving data and events for the user
//! as well as an gRPC client and server interface for the leeches.
//!
//! ## Leeches
//! Leeches are the workers of kraken.
//! Kraken for it self, does not collect any data.
#![warn(missing_docs)]
#![cfg_attr(
    feature = "rorm-main",
    allow(dead_code, unused_variables, unused_imports)
)]

use std::fs::read_to_string;

use actix_toolbox::logging::setup_logging;
use actix_web::cookie::Key;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use clap::{Parser, Subcommand};

use crate::config::Config;

pub mod config;

#[derive(Subcommand)]
enum Command {
    Start,
    Keygen,
}

#[derive(Parser)]
struct Cli {
    #[clap(long = "config-path")]
    #[clap(help = "Specify an alternative path to the config file")]
    #[clap(default_value_t = String::from("/etc/kraken/config.toml"))]
    config_path: String,

    #[clap(subcommand)]
    command: Command,
}

#[rorm::rorm_main]
#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let config_content =
        read_to_string(&cli.config_path).map_err(|e| format!("Error reading config file: {e}"))?;
    let config: Config =
        toml::from_str(&config_content).map_err(|e| format!("Error parsing config file: {e}"))?;

    setup_logging(&config.logging)?;

    match cli.command {
        Command::Start => {}
        Command::Keygen => {
            let key = Key::generate();
            println!("{}", BASE64_STANDARD.encode(key.master()));
        }
    }

    Ok(())
}
