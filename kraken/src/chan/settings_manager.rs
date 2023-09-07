use rorm::{insert, query, Database, Model};
use thiserror::Error;
use tokio::sync::watch;
use tokio::sync::watch::error::SendError;
use tokio::sync::watch::{Receiver, Sender};
use uuid::Uuid;

use crate::models::{Settings, SettingsInsert};

/// The errors that can occur while
#[derive(Error, Debug)]
pub enum SettingsManagerError {
    /// Errors while occur while interacting with the database
    #[error("Database error: {0}")]
    Database(#[from] rorm::Error),
    /// An error occurred while pushing an update to the watch
    #[error("Watch send error: {0}")]
    SendError(#[from] SendError<Settings>),
}

/// The settings manager channel
///
/// This struct is intended to be handed out to handlers.
pub struct SettingsManagerChan {
    rx: Receiver<Settings>,
    tx: Sender<Settings>,
    db: Database,
}

impl SettingsManagerChan {
    /// Update the currently active settings
    pub async fn update_settings(
        &self,
        settings: &SettingsInsert,
    ) -> Result<(), SettingsManagerError> {
        let settings = insert!(&self.db, SettingsInsert).single(settings).await?;

        self.tx.send(settings)?;

        Ok(())
    }

    /// Retrieve the currently active settings
    pub fn get_settings(&self) -> Settings {
        self.rx.borrow().clone()
    }
}

/// Start the settings manager
pub async fn start_settings_manager(
    db: &Database,
) -> Result<SettingsManagerChan, SettingsManagerError> {
    let settings = match query!(db, Settings)
        .order_desc(Settings::F.created_at)
        .optional()
        .await?
    {
        Some(x) => x,
        None => {
            insert!(db, SettingsInsert)
                .single(&SettingsInsert {
                    uuid: Uuid::new_v4(),
                    dehashed_api_key: None,
                    dehashed_email: None,
                })
                .await?
        }
    };

    let (tx, rx) = watch::channel(settings);

    Ok(SettingsManagerChan {
        db: db.clone(),
        rx,
        tx,
    })
}
