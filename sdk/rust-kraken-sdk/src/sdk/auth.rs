use kraken::api::handler::auth::schema::LoginRequest;
use log::info;

use crate::error::KrakenError;
use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Test if the user is authenticated
    pub async fn test(&self) -> KrakenResult<bool> {
        match self.get("api/v1/auth/test").send().await {
            Ok(unit) => {
                let _: () = unit;
                Ok(true)
            }
            Err(KrakenError::AuthenticationFailed) => Ok(false),
            Err(err) => Err(err),
        }
    }

    /// Logging in
    pub async fn login(&self) -> KrakenResult<()> {
        info!("Logging in");
        self.post("api/v1/auth/login")
            .body(LoginRequest {
                username: self.username.clone(),
                password: self.password.clone(),
            })
            .send::<()>()
            .await?;

        info!("Logged in successfully");
        info!("Starting websocket");
        self.start_ws().await?;
        info!("Websocket started");

        Ok(())
    }

    /// Logout
    pub async fn logout(&self) -> KrakenResult<()> {
        self.get("api/v1/auth/logout").send().await
    }
}
