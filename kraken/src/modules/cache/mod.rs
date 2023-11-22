//! This module holds all caches of kraken

use std::collections::HashMap;
use std::ops::Sub;
use std::sync::RwLock;

use chrono::{DateTime, Duration, Utc};
use futures::TryStreamExt;
use rorm::{query, FieldAccess, Model};
use thiserror::Error;
use uuid::Uuid;

use crate::api::handler::ApiError;
use crate::chan::GLOBAL;
use crate::models::{Workspace, WorkspaceMember};

struct WorkspaceUsers {
    last_refresh: DateTime<Utc>,
    users: Vec<Uuid>,
}

/// The cache for accessing the users that have access to a workspace
#[derive(Default)]
pub struct WorkspaceCache {
    cache: RwLock<HashMap<Uuid, WorkspaceUsers>>,
}

impl WorkspaceCache {
    /// Create a new workspace cache
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Retrieves all users that have access to the workspace
    pub async fn get_users(&self, workspace: Uuid) -> Result<Vec<Uuid>, CacheError> {
        let now = Utc::now();

        let mut users: Vec<Uuid> = vec![];
        let entry = {
            let guard = self.cache.read().expect(
                "If you ever encounter this error, please open an issue with the stacktrace",
            );

            if let Some(WorkspaceUsers {
                users,
                last_refresh,
            }) = guard.get(&workspace)
            {
                if now.sub(Duration::minutes(5)) >= *last_refresh {
                    None
                } else {
                    Some(users.clone())
                }
            } else {
                None
            }
        };

        // If the key does not exists or the last refresh time
        // is more than `REFRESH_PERIOD` ago, update the entry
        if entry.is_none() {
            let mut tx = GLOBAL.db.start_transaction().await?;

            let (owner,) = query!(&mut tx, (Workspace::F.owner,))
                .condition(Workspace::F.uuid.equals(workspace))
                .optional()
                .await?
                .ok_or(CacheError::WorkspaceNotFound)?;

            let members: Vec<Uuid> = query!(&mut tx, (WorkspaceMember::F.member,))
                .condition(WorkspaceMember::F.workspace.equals(workspace))
                .stream()
                .map_ok(|(x,)| *x.key())
                .try_collect()
                .await?;

            tx.commit().await?;

            users.extend(members);
            users.push(*owner.key());
            {
                let mut guard = self.cache.write().expect(
                    "If you ever encounter this error, please open an issue with the stacktrace",
                );
                guard.entry(workspace).and_modify(|x| {
                    *x = WorkspaceUsers {
                        users: users.clone(),
                        last_refresh: now,
                    }
                });
            }
        } else if let Some(u) = entry {
            users.extend(u);
        }

        Ok(users)
    }
}

/// The errors that can occur while working with the cache
#[derive(Error, Debug)]
pub enum CacheError {
    /// Database error occurred
    #[error("Database error: {0}")]
    Database(#[from] rorm::Error),
    /// Workspace was not found
    #[error("Workspace was not found")]
    WorkspaceNotFound,
}

impl From<CacheError> for ApiError {
    fn from(value: CacheError) -> Self {
        match value {
            CacheError::Database(x) => ApiError::DatabaseError(x),
            CacheError::WorkspaceNotFound => ApiError::InvalidWorkspace,
        }
    }
}
