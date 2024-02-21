//! This module holds types for building caches

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, RwLock};

use log::debug;
use rorm::db::transaction::Transaction;
use rorm::internal::field::{Field, SingleColumnField};
use rorm::model::Identifiable;
use rorm::Model;
use thiserror::Error;
use uuid::Uuid;

use crate::chan::global::GLOBAL;

/// A generic struct for implementing caches.
///
/// Implement [RwCache] for a specific type on this type to create a new cache.
pub struct FullCache<M>
where
    M: Model,
    M::Primary: Field<Type = Uuid>,
{
    cache: Arc<RwLock<InnerCache<M>>>,
}

type InnerCache<M> = HashMap<Uuid, Option<CacheItem<M>>>;

#[derive(Clone)]
struct CacheItem<M>
where
    M: Model,
    M::Primary: Field<Type = Uuid>,
{
    item: M,
    changed: bool,
}

/// The database control for a model
///
/// This is used by [RwCache] to be able to default-implement all high-level methods.
/// To create a new cache, implement these methods for your specific model.
pub trait CacheDBControl<M>
where
    M: Model + Identifiable + Send,
    M::Primary: Field<Type = Uuid> + SingleColumnField,
    <M as rorm::Patch>::Decoder: Send,
{
    /// Delete a specific item in the database
    fn delete_item_db(
        &self,
        identifier: Uuid,
    ) -> impl Future<Output = Result<u64, rorm::Error>> + Send {
        async move {
            rorm::delete!(&GLOBAL.db, M)
                .condition(M::Primary::type_as_value(&identifier))
                .await
        }
    }

    /// Retrieve a specific item in the database
    fn get_item_db(
        &self,
        identifier: Uuid,
    ) -> impl Future<Output = Result<Option<M>, rorm::Error>> + Send {
        async move {
            rorm::query!(&GLOBAL.db, M)
                .condition(M::Primary::type_as_value(&identifier))
                .optional()
                .await
        }
    }

    /// Take the item and update it in the database using
    /// this provided [Transaction].
    fn update_item_db(
        &self,
        item: M,
        tx: &mut Transaction,
    ) -> impl Future<Output = Result<(), rorm::Error>> + Send;
}

/// The Model was not found
#[derive(Error, Debug)]
#[error("Item was not found")]
pub struct ItemNotInCacheError;

impl<M> FullCache<M>
where
    Self: CacheDBControl<M>,
    M: Model + Clone + Identifiable + Send,
    M::Primary: Field<Type = Uuid>,
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
        let deleted = self.delete_item_db(identifier).await?;

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

    /// This method is used to save the cache to the database.
    ///
    /// It should only be used by the scheduler for regularly saving the cache to the database
    pub async fn save_cache(&self) -> Result<(), rorm::Error> {
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
            self.update_item_db(item, &mut tx).await?;
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
        let item = self.get_item_db(identifier).await?;

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
