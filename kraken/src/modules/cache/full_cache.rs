//! This module holds types for building caches

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

use log::debug;
use log::error;
use rorm::db::transaction::Transaction;
use rorm::internal::field::Field;
use rorm::internal::field::SingleColumnField;
use rorm::model::Identifiable;
use rorm::update;
use rorm::Error;
use rorm::FieldAccess;
use rorm::Model;
use rorm::Patch;
use thiserror::Error;
use tokio::time::interval;
use uuid::Uuid;

use crate::chan::global::GLOBAL;
use crate::models::FindingDefinition;
use crate::models::Workspace;

impl CacheDBControl for Workspace {
    fn update_item_db(
        item: Self,
        tx: &mut Transaction,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            update!(tx, Self)
                .condition(Self::F.uuid.equals(item.uuid))
                .set(Self::F.notes, item.notes)
                .exec()
                .await
                .map(|_| ())
        }
    }
}

impl CacheDBControl for FindingDefinition {
    fn update_item_db(
        item: Self,
        tx: &mut Transaction,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            update!(tx, FindingDefinition)
                .condition(FindingDefinition::F.uuid.equals(item.uuid))
                .set(FindingDefinition::F.name, item.name)
                .set(FindingDefinition::F.cve, item.cve)
                .set(FindingDefinition::F.severity, item.severity)
                .set(FindingDefinition::F.summary, item.summary)
                .set(FindingDefinition::F.description, item.description)
                .set(FindingDefinition::F.impact, item.impact)
                .set(FindingDefinition::F.remediation, item.remediation)
                .set(FindingDefinition::F.references, item.references)
                .exec()
                .await
                .map(|_| ())
        }
    }
}

// -----------------------------------
// Cache Implementation
// -----------------------------------

/// A generic struct for implementing caches.
///
/// Implement [RwCache] for a specific type on this type to create a new cache.
#[derive(Clone)]
pub struct FullCache<M> {
    cache: Arc<RwLock<InnerCache<M>>>,
}

type InnerCache<M> = HashMap<Uuid, Option<CacheItem<M>>>;

#[derive(Clone)]
struct CacheItem<M> {
    item: M,
    changed: bool,
}

/// The database control for a model
///
/// This is used by [RwCache] to be able to default-implement all high-level methods.
/// To create a new cache, implement these methods for your specific model.
pub trait CacheDBControl
where
    Self: Model + Identifiable + Send,
    <Self as rorm::Model>::Primary: Field<Type = Uuid> + SingleColumnField,
    <Self as rorm::Patch>::Decoder: Send,
{
    /// Delete a specific item in the database
    fn delete_item_db(identifier: Uuid) -> impl Future<Output = Result<u64, rorm::Error>> + Send {
        async move {
            rorm::delete!(&GLOBAL.db, Self)
                .condition(Self::Primary::type_as_value(&identifier))
                .await
        }
    }

    /// Retrieve a specific item in the database
    fn get_item_db(
        identifier: Uuid,
    ) -> impl Future<Output = Result<Option<Self>, rorm::Error>> + Send {
        async move {
            rorm::query!(&GLOBAL.db, Self)
                .condition(Self::Primary::type_as_value(&identifier))
                .optional()
                .await
        }
    }

    /// Insert a specific item in the database
    fn insert_item_db<P>(item: P) -> impl Future<Output = Result<Self, rorm::Error>> + Send
    where
        P: Sync + Send,
        P: Patch<Model = Self>,
    {
        async move { rorm::insert!(&GLOBAL.db, Self).single(&item).await }
    }

    /// Take the item and update it in the database using
    /// this provided [Transaction].
    fn update_item_db(
        item: Self,
        tx: &mut Transaction,
    ) -> impl Future<Output = Result<(), rorm::Error>> + Send;
}

/// The Model was not found
#[derive(Error, Debug)]
#[error("Item was not found")]
pub struct ItemNotInCacheError;

impl<M> FullCache<M>
where
    M: Clone + Model + Identifiable + Send + Sync + CacheDBControl,
    <M as rorm::Model>::Primary: Field<Type = Uuid> + SingleColumnField,
    <M as rorm::Patch>::Decoder: Send,
{
    /// Retrieve a Model from the cache.
    ///
    /// If the Model was not found in the cache, a database lookup is made
    /// and the result (if some) is saved to the cache.
    pub async fn get(&self, identifier: Uuid) -> Result<Option<M>, rorm::Error> {
        if let Some(cached) = self.read_cache(identifier) {
            return Ok(cached.map(|x| x.item));
        }

        debug!("Model was not found in cache, making db lookup");
        self.query_db(identifier).await
    }

    /// Retrieve all available Models
    pub async fn get_all(&self) -> Result<Vec<M>, rorm::Error> {
        #[allow(clippy::expect_used)]
        let guard = self
            .cache
            .read()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");

        Ok(guard.values().flatten().cloned().map(|x| x.item).collect())
    }

    /// Check for the existing of a Model
    pub async fn exists(&self, identifier: Uuid) -> Result<bool, rorm::Error> {
        if let Some(cached) = self.read_cache(identifier) {
            return Ok(cached.is_some());
        }

        debug!("Model was not found in cache, making db lookup");
        Ok(self.query_db(identifier).await?.is_some())
    }

    /// Update a Model
    ///
    /// This method will update the specified Model in the cache and schedule the
    /// update in the database.
    pub async fn update(&self, item: M) -> Result<(), ItemNotInCacheError> {
        #[allow(clippy::expect_used)]
        let mut guard = self
            .cache
            .write()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");

        let uuid = *item.get_primary_key();

        if !guard.contains_key(&uuid) {
            return Err(ItemNotInCacheError);
        }

        guard.insert(
            uuid,
            Some(CacheItem {
                item,
                changed: true,
            }),
        );

        Ok(())
    }

    /// Deletes a Model
    ///
    /// You have to check for the deleted rows in the return value to check if
    /// there was a Model with the given uuid
    pub async fn delete(&self, identifier: Uuid) -> Result<u64, rorm::Error> {
        let deleted = M::delete_item_db(identifier).await?;

        // Short circuit if no entry was deleted
        if deleted == 0 {
            return Ok(deleted);
        }

        #[allow(clippy::expect_used)]
        let mut guard = self
            .cache
            .write()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");

        guard.remove_entry(&identifier);

        Ok(deleted)
    }

    /// Inserts a Model
    ///
    /// Returns an error if there is already a model with the same UUID
    pub async fn insert<P>(&self, item: P) -> Result<(), rorm::Error>
    where
        P: Sync + Send,
        P: Patch<Model = M>,
    {
        let populated = M::insert_item_db(item).await?;

        #[allow(clippy::expect_used)]
        let mut guard = self
            .cache
            .write()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");

        guard.insert(
            *populated.get_primary_key(),
            Some(CacheItem {
                item: populated,
                changed: false,
            }),
        );

        Ok(())
    }

    /// Construct a new instance of the FullCache
    pub fn new() -> Self {
        let cache = Self {
            cache: Arc::new(RwLock::new(InnerCache::new())),
        };

        tokio::spawn(cache.clone().schedule_cache_save(Duration::from_secs(5)));

        cache
    }

    /// Schedule the cache save
    async fn schedule_cache_save(self, itvl: Duration) {
        let mut timer = interval(itvl);
        loop {
            if let Err(err) = self.save_cache().await {
                error!("Error saving fd cache: {err}");
            }
            timer.tick().await;
        }
    }

    /// This method is used to save the cache to the database.
    ///
    /// It should only be used by the scheduler for regularly saving the cache to the database
    async fn save_cache(&self) -> Result<(), rorm::Error> {
        let items = {
            #[allow(clippy::expect_used)]
            let mut guard = self.cache.write().expect(
                "If you ever encounter this error, please open an issue with the stacktrace",
            );

            let items = guard
                .values()
                .flatten()
                .filter(|x| x.changed)
                .map(|x| &x.item)
                .cloned()
                .collect::<Vec<_>>();

            guard.values_mut().flatten().for_each(|x| x.changed = false);

            items
        };

        let mut tx = GLOBAL.db.start_transaction().await?;

        for item in items {
            M::update_item_db(item, &mut tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Reads the cache
    ///
    /// If the outer [Option] is [None], the database wasn't queried yet for this entry
    /// If the inner [Option] is [None], the database was queried, but no entry was found
    fn read_cache(&self, identifier: Uuid) -> Option<Option<CacheItem<M>>> {
        #[allow(clippy::expect_used)]
        let guard = self
            .cache
            .read()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");
        guard.get(&identifier).map(|x| x.to_owned())
    }

    /// Can be used to update the caching state.
    ///
    /// If no result with the given [Uuid] was found, [None] is inserted
    /// into the cache.
    async fn query_db(&self, identifier: Uuid) -> Result<Option<M>, rorm::Error> {
        let item = M::get_item_db(identifier).await?;

        #[allow(clippy::expect_used)]
        let mut guard = self
            .cache
            .write()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");
        if let Some(i) = &item {
            debug!("Model was found in db, inserting in cache");
            guard.insert(
                identifier,
                Some(CacheItem {
                    item: i.clone(),
                    changed: false,
                }),
            );
        } else {
            debug!("Model was not found in db, inserting in cache");
            guard.insert(identifier, None);
        }

        Ok(item)
    }
}
