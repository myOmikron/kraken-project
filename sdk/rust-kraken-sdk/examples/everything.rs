use std::env;

use kraken_sdk::KrakenClient;
use reqwest::Url;

const USERNAME: &str = "omikron";
const PASSWORD: &str = "Hallo-123";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO")
    }
    env_logger::init();

    let client = KrakenClient::new(
        Url::parse("https://kraken.test").unwrap(),
        USERNAME.to_string(),
        PASSWORD.to_string(),
        None,
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
