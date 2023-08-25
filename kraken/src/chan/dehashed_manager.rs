use std::sync::Arc;

use dehashed_rs::{DehashedApi, Scheduler};

use crate::chan::SettingsManagerChan;

/// Start the dehashed manager
pub async fn start_dehashed_manager(
    settings: Arc<SettingsManagerChan>,
) -> Result<Option<Scheduler>, String> {
    let settings = settings.get_settings();

    if settings.dehashed_email.is_none() || settings.dehashed_api_key.is_none() {
        return Ok(None);
    }

    let api = DehashedApi::new(
        settings.dehashed_email.unwrap(),
        settings.dehashed_api_key.unwrap(),
    )
    .map_err(|e| format!("Error starting dehashed api: {e}"))?;

    let scheduler = api.start_scheduler();

    Ok(Some(scheduler))
}
