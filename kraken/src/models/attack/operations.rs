use chrono::{DateTime, Utc};
use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::*;
use thiserror::Error;
use uuid::Uuid;

use crate::api::handler::ApiError;
use crate::models::{Attack, AttackType, User, Workspace};

#[derive(Patch)]
#[rorm(model = "Attack")]
struct AttackInsert {
    uuid: Uuid,
    attack_type: AttackType,
    started_by: ForeignModel<User>,
    workspace: ForeignModel<Workspace>,
    finished_at: Option<DateTime<Utc>>,
}

/// The errors that can occur while inserting an [Attack]
#[derive(Debug, Error)]
pub enum InsertAttackError {
    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(#[from] rorm::Error),
    /// The given workspace is not valid
    #[error("The given workspace is not valid")]
    WorkspaceInvalid,
}

impl From<InsertAttackError> for ApiError {
    fn from(value: InsertAttackError) -> Self {
        match value {
            InsertAttackError::DatabaseError(x) => ApiError::DatabaseError(x),
            InsertAttackError::WorkspaceInvalid => ApiError::InvalidWorkspace,
        }
    }
}

impl Attack {
    /// Insert a new [Attack]
    pub async fn insert(
        executor: impl Executor<'_>,
        attack_type: AttackType,
        started_by: Uuid,
        workspace: Uuid,
    ) -> Result<Uuid, InsertAttackError> {
        let mut guard = executor.ensure_transaction().await?;

        if !Workspace::exists(guard.get_transaction(), workspace).await? {
            return Err(InsertAttackError::WorkspaceInvalid);
        }

        let uuid = Uuid::new_v4();
        insert!(guard.get_transaction(), AttackInsert)
            .return_nothing()
            .single(&AttackInsert {
                uuid,
                attack_type,
                started_by: ForeignModelByField::Key(started_by),
                workspace: ForeignModelByField::Key(workspace),
                finished_at: None,
            })
            .await?;

        guard.commit().await?;

        Ok(uuid)
    }
}
