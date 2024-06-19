use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hash;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::time::Duration;

use log::error;
use log::trace;
use rorm::insert;
use rorm::or;
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
pub struct EditorCaches {
    /// Workspace notes cache
    pub ws_notes: EditorCache<WsNotes>,
    /// Finding definition summary cache
    pub fd_summary: EditorCache<FdSummary>,
    /// Finding definition description cache
    pub fd_description: EditorCache<FdDescription>,
    /// Finding definition impact cache
    pub fd_impact: EditorCache<FdImpact>,
    /// Finding definition remediation cache
    pub fd_remediation: EditorCache<FdRemediation>,
    /// Finding definition references cache
    pub fd_references: EditorCache<FdReferences>,
    /// Finding export details cache
    pub finding_export_details: EditorCache<FindingExportDetails>,
    /// FindingAffected export details cache
    pub finding_affected_export_details: EditorCache<FindingAffectedExportDetails>,
    /// Finding user details cache
    pub finding_user_details: EditorCache<FindingUserDetails>,
    /// FindingAffected user details cache
    pub finding_affected_user_details: EditorCache<FindingAffectedUserDetails>,
}

impl Default for EditorCaches {
    /// Create a new cache for everything that is cached via editor
    fn default() -> Self {
        let cache = Self {
            ws_notes: Default::default(),
            fd_summary: Default::default(),
            fd_description: Default::default(),
            fd_impact: Default::default(),
            fd_remediation: Default::default(),
            fd_references: Default::default(),
            finding_export_details: Default::default(),
            finding_affected_export_details: Default::default(),
            finding_user_details: Default::default(),
            finding_affected_user_details: Default::default(),
        };

        tokio::spawn(cache.ws_notes.clone().run_cache_save());
        tokio::spawn(cache.fd_summary.clone().run_cache_save());
        tokio::spawn(cache.fd_description.clone().run_cache_save());
        tokio::spawn(cache.fd_impact.clone().run_cache_save());
        tokio::spawn(cache.fd_remediation.clone().run_cache_save());
        tokio::spawn(cache.fd_references.clone().run_cache_save());
        tokio::spawn(cache.finding_export_details.clone().run_cache_save());
        tokio::spawn(
            cache
                .finding_affected_export_details
                .clone()
                .run_cache_save(),
        );
        tokio::spawn(cache.finding_user_details.clone().run_cache_save());
        tokio::spawn(cache.finding_affected_user_details.clone().run_cache_save());

        cache
    }
}

// --------
// FindingAffected Export Details
// --------

pub enum FindingAffectedExportDetails {}
impl EditorCacheImpl for FindingAffectedExportDetails {
    type Key = Uuid;
    type Workspace = Uuid;
    async fn query_db(key: Uuid) -> Result<Option<(String, Uuid)>, Error> {
        let mut tx = GLOBAL.db.start_transaction().await?;

        let res = if let Some((details, workspace)) = query!(
            &mut tx,
            (FindingAffected::F.details, FindingAffected::F.workspace)
        )
        .condition(or!(
            FindingAffected::F.domain.equals(key),
            FindingAffected::F.host.equals(key),
            FindingAffected::F.port.equals(key),
            FindingAffected::F.service.equals(key),
            FindingAffected::F.http_service.equals(key)
        ))
        .optional()
        .await?
        .map(|x| (x.0.map(|y| *y.key()), *x.1.key()))
        {
            if let Some(details) = details {
                let export_details = query!(&mut tx, (FindingDetails::F.export_details,))
                    .condition(FindingDetails::F.uuid.equals(details))
                    .one()
                    .await?
                    .0;
                Ok(Some((export_details, workspace)))
            } else {
                Ok(Some((String::new(), workspace)))
            }
        } else {
            Ok(None)
        };

        tx.commit().await?;

        res
    }

    async fn save_to_db(key: Uuid, value: String) -> Result<(), Error> {
        let mut tx = GLOBAL.db.start_transaction().await?;

        let old_details = query!(&mut tx, (FindingAffected::F.details,))
            .condition(or!(
                FindingAffected::F.domain.equals(key),
                FindingAffected::F.host.equals(key),
                FindingAffected::F.port.equals(key),
                FindingAffected::F.service.equals(key),
                FindingAffected::F.http_service.equals(key)
            ))
            .one()
            .await?
            .0
            .map(|x| *x.key());

        if let Some(old_details) = old_details {
            update!(&mut tx, FindingDetails)
                .condition(FindingDetails::F.uuid.equals(old_details))
                .set(FindingDetails::F.export_details, value)
                .exec()
                .await?;
        } else {
            let pk = insert!(&mut tx, FindingDetails)
                .return_primary_key()
                .single(&FindingDetails {
                    uuid: Uuid::new_v4(),
                    export_details: value,
                    user_details: String::new(),
                    tool_details: None,
                    screenshot: None,
                    log_file: None,
                })
                .await?;

            update!(&mut tx, FindingAffected)
                .condition(or!(
                    FindingAffected::F.domain.equals(key),
                    FindingAffected::F.host.equals(key),
                    FindingAffected::F.port.equals(key),
                    FindingAffected::F.service.equals(key),
                    FindingAffected::F.http_service.equals(key)
                ))
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

    fn file_name(key: Self::Key) -> String {
        format!("finding_affected_export_details_{key}")
    }
}

// --------
// Finding User Details
// --------

pub enum FindingExportDetails {}
impl EditorCacheImpl for FindingExportDetails {
    type Key = Uuid;
    type Workspace = Uuid;
    async fn query_db(key: Uuid) -> Result<Option<(String, Uuid)>, Error> {
        let db = &GLOBAL.db;

        query!(
            db,
            (Finding::F.details.export_details, Finding::F.workspace)
        )
        .condition(Finding::F.uuid.equals(key))
        .optional()
        .await
        .map(|x| x.map(|y| (y.0, *y.1.key())))
    }

    async fn save_to_db(key: Uuid, value: String) -> Result<(), Error> {
        let mut tx = GLOBAL.db.start_transaction().await?;

        let details_uuid = query!(&mut tx, (Finding::F.details,))
            .condition(Finding::F.uuid.equals(key))
            .one()
            .await?
            .0;

        update!(&mut tx, FindingDetails)
            .condition(FindingDetails::F.uuid.equals(*details_uuid.key()))
            .set(FindingDetails::F.export_details, value)
            .exec()
            .await?;

        tx.commit().await?;

        Ok(())
    }

    fn file_name(key: Self::Key) -> String {
        format!("finding_export_details_{key}")
    }
}

// --------
// FindingAffected User Details
// --------
pub enum FindingAffectedUserDetails {}
impl EditorCacheImpl for FindingAffectedUserDetails {
    type Key = Uuid;
    type Workspace = Uuid;
    async fn query_db(key: Uuid) -> Result<Option<(String, Uuid)>, Error> {
        let mut tx = GLOBAL.db.start_transaction().await?;

        let res = if let Some((details, workspace)) = query!(
            &mut tx,
            (FindingAffected::F.details, FindingAffected::F.workspace)
        )
        .condition(or!(
            FindingAffected::F.domain.equals(key),
            FindingAffected::F.host.equals(key),
            FindingAffected::F.port.equals(key),
            FindingAffected::F.service.equals(key),
            FindingAffected::F.http_service.equals(key)
        ))
        .optional()
        .await?
        .map(|x| (x.0.map(|y| *y.key()), *x.1.key()))
        {
            if let Some(details) = details {
                let user_details = query!(&mut tx, (FindingDetails::F.user_details,))
                    .condition(FindingDetails::F.uuid.equals(details))
                    .one()
                    .await?
                    .0;
                Ok(Some((user_details, workspace)))
            } else {
                Ok(Some((String::new(), workspace)))
            }
        } else {
            Ok(None)
        };

        tx.commit().await?;

        res
    }

    async fn save_to_db(key: Uuid, value: String) -> Result<(), Error> {
        let mut tx = GLOBAL.db.start_transaction().await?;

        let old_details = query!(&mut tx, (FindingAffected::F.details,))
            .condition(or!(
                FindingAffected::F.domain.equals(key),
                FindingAffected::F.host.equals(key),
                FindingAffected::F.port.equals(key),
                FindingAffected::F.service.equals(key),
                FindingAffected::F.http_service.equals(key)
            ))
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
                    export_details: String::new(),
                    user_details: value,
                    tool_details: None,
                    screenshot: None,
                    log_file: None,
                })
                .await?;

            update!(&mut tx, FindingAffected)
                .condition(or!(
                    FindingAffected::F.domain.equals(key),
                    FindingAffected::F.host.equals(key),
                    FindingAffected::F.port.equals(key),
                    FindingAffected::F.service.equals(key),
                    FindingAffected::F.http_service.equals(key)
                ))
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

    fn file_name(key: Self::Key) -> String {
        format!("finding_affected_user_details_{key}")
    }
}

// --------
// Finding User Details
// --------

pub enum FindingUserDetails {}
impl EditorCacheImpl for FindingUserDetails {
    type Key = Uuid;
    type Workspace = Uuid;
    async fn query_db(key: Uuid) -> Result<Option<(String, Uuid)>, Error> {
        let db = &GLOBAL.db;

        query!(db, (Finding::F.details.user_details, Finding::F.workspace))
            .condition(Finding::F.uuid.equals(key))
            .optional()
            .await
            .map(|x| x.map(|y| (y.0, *y.1.key())))
    }

    async fn save_to_db(key: Uuid, value: String) -> Result<(), Error> {
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

    fn file_name(key: Self::Key) -> String {
        format!("finding_user_details_{key}")
    }
}

// --------
// FD Summary
// --------

pub enum FdSummary {}

impl EditorCacheImpl for FdSummary {
    type Key = Uuid;
    type Workspace = ();
    async fn query_db(key: Uuid) -> Result<Option<(String, ())>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (FindingDefinition::F.summary,))
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| (x.0, ())))
    }

    async fn save_to_db(key: Uuid, value: String) -> Result<(), rorm::Error> {
        let db = &GLOBAL.db;
        update!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .set(FindingDefinition::F.summary, value)
            .exec()
            .await?;

        Ok(())
    }

    fn file_name(key: Self::Key) -> String {
        format!("fd_summary_{key}")
    }
}

// --------
// FD Description
// --------

pub enum FdDescription {}

impl EditorCacheImpl for FdDescription {
    type Key = Uuid;
    type Workspace = ();
    async fn query_db(key: Uuid) -> Result<Option<(String, ())>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (FindingDefinition::F.description,))
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| (x.0, ())))
    }

    async fn save_to_db(key: Uuid, value: String) -> Result<(), Error> {
        let db = &GLOBAL.db;
        update!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .set(FindingDefinition::F.description, value)
            .exec()
            .await?;

        Ok(())
    }

    fn file_name(key: Self::Key) -> String {
        format!("fd_description_{key}")
    }
}
// --------
// FD Impact
// --------

pub enum FdImpact {}

impl EditorCacheImpl for FdImpact {
    type Key = Uuid;
    type Workspace = ();
    async fn query_db(key: Uuid) -> Result<Option<(String, ())>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (FindingDefinition::F.impact,))
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| (x.0, ())))
    }

    async fn save_to_db(key: Uuid, value: String) -> Result<(), Error> {
        let db = &GLOBAL.db;
        update!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .set(FindingDefinition::F.impact, value)
            .exec()
            .await?;

        Ok(())
    }

    fn file_name(key: Self::Key) -> String {
        format!("fd_impact_{key}")
    }
}

// --------
// FD Remediation
// --------

pub enum FdRemediation {}

impl EditorCacheImpl for FdRemediation {
    type Key = Uuid;
    type Workspace = ();
    async fn query_db(key: Uuid) -> Result<Option<(String, ())>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (FindingDefinition::F.remediation,))
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| (x.0, ())))
    }

    async fn save_to_db(key: Uuid, value: String) -> Result<(), Error> {
        let db = &GLOBAL.db;
        update!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .set(FindingDefinition::F.remediation, value)
            .exec()
            .await?;

        Ok(())
    }

    fn file_name(key: Self::Key) -> String {
        format!("fd_remediation_{key}")
    }
}

// --------
// FD References
// --------

pub enum FdReferences {}

impl EditorCacheImpl for FdReferences {
    type Key = Uuid;
    type Workspace = ();
    async fn query_db(key: Uuid) -> Result<Option<(String, ())>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (FindingDefinition::F.references,))
            .condition(FindingDefinition::F.uuid.equals(key))
            .optional()
            .await?
            .map(|x| (x.0, ())))
    }

    async fn save_to_db(key: Uuid, value: String) -> Result<(), Error> {
        let db = &GLOBAL.db;
        update!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(key))
            .set(FindingDefinition::F.references, value)
            .exec()
            .await?;

        Ok(())
    }

    fn file_name(key: Self::Key) -> String {
        format!("fd_references_{key}")
    }
}

// --------
// WS NOTES
// --------

#[derive(Clone, Default)]
pub struct WsNotes {}

impl EditorCacheImpl for WsNotes {
    type Key = Uuid;
    type Workspace = Uuid;
    async fn query_db(key: Uuid) -> Result<Option<(String, Uuid)>, Error> {
        let db = &GLOBAL.db;
        Ok(query!(db, (WorkspaceNotes::F.notes,))
            .condition(WorkspaceNotes::F.workspace.equals(key))
            .order_desc(WorkspaceNotes::F.created_at)
            .optional()
            .await?
            .map(|x| (x.0, key)))
    }

    async fn save_to_db(key: Uuid, value: String) -> Result<(), Error> {
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

    fn file_name(key: Self::Key) -> String {
        format!("ws_notes_{key}")
    }
}

// --------------
// Implementation details
// --------------

pub struct EditorCache<Impl: EditorCacheImpl> {
    /// HashMap used to store cached entries
    ///
    /// Each entry can be in one of for states:
    /// - *key not in map*: the entry wasn't queried yet
    /// - *value is `None`*: the entry was queried but doesn't exist in the database
    /// - *`changed` is `false`*: the entry was found and the cache's state matches the database
    /// - *`changed` is `true`*: the entry was found but the cache's state is ahead of the database
    inner: Arc<RwLock<HashMap<Impl::Key, Option<InnerItem<Impl::Workspace>>>>>,
}

#[derive(Default, Clone)]
struct InnerItem<Workspace> {
    /// The actual cached data
    data: String,

    /// Flag whether `data` matches or is ahead of the database
    changed: bool,

    /// The workspace this item belongs to.
    ///
    /// The generic parameter `Workspace` will either be a `Uuid` or `()`
    /// depending on whether the type of item is associated with a workspace or not.
    workspace: Workspace,
}

/// The available error types that can be returned by the [`EditorCache`]
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum CacheError {
    #[error("Item was not found in database")]
    ItemNotFound,
    #[error("DB error: {0}")]
    DatabaseError(#[from] rorm::Error),
}

/// The implementation details of an [`EditorCache`]
/// which are specific to the database model (and field) being cached.
///
/// The type implementing this trait is just a marker.
/// It is recommended to use an empty enum to ensure the type can't be used for anything else by accident.
pub trait EditorCacheImpl {
    /// The key used in the `EditorCache`'s hashmap.
    ///
    /// This has to uniquely identify the cached items in the database.
    type Key: Eq + Hash + Copy + Send + Sync;

    /// Either `Uuid` if the cached items are associated with a workspace or `()` otherwise.
    type Workspace: Copy + Send + Sync;

    /// Query a single item
    fn query_db(
        key: Self::Key,
    ) -> impl Future<Output = Result<Option<(String, Self::Workspace)>, rorm::Error>> + Send;

    /// Save a single item
    fn save_to_db(
        key: Self::Key,
        value: String,
    ) -> impl Future<Output = Result<(), rorm::Error>> + Send;

    /// Get the file to write cached value to in case of db error
    ///
    /// The cache tries to save its pending changes in regular intervals to the database.
    /// If it fails to do so due to a database error, it will try to write the failed
    /// items to disk to avoid loss of data.
    /// They will be located under `/var/lib/kraken/editor_cache_failures/`.
    ///
    /// This function should return a file name uniquely identifying the cached item.
    fn file_name(key: Self::Key) -> String;
}

impl<Impl: EditorCacheImpl> EditorCache<Impl> {
    /// Retrieve an item through a key
    ///
    /// The option marks the availability of the key in the database.
    pub fn get(
        &self,
        key: Impl::Key,
    ) -> impl Future<Output = Result<Option<(String, Impl::Workspace)>, rorm::Error>> + Send + '_
    {
        async move {
            let cache_item = self.read_cache().get(&key).cloned();

            // Check if ws notes have already been queried once
            return if let Some(item) = cache_item {
                Ok(item.map(|x| (x.data, x.workspace)))
            } else {
                // Query the db to populate the cache
                let notes = Impl::query_db(key).await?;

                let Some((notes, workspace)) = notes else {
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
                        workspace,
                    }),
                );

                Ok(Some((notes, workspace)))
            };
        }
    }

    /// Invalidates everything marked as "Not found in DB"
    pub fn invalidate_not_found(&self) {
        let mut guard = self.write_cache();
        let to_remove: Vec<Impl::Key> = guard
            .iter()
            .filter_map(|(uuid, inner)| if inner.is_none() { Some(*uuid) } else { None })
            .collect();

        for uuid in to_remove {
            guard.remove(&uuid);
        }
    }

    /// Delete an item from the cache
    pub fn delete(&self, key: Impl::Key) {
        self.write_cache().remove(&key);
    }

    /// Update an item in the cache
    pub fn update(
        &self,
        key: Impl::Key,
        value: String,
    ) -> impl Future<Output = Result<(), CacheError>> + Send + '_ {
        async move {
            // Check if ws notes have already been queried once
            let item = self.read_cache().get(&key).cloned();

            match item {
                None => {
                    let (_, workspace) =
                        Impl::query_db(key).await?.ok_or(CacheError::ItemNotFound)?;

                    // If the item was found, insert the update in the cache
                    self.write_cache().insert(
                        key,
                        Some(InnerItem {
                            changed: true,
                            data: value,
                            workspace,
                        }),
                    );

                    Ok(())
                }
                Some(old) => {
                    let item = if let Some(data) = old {
                        InnerItem {
                            changed: true,
                            data: value,
                            workspace: data.workspace,
                        }
                    } else {
                        let (_, workspace) =
                            Impl::query_db(key).await?.ok_or(CacheError::ItemNotFound)?;
                        InnerItem {
                            changed: true,
                            data: value,
                            workspace,
                        }
                    };

                    self.write_cache().insert(key, Some(item));

                    Ok(())
                }
            }
        }
    }

    /// Saves the cache in regular intervals to the database
    ///
    /// This is an infinite loop and should be run as background task
    pub fn run_cache_save(self) -> impl Future<Output = Never> + Send
    where
        Self: Sized,
    {
        async move {
            let mut timer = tokio::time::interval(Duration::from_secs(30));

            let dir_path = Path::new("/var/lib/kraken/editor_cache_failures/");

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
                let data: Vec<(Impl::Key, String)> = self
                    .read_cache()
                    .iter()
                    .filter_map(|(uuid, inner)| match inner {
                        Some(inner) if inner.changed => Some((*uuid, inner.data.clone())),
                        _ => None,
                    })
                    .collect();

                let mut update_failed = vec![];
                let mut update_success = vec![];
                for (uuid, value) in data {
                    let res = Impl::save_to_db(uuid, value.clone()).await;

                    if let Err(err) = res {
                        error!("DB error when updating workspace notes: {err}");
                        update_failed.push((uuid, value))
                    } else {
                        update_success.push((uuid, value));
                    }
                }

                {
                    let mut guard = self.write_cache();
                    for (key, value) in update_success {
                        guard.get_mut(&key).and_then(|opt| {
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

                for (key, value) in update_failed {
                    match File::create(dir_path.join(Impl::file_name(key))).await {
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

    fn read_cache(
        &self,
    ) -> RwLockReadGuard<'_, HashMap<Impl::Key, Option<InnerItem<Impl::Workspace>>>> {
        #[allow(clippy::expect_used)]
        self.inner.read().expect(EXPECT_MSG)
    }

    fn write_cache(
        &self,
    ) -> RwLockWriteGuard<'_, HashMap<Impl::Key, Option<InnerItem<Impl::Workspace>>>> {
        #[allow(clippy::expect_used)]
        self.inner.write().expect(EXPECT_MSG)
    }
}
impl<Impl: EditorCacheImpl> Clone for EditorCache<Impl> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<Impl: EditorCacheImpl> Default for EditorCache<Impl> {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
/// Type which can't be created
///
/// Replace with `!` once its stable
pub enum Never {}
