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
use crate::models::Finding;
use crate::models::FindingAffected;
use crate::models::FindingDefinition;
use crate::models::FindingDetails;
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
    /// Finding details cache
    pub finding_details: FindingDetailsCache,
    /// FindingAffected details cache
    pub finding_affected_details: FindingAffectedDetailsCache,
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
            finding_details: Default::default(),
            finding_affected_details: Default::default(),
        };

        tokio::spawn(cache.ws_notes.clone().run_cache_save());
        tokio::spawn(cache.fd_summary.clone().run_cache_save());
        tokio::spawn(cache.fd_description.clone().run_cache_save());
        tokio::spawn(cache.fd_impact.clone().run_cache_save());
        tokio::spawn(cache.fd_remediation.clone().run_cache_save());
        tokio::spawn(cache.fd_references.clone().run_cache_save());
        tokio::spawn(cache.finding_details.clone().run_cache_save());
        tokio::spawn(cache.finding_affected_details.clone().run_cache_save());

        cache
    }
}

// --------
// FindingAffected Details
// --------

#[derive(Clone, Default)]
pub struct FindingAffectedDetailsCache(InnerCache<Uuid>);

impl InternalEditorCached<Uuid> for FindingAffectedDetailsCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem<Uuid>>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem<Uuid>>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<(String, Uuid)>, Error> {
        let db = &GLOBAL.db;

        Ok(query!(
            db,
            (
                FindingAffected::F.details.user_details,
                FindingAffected::F.workspace
            )
        )
        .condition(FindingAffected::F.uuid.equals(key))
        .optional()
        .await?
        .map(|x| (x.0, *x.1.key())))
    }

    async fn save_to_db(&self, key: Uuid, value: String) -> Result<(), Error> {
        let mut tx = GLOBAL.db.start_transaction().await?;

        let old_details = query!(&mut tx, (FindingAffected::F.details,))
            .condition(FindingAffected::F.uuid.equals(key))
            .one()
            .await?
            .0
            .map(|x| *x.key());

        if let Some(old_details) = old_details {
            update!(&mut tx, FindingDetails)
                .condition(FindingDetails::F.uuid.equals(old_details))
                .set(FindingDetails::F.user_details, value)
                .exec()
                .await?;
        } else {
            let pk = insert!(&mut tx, FindingDetails)
                .return_primary_key()
                .single(&FindingDetails {
                    uuid: Uuid::new_v4(),
                    user_details: value,
                    tool_details: None,
                    screenshot: None,
                    log_file: None,
                })
                .await?;

            update!(&mut tx, FindingAffected)
                .condition(FindingAffected::F.uuid.equals(key))
                .set(
                    FindingAffected::F.details,
                    Some(ForeignModelByField::Key(pk)),
                )
                .exec()
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

impl EditorCached<Uuid> for FindingAffectedDetailsCache {}

// --------
// Finding Details
// --------

#[derive(Clone, Default)]
pub struct FindingDetailsCache(InnerCache<Uuid>);

impl InternalEditorCached<Uuid> for FindingDetailsCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem<Uuid>>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem<Uuid>>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<(String, Uuid)>, Error> {
        let db = &GLOBAL.db;

        query!(db, (Finding::F.details.user_details, Finding::F.workspace))
            .condition(Finding::F.uuid.equals(key))
            .optional()
            .await
            .map(|x| x.map(|y| (y.0, *y.1.key())))
    }

    async fn save_to_db(&self, key: Uuid, value: String) -> Result<(), Error> {
        let mut tx = GLOBAL.db.start_transaction().await?;

        let details_uuid = query!(&mut tx, (Finding::F.details,))
            .condition(Finding::F.uuid.equals(key))
            .one()
            .await?
            .0;

        update!(&mut tx, FindingDetails)
            .condition(FindingDetails::F.uuid.equals(*details_uuid.key()))
            .set(FindingDetails::F.user_details, value)
            .exec()
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

impl EditorCached<Uuid> for FindingDetailsCache {}

// --------
// FD Summary
// --------

#[derive(Clone, Default)]
pub struct FdSummaryCache(InnerCache<()>);

impl InternalEditorCached<()> for FdSummaryCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem<()>>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem<()>>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<(String, ())>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (FindingDefinition::F.summary,))
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| (x.0, ())))
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
impl EditorCached<()> for FdSummaryCache {}

// --------
// FD Description
// --------

#[derive(Clone, Default)]
pub struct FdDescriptionCache(InnerCache<()>);

impl InternalEditorCached<()> for FdDescriptionCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem<()>>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem<()>>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<(String, ())>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (FindingDefinition::F.description,))
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| (x.0, ())))
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
impl EditorCached<()> for FdDescriptionCache {}

// --------
// FD Impact
// --------

#[derive(Clone, Default)]
pub struct FdImpactCache(InnerCache<()>);

impl InternalEditorCached<()> for FdImpactCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem<()>>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem<()>>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<(String, ())>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (FindingDefinition::F.impact,))
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| (x.0, ())))
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
impl EditorCached<()> for FdImpactCache {}

// --------
// FD Remediation
// --------

#[derive(Clone, Default)]
pub struct FdRemediationCache(InnerCache<()>);

impl InternalEditorCached<()> for FdRemediationCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem<()>>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem<()>>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<(String, ())>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (FindingDefinition::F.remediation,))
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| (x.0, ())))
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
impl EditorCached<()> for FdRemediationCache {}

// --------
// FD References
// --------

#[derive(Clone, Default)]
pub struct FdReferencesCache(InnerCache<()>);

impl InternalEditorCached<()> for FdReferencesCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem<()>>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem<()>>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<(String, ())>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (FindingDefinition::F.references,))
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| (x.0, ())))
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
impl EditorCached<()> for FdReferencesCache {}

// --------
// WS NOTES
// --------

#[derive(Clone, Default)]
pub struct WsNotesCache(InnerCache<Uuid>);

impl InternalEditorCached<Uuid> for WsNotesCache {
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem<Uuid>>>> {
        #[allow(clippy::expect_used)]
        self.0.read().expect(EXPECT_MSG)
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem<Uuid>>>> {
        #[allow(clippy::expect_used)]
        self.0.write().expect(EXPECT_MSG)
    }

    async fn query_db(&self, key: Uuid) -> Result<Option<(String, Uuid)>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (WorkspaceNotes::F.notes,))
            .condition(WorkspaceNotes::F.workspace.equals(key))
            .order_desc(WorkspaceNotes::F.created_at)
            .optional()
            .await?
            .map(|x| (x.0, key)))
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

impl EditorCached<Uuid> for WsNotesCache {}

// --------------
// Implementation details
// --------------
pub trait InternalEditorCached<WS>
where
    WS: Send,
    Self: Send + Sync,
{
    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Uuid, Option<InnerItem<WS>>>>;

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, Option<InnerItem<WS>>>>;

    fn query_db(
        &self,
        key: Uuid,
    ) -> impl Future<Output = Result<Option<(String, WS)>, rorm::Error>> + Send;

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
pub trait EditorCached<WS>
where
    Self: InternalEditorCached<WS>,
    WS: Copy + Default + Send,
{
    /// Retrieve an item through a key
    ///
    /// The option marks the availability of the key in the database.
    fn get(
        &self,
        key: Uuid,
    ) -> impl Future<Output = Result<Option<(String, WS)>, rorm::Error>> + Send {
        async move {
            let cache_item = self.read_cache().get(&key).cloned();

            // Check if ws notes have already been queried once
            return if let Some(item) = cache_item {
                Ok(Some(item.map(|x| (x.data, x.ws)).unwrap_or_default()))
            } else {
                // Query the db to populate the cache
                let notes = self.query_db(key).await?;

                let Some((notes, ws)) = notes else {
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
                        ws,
                    }),
                );

                Ok(Some((notes, ws)))
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
    fn update(
        &self,
        key: Uuid,
        value: String,
    ) -> impl Future<Output = Result<(), CacheError>> + Send {
        async move {
            // Check if ws notes have already been queried once
            let item = self.read_cache().get(&key).cloned();

            match item {
                None => {
                    let (_, ws) = self.query_db(key).await?.ok_or(CacheError::ItemNotFound)?;

                    // If the item was found, insert the update in the cache
                    self.write_cache().insert(
                        key,
                        Some(InnerItem {
                            changed: true,
                            data: value,
                            ws,
                        }),
                    );

                    Ok(())
                }
                Some(old) => {
                    let item = if let Some(data) = old {
                        InnerItem {
                            changed: true,
                            data: value,
                            ws: data.ws,
                        }
                    } else {
                        let (_, ws) = self.query_db(key).await?.ok_or(CacheError::ItemNotFound)?;
                        InnerItem {
                            changed: true,
                            data: value,
                            ws,
                        }
                    };

                    self.write_cache().insert(key, Some(item));

                    Ok(())
                }
            }
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
pub struct InnerItem<WS>
where
    WS: Send,
{
    data: String,
    changed: bool,
    ws: WS,
}

type InnerCache<WS> = Arc<RwLock<HashMap<Uuid, Option<InnerItem<WS>>>>>;
