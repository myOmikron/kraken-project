use rorm::prelude::*;
use uuid::Uuid;

/// An registered application which may perform oauth requests
#[derive(Model)]
pub struct OauthClient {
    /// The primary key as well as oauth's `client_id`
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// A name to show the user when asking for permissions
    #[rorm(max_length = 255)]
    pub name: String,

    /// oauth's `client_secret` to compare with in the `/token` request
    #[rorm(max_length = 255)]
    pub secret: String,

    /// oauth's `redirect_uri` to compare with in the initial `/auth` request
    #[rorm(max_length = 255)]
    pub redirect_uri: String,
}
