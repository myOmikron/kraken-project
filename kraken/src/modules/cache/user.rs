use std::collections::HashMap;
use std::ops::Add;
use std::sync::RwLock;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use futures::TryStreamExt;
use log::debug;
use rorm::query;
use rorm::Model;
use uuid::Uuid;

use crate::api::handler::users::schema::FullUser;
use crate::api::handler::users::schema::SimpleUser;
use crate::chan::global::GLOBAL;
use crate::models::User;

/// A cache to retrieve users from the database
///
/// The cache is refreshed fully, so the whole user collection is queried.
#[derive(Default)]
pub struct UserCache {
    /// The data of the cache
    cache: RwLock<UserCacheData>,
}

#[derive(Default)]
struct UserCacheData {
    /// Provides the access to all [FullUser]
    users: HashMap<Uuid, FullUser>,
    /// The point in time when the cache was refreshed the last time
    last_refresh: DateTime<Utc>,
}

impl UserCache {
    /// Manually refresh the whole cache
    ///
    /// The updated cache is returned
    pub async fn refresh(&self) -> Result<HashMap<Uuid, FullUser>, rorm::Error> {
        let now = Utc::now();

        let db = &GLOBAL.db;
        let users: HashMap<Uuid, FullUser> = query!(
            db,
            (
                User::F.uuid,
                User::F.username,
                User::F.display_name,
                User::F.permission,
                User::F.created_at,
                User::F.last_login
            )
        )
        .stream()
        .map_ok(
            |(uuid, username, display_name, permission, created_at, last_login)| {
                (
                    uuid,
                    FullUser {
                        uuid,
                        username,
                        display_name,
                        permission,
                        created_at,
                        last_login,
                    },
                )
            },
        )
        .try_collect()
        .await?;

        #[allow(clippy::expect_used)]
        let mut guard = self
            .cache
            .write()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");

        guard.users = users.clone();
        guard.last_refresh = now;

        Ok(users)
    }

    /// Retrieve a [FullUser] from the cache
    ///
    /// If `Ok(None)` is returned, the specified user does not exist
    pub async fn get_full_user(&self, uuid: Uuid) -> Result<Option<FullUser>, rorm::Error> {
        debug!("Workspace Member Cache was hit");
        let now = Utc::now();
        // panics on out-of-bounds, so this is fine
        #[allow(clippy::unwrap_used)]
        let refresh_period = Duration::try_minutes(5).unwrap();

        let user: Option<FullUser> = {
            #[allow(clippy::expect_used)]
            let guard = self.cache.read().expect(
                "If you ever encounter this error, please open an issue with the stacktrace",
            );

            if guard.last_refresh.add(refresh_period) <= now {
                debug!("Cache outdated");
                None
            } else {
                guard.users.get(&uuid).map(|x| x.to_owned())
            }
        };

        // If the key does not exists or the last refresh time
        // is more than `refresh_period` ago, update the cache
        if user.is_some() {
            Ok(user)
        } else {
            debug!("Refreshing cache");
            let mut users = self.refresh().await?;
            Ok(users.remove_entry(&uuid).map(|(_, user)| user))
        }
    }

    /// Retrieve a [SimpleUser] from the cache
    ///
    /// If `Ok(None)` is returned, the specified user does not exist
    pub async fn get_simple_user(&self, uuid: Uuid) -> Result<Option<SimpleUser>, rorm::Error> {
        Ok(self.get_full_user(uuid).await?.map(|user| SimpleUser {
            uuid: user.uuid,
            username: user.username,
            display_name: user.display_name,
        }))
    }
}
