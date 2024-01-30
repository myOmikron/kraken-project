use std::collections::HashMap;
use std::ops::Add;
use std::sync::RwLock;

use chrono::{DateTime, Duration, Utc};
use futures::TryStreamExt;
use log::debug;
use rorm::{query, Model};
use uuid::Uuid;

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
    /// Provides the access from the user uuid to the username
    /// and display_name via the [SimpleUser]
    users: HashMap<Uuid, SimpleUser>,
    /// The point in time when the cache was refreshed the last time
    last_refresh: DateTime<Utc>,
}

impl UserCache {
    /// Manually refresh the whole cache
    ///
    /// The updated cache is returned
    pub async fn refresh(&self) -> Result<HashMap<Uuid, SimpleUser>, rorm::Error> {
        let now = Utc::now();

        let db = &GLOBAL.db;
        let users: HashMap<Uuid, SimpleUser> =
            query!(db, (User::F.uuid, User::F.username, User::F.display_name))
                .stream()
                .map_ok(|(uuid, username, display_name)| {
                    (
                        uuid,
                        SimpleUser {
                            uuid,
                            username,
                            display_name,
                        },
                    )
                })
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

    /// Retrieve an user from the cache
    ///
    /// If `Ok(None)` is returned, the specified user does not exist
    pub async fn get_user(&self, uuid: Uuid) -> Result<Option<SimpleUser>, rorm::Error> {
        debug!("Workspace Member Cache was hit");
        let now = Utc::now();
        let refresh_period = Duration::minutes(5);

        let user: Option<SimpleUser> = {
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
            let users = self.refresh().await?;
            Ok(users.get(&uuid).map(|x| x.to_owned()))
        }
    }
}
