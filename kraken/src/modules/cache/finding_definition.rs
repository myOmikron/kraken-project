use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use futures::TryStreamExt;
use log::{debug, error};
use rorm::db::Executor;
use rorm::{delete, query, update, FieldAccess, Model};
use thiserror::Error;
use tokio::time::interval;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::models::{FindingDefinition, FindingDefinitionInsertError, FindingSeverity};

/// The caching layer for [FindingDefinition]
///
/// It offers capabilities to retrieve existing definitions, as well as
/// delete and create new ones.
///
/// You should not interact with [FindingDefinition] directly, but do all
/// changes through this cache.
pub struct FindingDefinitionCache {
    /// The cache for the [FindingDefinition]
    ///
    /// It maps the [Uuid] of the [FindingDefinition] to
    /// an optional [FindingDefinition].
    ///
    /// If the state is [None], a query was already tried, but resulted in no result
    cache: Arc<RwLock<HashMap<Uuid, Option<CacheFindingDefinition>>>>,
}

/// Internal helper for marking changes in cache
#[derive(Clone)]
struct CacheFindingDefinition {
    fd: FindingDefinition,
    changed: bool,
}

/// The finding definition was not found
#[derive(Error, Debug)]
#[error("Finding definition was not found")]
pub struct FindingDefinitionNotFound;

impl FindingDefinitionCache {
    /// Initialize the cache with existing findings
    pub async fn new(db: impl Executor<'_>) -> Result<Self, rorm::Error> {
        let findings = query!(db, FindingDefinition)
            .stream()
            .map_ok(|x| {
                (
                    x.uuid,
                    Some(CacheFindingDefinition {
                        fd: x,
                        changed: false,
                    }),
                )
            })
            .try_collect()
            .await?;

        Ok(Self {
            cache: Arc::new(RwLock::new(findings)),
        })
    }

    /// Retrieve a finding definition from the cache.
    ///
    /// If the finding definition was not found in the cache, a database lookup is made
    /// and the result (if some) is saved to the cache.
    pub async fn get(
        &self,
        finding_definition: Uuid,
    ) -> Result<Option<FindingDefinition>, rorm::Error> {
        if let Some(fd_cached) = self.read_cache(finding_definition) {
            return Ok(fd_cached.map(|x| x.fd));
        }

        debug!("Finding definition was not found in cache, making db lookup");
        self.query_db(finding_definition).await
    }

    /// Retrieve all available finding definitions
    pub async fn get_all(&self) -> Result<Vec<FindingDefinition>, rorm::Error> {
        #[allow(clippy::expect_used)]
        let guard = self
            .cache
            .read()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");

        Ok(guard.values().flatten().cloned().map(|x| x.fd).collect())
    }

    /// Check for the existing of a [FindingDefinition]
    pub async fn exists(&self, finding_definition: Uuid) -> Result<bool, rorm::Error> {
        if let Some(fd_cached) = self.read_cache(finding_definition) {
            return Ok(fd_cached.is_some());
        }

        debug!("Finding definition was not found in cache, making db lookup");
        Ok(self.query_db(finding_definition).await?.is_some())
    }

    /// Update a [FindingDefinition]
    ///
    /// This method will update the specified definition in the cache and schedule the
    /// update in the database.
    pub async fn update(
        &self,
        finding_definition: FindingDefinition,
    ) -> Result<(), FindingDefinitionNotFound> {
        #[allow(clippy::expect_used)]
        let mut guard = self
            .cache
            .write()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");

        if !guard.contains_key(&finding_definition.uuid) {
            return Err(FindingDefinitionNotFound);
        }

        guard.insert(
            finding_definition.uuid,
            Some(CacheFindingDefinition {
                fd: finding_definition,
                changed: true,
            }),
        );

        Ok(())
    }

    /// Deletes a [FindingDefinition]
    ///
    /// You have to check for the deleted rows in the return value to check if
    /// there were a finding definition with the given uuid
    pub async fn delete(&self, finding_definition: Uuid) -> Result<u64, rorm::Error> {
        let deleted = delete!(&GLOBAL.db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(finding_definition))
            .await?;

        // Short circuit if no finding definition was deleted
        if deleted == 0 {
            return Ok(deleted);
        }

        #[allow(clippy::expect_used)]
        let mut guard = self
            .cache
            .write()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");

        guard.remove_entry(&finding_definition);

        Ok(deleted)
    }

    /// Inserts a new [FindingDefinition] and updates the cache
    #[allow(clippy::too_many_arguments)]
    pub async fn insert(
        &self,
        name: String,
        summary: String,
        severity: FindingSeverity,
        cve: Option<String>,
        description: String,
        impact: String,
        remediation: String,
        references: String,
    ) -> Result<Uuid, FindingDefinitionInsertError> {
        let fd = FindingDefinition::insert(
            &GLOBAL.db,
            name,
            summary,
            severity,
            cve,
            description,
            impact,
            remediation,
            references,
        )
        .await?;

        let uuid = fd.uuid;
        {
            #[allow(clippy::expect_used)]
            let mut guard = self.cache.write().expect(
                "If you ever encounter this error, please open an issue with the stacktrace",
            );

            guard.insert(uuid, Some(CacheFindingDefinition { fd, changed: false }));
        }

        Ok(uuid)
    }

    /// This method is used to save the cache to the database.
    ///
    /// It should only be used by the scheduler for regularly saving the cache to the database
    async fn save_cache(&self) -> Result<(), rorm::Error> {
        let fds = {
            #[allow(clippy::expect_used)]
            let mut guard = self.cache.write().expect(
                "If you ever encounter this error, please open an issue with the stacktrace",
            );

            let fds = guard
                .values()
                .flatten()
                .filter(|x| x.changed)
                .map(|x| &x.fd)
                .cloned()
                .collect::<Vec<_>>();

            guard.values_mut().flatten().for_each(|x| x.changed = false);

            fds
        };

        let mut tx = GLOBAL.db.start_transaction().await?;

        for fd in fds {
            update!(&mut tx, FindingDefinition)
                .condition(FindingDefinition::F.uuid.equals(fd.uuid))
                .set(FindingDefinition::F.name, fd.name)
                .set(FindingDefinition::F.cve, fd.cve)
                .set(FindingDefinition::F.severity, fd.severity)
                .set(FindingDefinition::F.summary, fd.summary)
                .set(FindingDefinition::F.description, fd.description)
                .set(FindingDefinition::F.impact, fd.impact)
                .set(FindingDefinition::F.remediation, fd.remediation)
                .set(FindingDefinition::F.references, fd.references)
                .exec()
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Reads the cache
    ///
    /// If the outer [Option] is [None], the database wasn't queried yet for this entry
    /// If the inner [Option] is [None], the database was queried, but no entry was found
    fn read_cache(&self, finding_definition: Uuid) -> Option<Option<CacheFindingDefinition>> {
        #[allow(clippy::expect_used)]
        let guard = self
            .cache
            .read()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");
        guard.get(&finding_definition).map(|x| x.to_owned())
    }

    /// Can be used to update the caching state.
    ///
    /// If no result with the given [Uuid] was found, [None] is inserted
    /// into the cache.
    async fn query_db(
        &self,
        finding_definition: Uuid,
    ) -> Result<Option<FindingDefinition>, rorm::Error> {
        let db = &GLOBAL.db;
        let fd_db = query!(db, FindingDefinition)
            .condition(FindingDefinition::F.uuid.equals(finding_definition))
            .optional()
            .await?;

        #[allow(clippy::expect_used)]
        let mut guard = self
            .cache
            .write()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");
        if let Some(fd) = &fd_db {
            debug!("Finding definition was found in db, inserting in cache");
            guard.insert(
                finding_definition,
                Some(CacheFindingDefinition {
                    fd: fd.clone(),
                    changed: false,
                }),
            );
        } else {
            debug!("Finding definition was not found in db, inserting in cache");
            guard.insert(finding_definition, None);
        }

        Ok(fd_db)
    }
}

/// Schedule the cache save
pub async fn schedule_cache_save(itvl: Duration) {
    let mut timer = interval(itvl);
    loop {
        if let Err(err) = GLOBAL.finding_definition_cache.save_cache().await {
            error!("Error saving fd cache: {err}");
        }
        timer.tick().await;
    }
}
