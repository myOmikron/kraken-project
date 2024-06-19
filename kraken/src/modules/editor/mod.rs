//! This module provides synchronization capabilities for the editor.
//!
//! Use [EditorSync] to add events that should be processed.
//!
//! The internal synchronization will take care of fanning the events to the websocket
//! as well as caching the results internally and saving them regularly to the database

use std::sync::Arc;

use log::debug;
use log::error;
use log::warn;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::Change;
use crate::chan::ws_manager::schema::CursorPosition;
use crate::chan::ws_manager::schema::EditorTarget;
use crate::chan::ws_manager::schema::FindingDetails;
use crate::chan::ws_manager::schema::FindingSection;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::FindingDefinition;

/// Sync editor
#[derive(Clone)]
pub struct EditorSync {
    ws_notes_tx: Arc<Sender<(Uuid, Uuid, Change)>>,
    finding_definition_tx: Arc<Sender<(Uuid, Uuid, FindingSection, Change)>>,
    finding_tx: Arc<Sender<(Uuid, Uuid, FindingDetails, Change)>>,
    finding_affected_tx: Arc<Sender<(Uuid, Uuid, Uuid, FindingDetails, Change)>>,
}

impl EditorSync {
    /// Create a new instance of [EditorSync]
    ///
    /// This will also start the worker for the synchronization
    pub fn start() -> Self {
        let (ws_notes_tx, ws_notes_rx) = mpsc::channel(1);
        let (finding_definition_tx, finding_definition_rx) = mpsc::channel(1);
        let (finding_tx, finding_rx) = mpsc::channel(1);
        let (finding_affected_tx, finding_affected_rx) = mpsc::channel(1);

        let editor_sync = Self {
            ws_notes_tx: Arc::new(ws_notes_tx),
            finding_definition_tx: Arc::new(finding_definition_tx),
            finding_tx: Arc::new(finding_tx),
            finding_affected_tx: Arc::new(finding_affected_tx),
        };
        tokio::spawn(
            editor_sync
                .clone()
                .process_client_edit_ws_notes(ws_notes_rx),
        );
        tokio::spawn(
            editor_sync
                .clone()
                .process_client_edit_finding_definition(finding_definition_rx),
        );
        tokio::spawn(editor_sync.clone().process_finding(finding_rx));
        tokio::spawn(
            editor_sync
                .clone()
                .process_finding_affected(finding_affected_rx),
        );

        editor_sync
    }

    /// Send workspace notes change
    pub async fn send_ws_notes(&self, user: Uuid, ws: Uuid, change: Change) {
        if let Err(err) = self.ws_notes_tx.send((user, ws, change)).await {
            error!("Couldn't send to EditorSync: {err}");
        }
    }

    /// Send finding definition change
    pub async fn send_finding_definition(
        &self,
        user: Uuid,
        fd: Uuid,
        fs: FindingSection,
        change: Change,
    ) {
        if let Err(err) = self
            .finding_definition_tx
            .send((user, fd, fs, change))
            .await
        {
            error!("Couldn't send to EditorSync: {err}");
        }
    }

    /// Send a finding details change
    pub async fn send_finding(
        &self,
        user: Uuid,
        finding: Uuid,
        details: FindingDetails,
        change: Change,
    ) {
        if let Err(err) = self.finding_tx.send((user, finding, details, change)).await {
            error!("Couldn't send to EditorSync: {err}");
        }
    }

    /// Send a finding affected details change
    pub async fn send_finding_affected(
        &self,
        user: Uuid,
        finding: Uuid,
        affected: Uuid,
        details: FindingDetails,
        change: Change,
    ) {
        if let Err(err) = self
            .finding_affected_tx
            .send((user, finding, affected, details, change))
            .await
        {
            error!("Couldn't send to EditorSync: {err}");
        }
    }

    /// Start processing of [FindingAffected] user-details changes
    pub async fn process_finding_affected(
        self,
        mut rx: Receiver<(Uuid, Uuid, Uuid, FindingDetails, Change)>,
    ) {
        while let Some((user, finding, affected, finding_details, change)) = rx.recv().await {
            let cache_result = match finding_details {
                FindingDetails::Export => {
                    GLOBAL
                        .editor_cache
                        .finding_affected_export_details
                        .get(affected)
                        .await
                }
                FindingDetails::User => {
                    GLOBAL
                        .editor_cache
                        .finding_affected_user_details
                        .get(affected)
                        .await
                }
            };
            let (existing, ws) = match cache_result {
                Ok(details) => {
                    if let Some(x) = details {
                        x
                    } else {
                        debug!("Finding affected details not exist");
                        continue;
                    }
                }
                Err(err) => {
                    error!("Error gathering finding affected details from cache: {err}");
                    continue;
                }
            };

            // Check access
            let users = match GLOBAL.workspace_users_cache.get_users(ws, &GLOBAL.db).await {
                Ok(Some(users)) => users,
                Ok(None) => {
                    error!("Workspace from cache does not exist in workspace users cache");
                    continue;
                }
                Err(err) => {
                    error!("DB error: {err}");
                    continue;
                }
            };

            if !users.iter().any(|x| *x == user) {
                debug!("User does not has access to the workspace");
                continue;
            }

            // Get simple user for websocket msg
            let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
                error!("Could not retrieve user from cache for active websocket");
                continue;
            };

            // Check change
            if !is_change_valid(&change) {
                warn!("Invalid change received");
                continue;
            }

            // Update cache
            let cache_result = match finding_details {
                FindingDetails::Export => {
                    GLOBAL
                        .editor_cache
                        .finding_affected_export_details
                        .update(affected, apply_change(&existing, &change))
                        .await
                }
                FindingDetails::User => {
                    GLOBAL
                        .editor_cache
                        .finding_affected_user_details
                        .update(affected, apply_change(&existing, &change))
                        .await
                }
            };
            if let Err(err) = cache_result {
                error!("Error updating finding affected details cache: {err}");
                continue;
            }

            // Send websocket message
            let msg = WsMessage::EditorChangedContent {
                user,
                change,
                target: EditorTarget::FindingAffected {
                    finding,
                    affected,
                    finding_details,
                },
            };
            GLOBAL.ws.message_workspace(ws, msg).await;
        }
    }

    /// Process a client cursor update event
    ///
    /// The event is fanned out via websocket
    pub async fn process_client_cursor_update_finding_affected_details(
        &self,
        user: Uuid,
        finding: Uuid,
        affected: Uuid,
        finding_details: FindingDetails,
        cursor: CursorPosition,
    ) {
        let cache_result = match finding_details {
            FindingDetails::Export => {
                GLOBAL
                    .editor_cache
                    .finding_affected_export_details
                    .get(affected)
                    .await
            }
            FindingDetails::User => {
                GLOBAL
                    .editor_cache
                    .finding_affected_user_details
                    .get(affected)
                    .await
            }
        };
        let (_, ws) = match cache_result {
            Ok(details) => {
                if let Some(x) = details {
                    x
                } else {
                    debug!("Finding details not exist");
                    return;
                }
            }
            Err(err) => {
                error!("Error gathering finding details from cache: {err}");
                return;
            }
        };

        // Check access
        let users = match GLOBAL.workspace_users_cache.get_users(ws, &GLOBAL.db).await {
            Ok(Some(users)) => users,
            Ok(None) => {
                error!("Workspace was not found");
                return;
            }
            Err(err) => {
                error!("Database error occurred: {err}");
                return;
            }
        };
        if !users.iter().any(|x| *x == user) {
            debug!("User does not has access to the workspace");
            return;
        }

        let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
            error!("Could not retrieve user");
            return;
        };

        GLOBAL
            .ws
            .message_all(WsMessage::EditorChangedCursor {
                target: EditorTarget::FindingAffected {
                    finding,
                    affected,
                    finding_details,
                },
                cursor,
                user,
            })
            .await;
    }

    /// Start processing of finding user details changes
    pub async fn process_finding(self, mut rx: Receiver<(Uuid, Uuid, FindingDetails, Change)>) {
        while let Some((user, finding, finding_details, change)) = rx.recv().await {
            let cache_result = match finding_details {
                FindingDetails::Export => {
                    GLOBAL
                        .editor_cache
                        .finding_export_details
                        .get(finding)
                        .await
                }
                FindingDetails::User => GLOBAL.editor_cache.finding_user_details.get(finding).await,
            };
            let (existing, ws) = match cache_result {
                Ok(details) => {
                    if let Some(x) = details {
                        x
                    } else {
                        debug!("Finding details not exist");
                        continue;
                    }
                }
                Err(err) => {
                    error!("Error gathering finding details from cache: {err}");
                    continue;
                }
            };

            // Check access
            let users = match GLOBAL.workspace_users_cache.get_users(ws, &GLOBAL.db).await {
                Ok(Some(users)) => users,
                Ok(None) => {
                    error!("Workspace from cache does not exist in workspace users cache");
                    continue;
                }
                Err(err) => {
                    error!("DB error: {err}");
                    continue;
                }
            };

            if !users.iter().any(|x| *x == user) {
                debug!("User does not has access to the workspace");
                continue;
            }

            // Get simple user for websocket msg
            let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
                error!("Could not retrieve user from cache for active websocket");
                continue;
            };

            // Check change
            if !is_change_valid(&change) {
                warn!("Invalid change received");
                continue;
            }

            // Update cache
            let cache_result = match finding_details {
                FindingDetails::Export => {
                    GLOBAL
                        .editor_cache
                        .finding_export_details
                        .update(finding, apply_change(&existing, &change))
                        .await
                }
                FindingDetails::User => {
                    GLOBAL
                        .editor_cache
                        .finding_user_details
                        .update(finding, apply_change(&existing, &change))
                        .await
                }
            };
            if let Err(err) = cache_result {
                error!("Error updating finding details cache: {err}");
                continue;
            }

            // Send websocket message
            let msg = WsMessage::EditorChangedContent {
                user,
                change,
                target: EditorTarget::Finding {
                    finding,
                    finding_details,
                },
            };
            GLOBAL.ws.message_workspace(ws, msg).await;
        }
    }

    /// Process a client cursor update event
    ///
    /// The event is fanned out via websocket
    pub async fn process_client_cursor_update_finding_details(
        &self,
        user: Uuid,
        finding: Uuid,
        finding_details: FindingDetails,
        cursor: CursorPosition,
    ) {
        let cache_result = match finding_details {
            FindingDetails::Export => {
                GLOBAL
                    .editor_cache
                    .finding_export_details
                    .get(finding)
                    .await
            }
            FindingDetails::User => GLOBAL.editor_cache.finding_user_details.get(finding).await,
        };
        let (_, ws) = match cache_result {
            Ok(details) => {
                if let Some(x) = details {
                    x
                } else {
                    debug!("Finding details not exist");
                    return;
                }
            }
            Err(err) => {
                error!("Error gathering finding details from cache: {err}");
                return;
            }
        };

        // Check access
        let users = match GLOBAL.workspace_users_cache.get_users(ws, &GLOBAL.db).await {
            Ok(Some(users)) => users,
            Ok(None) => {
                error!("Workspace was not found");
                return;
            }
            Err(err) => {
                error!("Database error occurred: {err}");
                return;
            }
        };
        if !users.iter().any(|x| *x == user) {
            debug!("User does not has access to the workspace");
            return;
        }

        let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
            error!("Could not retrieve user");
            return;
        };

        GLOBAL
            .ws
            .message_all(WsMessage::EditorChangedCursor {
                target: EditorTarget::Finding {
                    finding,
                    finding_details,
                },
                cursor,
                user,
            })
            .await;
    }

    /// Process a change in workspace notes
    pub async fn process_client_edit_ws_notes(self, mut rx: Receiver<(Uuid, Uuid, Change)>) {
        while let Some((user, ws, change)) = rx.recv().await {
            let (existing, _) = match GLOBAL.editor_cache.ws_notes.get(ws).await {
                Ok(ws) => {
                    if let Some(ws) = ws {
                        ws
                    } else {
                        debug!("Workspace does not exist");
                        continue;
                    }
                }

                Err(err) => {
                    error!("Error gathering ws from cache: {err}");
                    continue;
                }
            };

            let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
                error!("Could not retrieve user from cache for active websocket");
                continue;
            };

            if !is_change_valid(&change) {
                warn!("Invalid change received!");
                continue;
            }

            if let Err(err) = GLOBAL
                .editor_cache
                .ws_notes
                .update(ws, apply_change(&existing, &change))
                .await
            {
                error!("Cache error: {err}");
                continue;
            }

            // Notify all users about the change
            GLOBAL
                .ws
                .message_all(WsMessage::EditorChangedContent {
                    target: EditorTarget::WorkspaceNotes { workspace: ws },
                    change,
                    user,
                })
                .await;
        }
    }

    /// Process a client cursor update event
    ///
    /// The event is fanned out via websocket with
    pub async fn process_client_cursor_update_ws_notes(
        &self,
        user: Uuid,
        ws: Uuid,
        cursor: CursorPosition,
    ) {
        if let Err(err) = GLOBAL.editor_cache.ws_notes.get(ws).await {
            error!("Error gathering ws from cache: {err}");
            return;
        }

        let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
            error!("Could not retrieve user");
            return;
        };

        GLOBAL
            .ws
            .message_all(WsMessage::EditorChangedCursor {
                target: EditorTarget::WorkspaceNotes { workspace: ws },
                cursor,
                user,
            })
            .await;
    }

    /// Process a client's EditFindingDefinition update
    pub async fn process_client_edit_finding_definition(
        self,
        mut rx: Receiver<(Uuid, Uuid, FindingSection, Change)>,
    ) {
        while let Some((user, fd, fs, change)) = rx.recv().await {
            if !self.does_fd_exist(fd).await {
                continue;
            }

            let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
                error!("Could not retrieve user from cache for active websocket");
                continue;
            };

            if !is_change_valid(&change) {
                warn!("Invalid change received!");
                continue;
            }

            // Apply change
            match fs {
                FindingSection::Summary => {
                    let Ok(Some((summary, _))) = GLOBAL.editor_cache.fd_summary.get(fd).await
                    else {
                        warn!("Summary not in cache");
                        continue;
                    };
                    if let Err(err) = GLOBAL
                        .editor_cache
                        .fd_summary
                        .update(fd, apply_change(&summary, &change))
                        .await
                    {
                        warn!("Cache error: {err}");
                        continue;
                    }
                }
                FindingSection::Description => {
                    let Ok(Some((description, _))) =
                        GLOBAL.editor_cache.fd_description.get(fd).await
                    else {
                        warn!("Description not in cache");
                        continue;
                    };
                    if let Err(err) = GLOBAL
                        .editor_cache
                        .fd_description
                        .update(fd, apply_change(&description, &change))
                        .await
                    {
                        warn!("Cache error: {err}");
                        continue;
                    }
                }
                FindingSection::Impact => {
                    let Ok(Some((impact, _))) = GLOBAL.editor_cache.fd_impact.get(fd).await else {
                        warn!("Impact not in cache");
                        continue;
                    };
                    if let Err(err) = GLOBAL
                        .editor_cache
                        .fd_impact
                        .update(fd, apply_change(&impact, &change))
                        .await
                    {
                        warn!("Cache error: {err}");
                        continue;
                    }
                }
                FindingSection::Remediation => {
                    let Ok(Some((remediation, _))) =
                        GLOBAL.editor_cache.fd_remediation.get(fd).await
                    else {
                        warn!("Remediation not in cache");
                        continue;
                    };
                    if let Err(err) = GLOBAL
                        .editor_cache
                        .fd_remediation
                        .update(fd, apply_change(&remediation, &change))
                        .await
                    {
                        warn!("Cache error: {err}");
                        continue;
                    }
                }
                FindingSection::References => {
                    let Ok(Some((references, _))) = GLOBAL.editor_cache.fd_references.get(fd).await
                    else {
                        warn!("References not in cache");
                        continue;
                    };
                    if let Err(err) = GLOBAL
                        .editor_cache
                        .fd_references
                        .update(fd, apply_change(&references, &change))
                        .await
                    {
                        warn!("Cache error: {err}");
                        continue;
                    }
                }
            }

            // Notify all users about the change
            GLOBAL
                .ws
                .message_all(WsMessage::EditorChangedContent {
                    target: EditorTarget::FindingDefinition {
                        finding_definition: fd,
                        finding_section: fs,
                    },
                    change,
                    user,
                })
                .await;
        }
    }

    /// Process a client cursor update event
    ///
    /// The event is fanned out via websocket with
    ///
    /// If the finding definition is unknown, the event will be dropped
    pub async fn process_client_cursor_update_finding_definition(
        &self,
        user: Uuid,
        finding_definition: Uuid,
        finding_section: FindingSection,
        cursor: CursorPosition,
    ) {
        if !self.does_fd_exist(finding_definition).await {
            return;
        }

        let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
            error!("Could not retrieve user");
            return;
        };

        GLOBAL
            .ws
            .message_all(WsMessage::EditorChangedCursor {
                target: EditorTarget::FindingDefinition {
                    finding_definition,
                    finding_section,
                },
                cursor,
                user,
            })
            .await;
    }

    async fn does_fd_exist(&self, finding_definition: Uuid) -> bool {
        query!(&GLOBAL.db, (FindingDefinition::F.uuid,))
            .condition(FindingDefinition::F.uuid.equals(finding_definition))
            .optional()
            .await
            .map(|x| x.is_some())
            .unwrap_or(false)
    }
}

fn is_change_valid(change: &Change) -> bool {
    // Check change
    // Requirements:
    // - start must be before or equal to end
    change.end_line > change.start_line
        || change.start_line == change.end_line && change.end_column >= change.start_column
}

fn apply_change(given: &str, change: &Change) -> String {
    /// line and columns are utf-16 indexes (starting at 0)
    fn find_index(source: &str, mut line: usize, mut column: usize) -> usize {
        for (i, c) in source.char_indices() {
            if c == '\n' {
                if line > 0 {
                    line -= 1;
                } else {
                    return i;
                }
            } else if line == 0 {
                if column > 0 {
                    column -= c.len_utf16();
                } else {
                    return i;
                }
            }
        }
        source.len()
    }

    let text = change.text.as_str();
    let start_line = (change.start_line.get() - 1) as usize;
    let end_line = (change.end_line.get() - 1) as usize;
    let start_column = (change.start_column.get() - 1) as usize;
    let end_column = (change.end_column.get() - 1) as usize;

    let start = find_index(given, start_line, start_column);
    let end = find_index(given, end_line, end_column);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&given.as_bytes()[..start]);
    bytes.extend_from_slice(text.as_bytes());
    bytes.extend_from_slice(&given.as_bytes()[end..]);
    #[allow(clippy::expect_used)]
    String::from_utf8(bytes).expect("Indexes retrieved from char_indices")
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::num::NonZeroU64;

    use serde::Deserialize;

    use crate::chan::ws_manager::schema::Change;
    use crate::modules::editor::apply_change;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestChange<'a> {
        text: Cow<'a, str>,
        start_column: NonZeroU64,
        end_column: NonZeroU64,
        start_line: NonZeroU64,
        end_line: NonZeroU64,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestCase<'a> {
        after: Cow<'a, str>,
        before: Cow<'a, str>,
        change: TestChange<'a>,
    }

    #[test]
    fn test() {
        let tests = include_str!("finding-tests.json");
        let test_list: Vec<TestCase> = serde_json::from_str(tests).unwrap();

        for (idx, test) in test_list.iter().enumerate() {
            println!("Starting test {idx}");
            assert_eq!(
                test.after.as_ref(),
                &apply_change(
                    test.before.as_ref(),
                    &Change {
                        text: test.change.text.to_string(),
                        start_column: test.change.start_column,
                        end_column: test.change.end_column,
                        start_line: test.change.start_line,
                        end_line: test.change.end_line,
                    }
                )
            );
            println!("Finished test {idx}");
        }
    }
}
