use std::collections::HashMap;
use std::ops::Add;
use std::sync::RwLock;

use chrono::{DateTime, Duration, Utc};
use futures::TryStreamExt;
use log::debug;
use rorm::db::Executor;
use rorm::{query, FieldAccess, Model};
use uuid::Uuid;

use crate::models::{Workspace, WorkspaceMember};

#[derive(Debug)]
struct WorkspaceUsers {
    last_refresh: DateTime<Utc>,
    users: Option<Vec<Uuid>>,
}

/// The cache for accessing the users that have access to a workspace
#[derive(Default)]
pub struct WorkspaceCache {
    cache: RwLock<HashMap<Uuid, WorkspaceUsers>>,
}

impl WorkspaceCache {
    /// Trigger a manual refresh of the users of a specific workspace
    pub async fn refresh_users(
        &self,
        workspace: Uuid,
        tx: impl Executor<'_>,
    ) -> Result<Option<Vec<Uuid>>, rorm::Error> {
        let now = Utc::now();
        let mut users = None;

        let mut tx_guard = tx.ensure_transaction().await?;

        let owner = query!(tx_guard.get_transaction(), (Workspace::F.owner,))
            .condition(Workspace::F.uuid.equals(workspace))
            .optional()
            .await?;

        if let Some((owner,)) = owner {
            let mut members: Vec<Uuid> =
                query!(tx_guard.get_transaction(), (WorkspaceMember::F.member,))
                    .condition(WorkspaceMember::F.workspace.equals(workspace))
                    .stream()
                    .map_ok(|(x,)| *x.key())
                    .try_collect()
                    .await?;

            members.push(*owner.key());
            users = Some(members);
        }

        tx_guard.commit().await?;
        #[allow(clippy::expect_used)]
        let mut guard = self
            .cache
            .write()
            .expect("If you ever encounter this error, please open an issue with the stacktrace");

        guard.insert(
            workspace,
            WorkspaceUsers {
                users: users.clone(),
                last_refresh: now,
            },
        );

        Ok(users)
    }

    /// Retrieves all users that have access to the workspace
    ///
    /// If the Option is None, there is no workspace with that uuid
    pub async fn get_users(
        &self,
        workspace: Uuid,
        tx: impl Executor<'_>,
    ) -> Result<Option<Vec<Uuid>>, rorm::Error> {
        debug!("Workspace Member Cache was hit");
        let now = Utc::now();
        let refresh_period = Duration::minutes(5);

        let mut users: Option<Vec<Uuid>> = None;
        let entry = {
            #[allow(clippy::expect_used)]
            let guard = self.cache.read().expect(
                "If you ever encounter this error, please open an issue with the stacktrace",
            );

            if let Some(WorkspaceUsers {
                users,
                last_refresh,
            }) = guard.get(&workspace)
            {
                if last_refresh.add(refresh_period) <= now {
                    None
                } else {
                    Some(users.clone())
                }
            } else {
                None
            }
        };

        // If the key does not exists or the last refresh time
        // is more than `refresh_period` ago, update the entry
        if entry.is_none() {
            debug!("Refreshing users");
            users = self.refresh_users(workspace, tx).await?;
        } else if let Some(u) = entry {
            users = u;
        }

        Ok(users)
    }
}
