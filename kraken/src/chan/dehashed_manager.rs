use dehashed_rs::{DehashedApi, Scheduler};

use crate::chan::settings_manager::SettingsManagerChan;

/// Start the dehashed manager
pub async fn start_dehashed_manager(
    settings: &SettingsManagerChan,
) -> Result<Option<Scheduler>, String> {
    let settings = settings.get_settings();

    let Some(email) = settings.dehashed_email else {
        return Ok(None);
    };
    let Some(api_key) = settings.dehashed_api_key else {
        return Ok(None);
    };

    let api = DehashedApi::new(email, api_key)
        .map_err(|e| format!("Error starting dehashed api: {e}"))?;

    let scheduler = api.start_scheduler();

    Ok(Some(scheduler))
}
