//! This module provides synchronization capabilities for the editor.
//!
//! Use [EditorSync] to add events that should be processed.
//!
//! The internal synchronization will take care of fanning the events to the websocket
//! as well as caching the results internally and saving them regularly to the database

use log::{debug, error, warn};
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::{Change, FindingSection, WsMessage};

/// Sync editor
pub struct EditorSync;

impl EditorSync {
    /// Process a client's EditFindingDefinition update
    pub async fn process_client_edit_finding_definition(
        &self,
        user: Uuid,
        finding_definition: Uuid,
        finding_section: FindingSection,
        change: Change,
    ) {
        if !self.does_fd_exist(finding_definition).await {
            return;
        }

        let Ok(Some(user)) = GLOBAL.user_cache.get_simple_user(user).await else {
            error!("Could not retrieve user from cache for active websocket");
            return;
        };

        let Ok(Some(mut fd)) = GLOBAL
            .finding_definition_cache
            .get(finding_definition)
            .await
        else {
            error!("Could not retrieve finding definition from cache");
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
            FindingSection::Summary => fd.summary = apply_change(&fd.summary, &change),
            FindingSection::Description => fd.description = apply_change(&fd.description, &change),
            FindingSection::Impact => fd.impact = apply_change(&fd.impact, &change),
            FindingSection::Remediation => fd.remediation = apply_change(&fd.remediation, &change),
            FindingSection::References => fd.references = apply_change(&fd.references, &change),
        }

        // Update cache
        if GLOBAL.finding_definition_cache.update(fd).await.is_err() {
            debug!("Finding definition was deleted while updating it");
        }

        // Notify all users about the change
        GLOBAL
            .ws
            .message_all(WsMessage::EditFindingDefinition {
                finding_definition,
                finding_section,
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
    pub async fn process_client_cursor_update(
        &self,
        user: Uuid,
        finding_definition: Uuid,
        finding_section: FindingSection,
        line: u64,
        column: u64,
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
            .message_all(WsMessage::ChangedCursorFindingDefinition {
                finding_definition,
                finding_section,
                line,
                column,
                user,
            })
            .await;
    }

    async fn does_fd_exist(&self, finding_definition: Uuid) -> bool {
        GLOBAL
            .finding_definition_cache
            .exists(finding_definition)
            .await
            .unwrap_or_else(|err| {
                error!("DB error: {err}");
                false
            })
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
