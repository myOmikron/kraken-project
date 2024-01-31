use log::error;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::{Change, FindingSection, WsMessage};

/// Sync editor
pub struct EditorSync;

impl EditorSync {
    /// Process a client cursor update event
    ///
    /// The event is fanned out via websocket with
    pub async fn process_client_cursor_update(
        &self,
        user: Uuid,
        finding_definition: Uuid,
        finding_section: FindingSection,
        line: u64,
        column: u64,
    ) {
        let res = GLOBAL.user_cache.get_simple_user(user).await;

        if let Ok(Some(user)) = res {
            GLOBAL
                .ws
                .message_all(WsMessage::ChangedCursorFindingDefinition {
                    finding_definition,
                    finding_section,
                    line,
                    column,
                    user,
                })
                .await;
        } else {
            error!("Could not retrieve user: {res:?}");
        }
    }
}
