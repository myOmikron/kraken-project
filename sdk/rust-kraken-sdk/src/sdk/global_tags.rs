use kraken::api::handler::global_tags::schema::ListGlobalTags;

use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    /// Get all global tags
    pub async fn get_all_global_tags(&self) -> KrakenResult<ListGlobalTags> {
        self.get("api/v1/globalTags").send().await
    }
}
