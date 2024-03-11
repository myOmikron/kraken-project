use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::time::Duration;

use log::error;
use log::trace;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::update;
use rorm::Error;
use rorm::FieldAccess;
use rorm::Model;
use thiserror::Error;
use tokio::fs::create_dir_all;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::models::FindingDefinition;
use crate::models::WorkspaceNotes;
use crate::models::WorkspaceNotesInsert;

const EXPECT_MSG: &str = "If you ever encounter this error, open an issues with the stacktrace";

/// The cache for editors.
///
/// Holds all available caches for editors
#[derive(Clone)]
pub struct EditorCache {
    /// Workspace notes cache
    pub ws_notes: WsNotesCache,
    /// Finding definition summary cache
    pub fd_summary: FdSummaryCache,
    /// Finding definition description cache
    pub fd_description: FdDescriptionCache,
    /// Finding definition impact cache
    pub fd_impact: FdImpactCache,
    /// Finding definition remediation cache
    pub fd_remediation: FdRemediationCache,
    /// Finding definition references cache
    pub fd_references: FdReferencesCache,
}

impl Default for EditorCache {
    /// Create a new cache for everything that is cached via editor
    fn default() -> Self {
        let cache = Self {
            ws_notes: Default::default(),
            fd_summary: Default::default(),
            fd_description: Default::default(),
            fd_impact: Default::default(),
            fd_remediation: Default::default(),
            fd_references: Default::default(),
        };

        tokio::spawn(cache.ws_notes.clone().run_cache_save());
        tokio::spawn(cache.fd_summary.clone().run_cache_save());
        tokio::spawn(cache.fd_description.clone().run_cache_save());
        tokio::spawn(cache.fd_impact.clone().run_cache_save());
        tokio::spawn(cache.fd_remediation.clone().run_cache_save());
        tokio::spawn(cache.fd_references.clone().run_cache_save());

        cache
    }
}

// --------
// FD Summary
// --------

#[derive(Clone, Default)]
pub struct FdSummaryCache(InnerCache);

impl InternalEditorCached for FdSummaryCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<String>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| x.summary))
    }

    async fn save_to_db(&self, key: Uuid, value: String) -> Result<(), rorm::Error> {
        let db = &GLOBAL.db;
        update!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .set(FindingDefinition::F.summary, value)
            .exec()
            .await?;

        Ok(())
    }
}
impl EditorCached for FdSummaryCache {}

// --------
// FD Description
// --------

#[derive(Clone, Default)]
pub struct FdDescriptionCache(InnerCache);

impl InternalEditorCached for FdDescriptionCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<String>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| x.description))
    }

    async fn save_to_db(&self, key: Uuid, value: String) -> Result<(), Error> {
        let db = &GLOBAL.db;
        update!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .set(FindingDefinition::F.description, value)
            .exec()
            .await?;

        Ok(())
    }
}
impl EditorCached for FdDescriptionCache {}

// --------
// FD Impact
// --------

#[derive(Clone, Default)]
pub struct FdImpactCache(InnerCache);

impl InternalEditorCached for FdImpactCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<String>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| x.impact))
    }

    async fn save_to_db(&self, key: Uuid, value: String) -> Result<(), Error> {
        let db = &GLOBAL.db;
        update!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .set(FindingDefinition::F.impact, value)
            .exec()
            .await?;

        Ok(())
    }
}
impl EditorCached for FdImpactCache {}

// --------
// FD Remediation
// --------

#[derive(Clone, Default)]
pub struct FdRemediationCache(InnerCache);

impl InternalEditorCached for FdRemediationCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<String>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| x.remediation))
    }

    async fn save_to_db(&self, key: Uuid, value: String) -> Result<(), Error> {
        let db = &GLOBAL.db;
        update!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .set(FindingDefinition::F.remediation, value)
            .exec()
            .await?;

        Ok(())
    }
}
impl EditorCached for FdRemediationCache {}

// --------
// FD References
// --------

#[derive(Clone, Default)]
pub struct FdReferencesCache(InnerCache);

impl InternalEditorCached for FdReferencesCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<String>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| x.references))
    }

    async fn save_to_db(&self, key: Uuid, value: String) -> Result<(), Error> {
        let db = &GLOBAL.db;
        update!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .set(FindingDefinition::F.references, value)
            .exec()
            .await?;

        Ok(())
    }
}
impl EditorCached for FdReferencesCache {}

// --------
// WS NOTES
// --------

#[derive(Clone, Default)]
pub struct WsNotesCache(InnerCache);

impl InternalEditorCached for WsNotesCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<String>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (WorkspaceNotes::F.notes,))
            .condition(WorkspaceNotes::F.workspace.equals(key))
            .order_desc(WorkspaceNotes::F.created_at)
            .optional()
            .await?
            .map(|x| x.0))
    }

    async fn save_to_db(&self, key: Uuid, value: String) -> Result<(), Error> {
        let db = &GLOBAL.db;

        insert!(db, WorkspaceNotes)
            .return_nothing()
            .single(&WorkspaceNotesInsert {
                uuid: Uuid::new_v4(),
                notes: value,
                workspace: ForeignModelByField::Key(key),
            })
            .await?;

        // TODO: Manage old workspace notes

        Ok(())
    }
}

impl EditorCached for WsNotesCache {}

// --------------
// Implementation details
// --------------
pub trait InternalEditorCached
where
    Self: Send + Sync,
{
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem>>>;

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem>>>;

    fn query_db(
        &self,
        key: Uuid,
    ) -> impl Future<Output = Result<Option<String>, rorm::Error>> + Send;

    fn save_to_db(
        &self,
        key: Uuid,
        value: String,
    ) -> impl Future<Output = Result<(), rorm::Error>> + Send;

    fn run_cache_save(self) -> impl Future<Output = ()> + Send
    where
        Self: Sized,
    {
        async move {
            let mut timer = tokio::time::interval(Duration::from_secs(30));

            let dir_path = Path::new("/var/lib/kraken/ws_notes/");

            if let Err(err) = create_dir_all(dir_path).await {
                error!("{err}");
            }

            loop {
                timer.tick().await;

                if !GLOBAL.is_initialized() {
                    trace!("Skipping cache save run as GLOBAL isn't initialized yet");
                    continue;
                }

                // Get all changed records
                let data: Vec<(Uuid, String)> = self.get_changed();

                let mut update_failed = vec![];
                let mut update_success = vec![];
                for (uuid, value) in data {
                    let res = self.save_to_db(uuid, value.clone()).await;

                    if let Err(err) = res {
                        error!("DB error when updating workspace notes: {err}");
                        update_failed.push((uuid, value))
                    } else {
                        update_success.push((uuid, value));
                    }
                }

                {
                    let mut guard = self.write_cache();
                    for (uuid, value) in update_success {
                        guard.get_mut(&uuid).and_then(|opt| {
                            opt.as_mut().map(|inner| {
                                // If the data was changed in the meantime, we shouldn't set
                                // changed to false
                                if inner.data == value {
                                    inner.changed = false
                                }
                            })
                        });
                    }
                }

                for (uuid, value) in update_failed {
                    match File::create(dir_path.join(uuid.to_string())).await {
                        Ok(mut file) => {
                            if let Err(err) = file.write_all(value.as_bytes()).await {
                                error!("{err}");
                            }
                        }
                        Err(err) => error!("{err}"),
                    }
                }
            }
        }
    }

    fn get_changed(&self) -> Vec<(Uuid, String)> {
        self.read_cache()
            .iter()
            .filter_map(|(uuid, inner)| {
                if let Some(inner) = inner {
                    if inner.changed {
                        return Some((*uuid, inner.data.clone()));
                    }
                }
                None
            })
            .collect()
    }
}

/// Trait for accessing the editor cache
pub trait EditorCached
where
    Self: InternalEditorCached,
{
    /// Retrieve an item through a key
    ///
    /// The option marks the availability of the key in the database.
    fn get(&self, key: Uuid) -> impl Future<Output = Result<Option<String>, rorm::Error>> {
        async move {
            let cache_item = self.read_cache().get(&key).cloned();

            // Check if ws notes have already been queried once
            return if let Some(item) = cache_item {
                Ok(Some(item.map(|x| x.data).unwrap_or_default()))
            } else {
                // Query the db to populate the cache
                let notes = self.query_db(key).await?;

                let Some(notes) = notes else {
                    // Update cache so it knows that there's no DB entry
                    self.write_cache().insert(key, None);
                    return Ok(None);
                };

                // If the workspace was found, insert it into the cache
                self.write_cache().insert(
                    key,
                    Some(InnerItem {
                        changed: true,
                        data: notes.clone(),
                    }),
                );

                Ok(Some(notes))
            };
        }
    }

    /// Invalidates everything marked as "Not found in DB"
    fn invalidate_not_found(&self) {
        let mut guard = self.write_cache();
        let to_remove: Vec<Uuid> = guard
            .iter()
            .filter_map(|(uuid, inner)| if inner.is_none() { Some(*uuid) } else { None })
            .collect();

        for uuid in to_remove {
            guard.remove(&uuid);
        }
    }

    /// Delete an item from the cache
    fn delete(&self, key: Uuid) {
        self.write_cache().remove(&key);
    }

    /// Update an item in the cache
    fn update(&self, key: Uuid, value: String) -> impl Future<Output = Result<(), CacheError>> {
        async move {
            // Check if ws notes have already been queried once
            if !self.read_cache().contains_key(&key) {
                self.query_db(key).await?.ok_or(CacheError::ItemNotFound)?;

                // If the item was found, insert the update in the cache
                self.write_cache().insert(
                    key,
                    Some(InnerItem {
                        changed: true,
                        data: value,
                    }),
                );

                return Ok(());
            }

            // Update the cache
            self.write_cache().insert(
                key,
                Some(InnerItem {
                    changed: true,
                    data: value,
                }),
            );

            Ok(())
        }
    }
}

/// The available error types that can be returned by the [EditorCache]
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum CacheError {
    #[error("Item was not found in database")]
    ItemNotFound,
    #[error("DB error: {0}")]
    DatabaseError(#[from] rorm::Error),
}

#[derive(Default, Clone)]
pub struct InnerItem {
    data: String,
    changed: bool,
}

type InnerCache = Arc<RwLock<HashMap<Uuid, Option<InnerItem>>>>;
