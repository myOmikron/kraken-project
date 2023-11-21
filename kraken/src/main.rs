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
use std::io;
use std::io::Write;
use std::sync::{Arc, RwLock};

use actix_toolbox::logging::setup_logging;
use actix_web::cookie::Key;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use clap::{Parser, Subcommand};
use rorm::{cli, Database, DatabaseConfiguration, DatabaseDriver};

use crate::api::server;
use crate::chan::{GlobalChan, LeechManager, GLOBAL};
use crate::config::Config;
use crate::models::{User, UserPermission};
use crate::modules::tls::TlsManager;
use crate::rpc::server::start_rpc_server;

pub mod api;
pub mod chan;
pub mod config;
pub mod models;
pub mod modules;
pub(crate) mod rpc;

/// The subcommands of kraken
#[derive(Subcommand)]
pub enum Command {
    /// Start the server
    Start,
    /// Generate a secret key
    Keygen,
    /// Creates a new user with administrative privileges
    CreateAdminUser,
    /// Apply the migrations to the database
    Migrate {
        /// The directory the migrations live in
        migration_dir: String,
    },
}

/// The cli for kraken
#[derive(Parser)]
#[clap(version, about = "The kraken core")]
pub struct Cli {
    /// Specify an alternative path to the config file
    #[clap(long = "config-path")]
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
        Command::Migrate { migration_dir } => cli::migrate::run_migrate_custom(
            cli::config::DatabaseConfig {
                last_migration_table_name: None,
                driver: DatabaseDriver::Postgres {
                    host: config.database.host,
                    port: config.database.port,
                    name: config.database.name,
                    user: config.database.user,
                    password: config.database.password,
                },
            },
            migration_dir,
            false,
            None,
        )
        .await
        .map_err(|e| e.to_string())?,
        Command::Start => {
            let db = get_db(&config).await?;

            let settings = Arc::new(
                chan::start_settings_manager(&db)
                    .await
                    .map_err(|e| e.to_string())?,
            );
            let dehashed = Arc::new(RwLock::new(
                chan::start_dehashed_manager(settings.clone()).await?,
            ));

            let tls = Arc::new(
                TlsManager::load("/var/lib/kraken")
                    .map_err(|e| format!("Failed to initialize tls: {e}"))?,
            );
            let leeches = LeechManager::start(db.clone(), tls.clone())
                .await
                .map_err(|e| format!("Failed to query initial leeches: {e}"))?;

            let ws = chan::start_ws_manager().await;

            GLOBAL.init(GlobalChan {
                db,
                leeches,
                ws,
                settings,
                dehashed,
                tls,
            });

            start_rpc_server(&config);

            server::start_server(&config).await?;

            // TODO: Stop rpc server as it also has access to the database
            GLOBAL.db.clone().close().await;
        }
        Command::Keygen => {
            let key = Key::generate();
            println!("{}", BASE64_STANDARD.encode(key.master()));
        }
        Command::CreateAdminUser => {
            let db = get_db(&config).await?;

            create_user(db).await?;
        }
    }

    Ok(())
}

/// Creates a new admin user
///
/// **Parameter**:
/// - `db`: [Database]
async fn create_user(db: Database) -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut username = String::new();
    let mut display_name = String::new();

    print!("Enter a username: ");
    stdout.flush().unwrap();
    stdin.read_line(&mut username).unwrap();
    let username = username.trim();

    print!("Enter a display name: ");
    stdout.flush().unwrap();
    stdin.read_line(&mut display_name).unwrap();

    let password = rpassword::prompt_password("Enter password: ").unwrap();

    User::insert_local_user(
        &db,
        username.to_string(),
        display_name,
        password,
        UserPermission::Admin,
    )
    .await
    .map_err(|e| format!("Failed to create user: {e}"))?;

    println!("Created user {username}");

    db.close().await;

    Ok(())
}

/// Opens a connection to the database using the provided config
///
/// **Parameter**:
/// - `config`: Reference to [Config]
async fn get_db(config: &Config) -> Result<Database, String> {
    let db_config = DatabaseConfiguration {
        driver: DatabaseDriver::Postgres {
            host: config.database.host.clone(),
            port: config.database.port,
            user: config.database.user.clone(),
            password: config.database.password.clone(),
            name: config.database.name.clone(),
        },
        min_connections: 2,
        max_connections: 20,
        disable_logging: Some(true),
        statement_log_level: None,
        slow_statement_log_level: None,
    };

    Database::connect(db_config)
        .await
        .map_err(|e| format!("Error connecting to the database: {e}"))
}
