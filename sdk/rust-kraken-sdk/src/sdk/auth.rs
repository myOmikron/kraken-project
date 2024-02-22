use kraken::api::handler::auth::schema::LoginRequest;
use log::info;

use crate::error::KrakenError;
use crate::sdk::utils::KrakenRequest;
use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Test if the user is authenticated
    pub async fn test(&self) -> KrakenResult<bool> {
        #[allow(clippy::expect_used)]
        let url = self.base_url.join("api/v1/auth/test").expect("Valid url");

        match self.make_request(KrakenRequest::get(url).build()).await {
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
        #[allow(clippy::expect_used)]
        let url = self.base_url.join("api/v1/auth/login").expect("Valid url");

        info!("Logging in");
        self.make_request(
            KrakenRequest::post(url)
                .body(LoginRequest {
                    username: self.username.clone(),
                    password: self.password.clone(),
                })
                .build(),
        )
        .await?;

        info!("Logged in successfully");
        info!("Starting websocket");
        self.start_ws().await?;
        info!("Websocket started");

        Ok(())
    }

    /// Logout
    pub async fn logout(&self) -> Result<(), KrakenError> {
        #[allow(clippy::expect_used)]
        let url = self.base_url.join("api/v1/auth/logout").expect("Valid url");
        self.make_request(KrakenRequest::get(url).build()).await?;

        Ok(())
    }
}
