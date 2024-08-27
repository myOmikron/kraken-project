use kraken::api::handler::wordlists::schema::ListWordlists;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Retrieve all wordlists
    pub async fn get_all_wordlists(&self) -> KrakenResult<ListWordlists> {
        self.get("api/v1/wordlists").send().await
    }
}
