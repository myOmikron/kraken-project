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
use std::process::exit;
use std::sync::Arc;

use actix_toolbox::logging::setup_logging;
use actix_web::cookie::Key;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use clap::{Parser, Subcommand};
use rand::thread_rng;
use rorm::{
    cli, insert, query, Database, DatabaseConfiguration, DatabaseDriver, FieldAccess, Model,
};
use webauthn_rs::prelude::Uuid;

use crate::api::server;
use crate::config::Config;
use crate::models::{User, UserInsert};
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

            let settings_manager_chan = Arc::new(
                chan::start_settings_manager(&db)
                    .await
                    .map_err(|e| e.to_string())?,
            );

            let (rpc_manager_chan, rpc_clients) = chan::start_rpc_manager(db.clone()).await?;
            let ws_manager_chan = chan::start_ws_manager().await?;
            let dehashed_scheduler =
                chan::start_dehashed_manager(settings_manager_chan.clone()).await?;

            start_rpc_server(&config, db.clone())?;

            server::start_server(
                db,
                &config,
                rpc_manager_chan,
                rpc_clients,
                ws_manager_chan,
                settings_manager_chan,
                dehashed_scheduler,
            )
            .await?;
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

    if query!(&db, (User::F.username,))
        .condition(User::F.username.equals(username))
        .optional()
        .await
        .unwrap()
        .is_some()
    {
        eprintln!("There is already a user with that name");
        exit(1);
    }

    print!("Enter a display name: ");
    stdout.flush().unwrap();
    stdin.read_line(&mut display_name).unwrap();

    let password = rpassword::prompt_password("Enter password: ").unwrap();

    let salt = SaltString::generate(&mut thread_rng());
    let hashed_password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    insert!(&db, UserInsert)
        .single(&UserInsert {
            username: username.to_string(),
            display_name: display_name.to_string(),
            password_hash: hashed_password,
            admin: true,
            last_login: None,
            uuid: Uuid::new_v4(),
        })
        .await
        .map_err(|e| format!("Failed to create user: {e}"))?;

    println!("Created user {username}");

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
