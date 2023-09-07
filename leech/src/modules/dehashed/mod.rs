//! The dehashed related code lives here

use dehashed_rs::{DehashedApi, Query, ScheduledRequest, SearchResult};
use log::error;
use tokio::sync::oneshot;

use crate::modules::dehashed::error::DehashedError;

pub mod error;

/// Query the dehashed api
pub async fn query(
    email: String,
    api_key: String,
    query: Query,
) -> Result<SearchResult, DehashedError> {
    let api = DehashedApi::new(email, api_key)?;
    let scheduler = api.start_scheduler();
    let s_tx = scheduler.retrieve_sender();
    let (tx, rx) = oneshot::channel();
    if let Err(err) = s_tx.send(ScheduledRequest::new(query, tx)).await {
        error!("Couldn't send to dehashed scheduler: {err}");
        return Err(DehashedError::DehashedSchedulerUnreachable);
    }

    match rx.await {
        Ok(x) => Ok(x?),
        Err(x) => {
            error!("{x}");

            Err(DehashedError::DehashedSchedulerUnreachable)
        }
    }
}
