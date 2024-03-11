//! This module provides synchronization capabilities for the editor.
//!
//! Use [EditorSync] to add events that should be processed.
//!
//! The internal synchronization will take care of fanning the events to the websocket
//! as well as caching the results internally and saving them regularly to the database

use log::debug;
use log::error;
use log::warn;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::Change;
use crate::chan::ws_manager::schema::CursorPosition;
use crate::chan::ws_manager::schema::EditorTarget;
use crate::chan::ws_manager::schema::FindingSection;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::FindingDefinition;
use crate::modules::cache::EditorCached;

/// Sync editor
pub struct EditorSync;

impl EditorSync {
    /// Process a change in workspace notes
    pub async fn process_client_edit_ws_notes(&self, user: Uuid, ws: Uuid, change: Change) {
        let existing = match GLOBAL.editor_cache.ws_notes.get(ws).await {
            Ok(ws) => {
                if let Some(ws) = ws {
                    ws
                } else {
                    debug!("Workspace does not exist");
                    return;
                }
            }

            Err(err) => {
                error!("Error gathering ws from cache: {err}");
                return;
            }
        };

        let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
            error!("Could not retrieve user from cache for active websocket");
            return;
        };

        // Check change
        // Requirements:
        // - start must be before or equal to end
        if change.end_line < change.start_line
            || change.start_line == change.end_line && change.end_column < change.start_column
        {
            warn!("Invalid change received!");
            return;
        }

        if let Err(err) = GLOBAL
            .editor_cache
            .ws_notes
            .update(ws, apply_change(&existing, &change))
            .await
        {
            error!("Cache error: {err}");
            return;
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
        &self,
        user: Uuid,
        fd: Uuid,
        finding_section: FindingSection,
        change: Change,
    ) {
        if !self.does_fd_exist(fd).await {
            return;
        }

        let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
            error!("Could not retrieve user from cache for active websocket");
            return;
        };

        // Check change
        // Requirements:
        // - start must be before or equal to end
        if change.end_line < change.start_line
            || change.start_line == change.end_line && change.end_column < change.start_column
        {
            warn!("Invalid change received!");
            return;
        }

        // Apply change
        match finding_section {
            FindingSection::Summary => {
                let Ok(Some(summary)) = GLOBAL.editor_cache.fd_summary.get(fd).await else {
                    warn!("Summary not in cache");
                    return;
                };
                if let Err(err) = GLOBAL
                    .editor_cache
                    .fd_summary
                    .update(fd, apply_change(&summary, &change))
                    .await
                {
                    warn!("Cache error: {err}");
                    return;
                }
            }
            FindingSection::Description => {
                let Ok(Some(description)) = GLOBAL.editor_cache.fd_description.get(fd).await else {
                    warn!("Description not in cache");
                    return;
                };
                if let Err(err) = GLOBAL
                    .editor_cache
                    .fd_description
                    .update(fd, apply_change(&description, &change))
                    .await
                {
                    warn!("Cache error: {err}");
                    return;
                }
            }
            FindingSection::Impact => {
                let Ok(Some(impact)) = GLOBAL.editor_cache.fd_impact.get(fd).await else {
                    warn!("Impact not in cache");
                    return;
                };
                if let Err(err) = GLOBAL
                    .editor_cache
                    .fd_impact
                    .update(fd, apply_change(&impact, &change))
                    .await
                {
                    warn!("Cache error: {err}");
                    return;
                }
            }
            FindingSection::Remediation => {
                let Ok(Some(remediation)) = GLOBAL.editor_cache.fd_remediation.get(fd).await else {
                    warn!("Remediation not in cache");
                    return;
                };
                if let Err(err) = GLOBAL
                    .editor_cache
                    .fd_remediation
                    .update(fd, apply_change(&remediation, &change))
                    .await
                {
                    warn!("Cache error: {err}");
                    return;
                }
            }
            FindingSection::References => {
                let Ok(Some(references)) = GLOBAL.editor_cache.fd_references.get(fd).await else {
                    warn!("References not in cache");
                    return;
                };
                if let Err(err) = GLOBAL
                    .editor_cache
                    .fd_references
                    .update(fd, apply_change(&references, &change))
                    .await
                {
                    warn!("Cache error: {err}");
                    return;
                }
            }
        }

        // Notify all users about the change
        GLOBAL
            .ws
            .message_all(WsMessage::EditorChangedContent {
                target: EditorTarget::FindingDefinition {
                    finding_definition: fd,
                    finding_section,
                },
                change,
                user,
            })
            .await;
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
