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
//! Kraken for itself, does not collect any data.
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(
    feature = "rorm-main",
    allow(dead_code, unused_variables, unused_imports)
)]

use std::io;
use std::io::Write;
use std::sync::Arc;
use std::sync::RwLock;

use actix_toolbox::logging::setup_logging;
use actix_toolbox::logging::AdditionalFileLogger;
use actix_toolbox::logging::LoggingConfig;
use actix_web::cookie::Key;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use clap::Parser;
use clap::Subcommand;
use kraken::api::server;
use kraken::chan::dehashed_manager::start_dehashed_manager;
use kraken::chan::global::GlobalChan;
use kraken::chan::global::GLOBAL;
use kraken::chan::leech_manager::LeechManager;
use kraken::chan::settings_manager::start_settings_manager;
use kraken::chan::ws_manager::chan::start_ws_manager;
use kraken::config;
use kraken::config::DB;
use kraken::config::VAR_DIR;
use kraken::models::User;
use kraken::models::UserPermission;
use kraken::modules::aggregator::Aggregator;
use kraken::modules::cache::EditorCaches;
use kraken::modules::cache::UserCache;
use kraken::modules::cache::WorkspaceUsersCache;
use kraken::modules::editor::EditorSync;
use kraken::modules::media_files::start_file_cleanup;
use kraken::modules::tls::TlsManager;
use kraken::rpc::server::start_rpc_server;
use log::LevelFilter;
use rorm::cli;
use rorm::Database;
use rorm::DatabaseConfiguration;

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(errors) = config::load_env() {
        for error in errors {
            eprintln!("{error}");
        }
        return Err("Failed to load configuration".into());
    }

    let cli = Cli::parse();

    setup_logging(&LoggingConfig {
        log_level: LevelFilter::Debug,
        path: "/var/log/kraken/main.log".to_string(),
        rotation_file_size: "10 MB".parse().unwrap(),
        max_rotation_count: 10,
        alternative_pattern: None,
        additional_file_loggers: vec![AdditionalFileLogger {
            name: "requests".to_string(),
            path: "/var/log/kraken/requests.log".to_string(),
            add_to_main_logger: None,
            rotation_file_size: "10 MB".parse().unwrap(),
            max_rotation_count: 5,
            log_level: None,
            alternative_pattern: Some(
                "{h([{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5}])} {m}{n}".to_string(),
            ),
        }],
    })?;

    match cli.command {
        Command::Migrate { migration_dir } => cli::migrate::run_migrate_custom(
            cli::config::DatabaseConfig {
                last_migration_table_name: None,
                driver: DB.clone(),
            },
            migration_dir,
            false,
            None,
        )
        .await
        .map_err(|e| e.to_string())?,
        Command::Start => {
            let db = get_db().await?;

            let settings = start_settings_manager(&db)
                .await
                .map_err(|e| e.to_string())?;

            let dehashed = RwLock::new(start_dehashed_manager(&settings).await?);

            let tls = Arc::new(
                TlsManager::load(VAR_DIR.get())
                    .map_err(|e| format!("Failed to initialize tls: {e}"))?,
            );

            let leeches = LeechManager::start(db.clone(), tls.clone())
                .await
                .map_err(|e| format!("Failed to query initial leeches: {e}"))?;

            let ws = start_ws_manager().await;
            start_file_cleanup()
                .await
                .map_err(|e| format!("Failed to initialize media file cleanup: {e}"))?;

            let workspace_users_cache = WorkspaceUsersCache::default();
            let user_cache = UserCache::default();
            let editor_cache = EditorCaches::default();

            let aggregator = Aggregator::default();

            let editor_sync = EditorSync::start();

            GLOBAL.init(GlobalChan {
                db,
                leeches,
                ws,
                settings,
                dehashed,
                tls,
                workspace_users_cache,
                user_cache,
                editor_cache,
                aggregator,
                editor_sync,
            });

            let rpc_handle =
                start_rpc_server().map_err(|e| format!("RPC listen address is invalid: {e}"))?;

            server::start_server().await?;

            // Stop the RPC server as the webserver has already shut down
            rpc_handle.abort();
            GLOBAL.db.clone().close().await;
        }
        Command::Keygen => {
            let key = Key::generate();
            println!("{}", BASE64_STANDARD.encode(key.master()));
        }
        Command::CreateAdminUser => {
            let db = get_db().await?;

            create_user(db).await?;
        }
    }

    Ok(())
}

/// Creates a new admin user
///
/// **Parameter**:
/// - `db`: [Database]
// Unwrap is okay, as no handling of errors is possible if we can't communicate with stdin / stdout
async fn create_user(db: Database) -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut username = String::new();
    let mut display_name = String::new();

    print!("Enter a username: ");
    #[allow(clippy::unwrap_used)]
    stdout.flush().unwrap();
    #[allow(clippy::unwrap_used)]
    stdin.read_line(&mut username).unwrap();
    let username = username.trim();

    print!("Enter a display name: ");
    #[allow(clippy::unwrap_used)]
    stdout.flush().unwrap();
    #[allow(clippy::unwrap_used)]
    stdin.read_line(&mut display_name).unwrap();
    let display_name = display_name.trim().to_string();

    #[allow(clippy::unwrap_used)]
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

/// Opens a connection to the database
async fn get_db() -> Result<Database, String> {
    Database::connect(DatabaseConfiguration {
        driver: DB.clone(),
        min_connections: 2,
        max_connections: 20,
        disable_logging: Some(true),
        statement_log_level: None,
        slow_statement_log_level: None,
    })
    .await
    .map_err(|e| format!("Error connecting to the database: {e}"))
}
