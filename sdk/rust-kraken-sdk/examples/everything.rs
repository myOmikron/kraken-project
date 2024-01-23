use std::env;

use kraken::chan::ws_manager::schema::WsMessage;
use kraken_sdk::KrakenClient;
use reqwest::Url;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

const USERNAME: &str = "omikron";
const PASSWORD: &str = "Hallo-123";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO")
    }
    env_logger::init();

    let (tx, rx) = mpsc::channel(1);
    tokio::spawn(async move { handle_ws_msg(rx) });

    let client = KrakenClient::new(
        Url::parse("https://kraken.test").unwrap(),
        USERNAME.to_string(),
        PASSWORD.to_string(),
        Some(tx),
        true,
    )?;
    client.login().await?;

    for workspace in client.get_all_workspaces().await?.workspaces {
        println!("{workspace:?}");
    }

    for invitation in client.get_all_invitations().await?.invitations {
        println!("{invitation:?}");
    }

    Ok(())
}

async fn handle_ws_msg(mut rx: Receiver<WsMessage>) {
    while let Some(msg) = rx.recv().await {
        println!("{msg:?}")
    }
}
