use kraken::api::handler::wordlists::schema::ListWordlists;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Retrieve all wordlists
    pub async fn get_all_wordlists(&self) -> KrakenResult<ListWordlists> {
        #[allow(clippy::expect_used)]
        let url = self.base_url.join("api/v1/wordlists").expect("Valid url");

        self.get(url).send().await
    }
}
