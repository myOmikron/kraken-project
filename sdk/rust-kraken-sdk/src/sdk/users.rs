use kraken::api::handler::users::schema::ListUsers;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Retrieve all users of kraken
    pub async fn get_all_users(&self) -> KrakenResult<ListUsers> {
        self.get("api/v1/users").send().await
    }
}
