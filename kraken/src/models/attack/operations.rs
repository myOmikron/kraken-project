use chrono::DateTime;
use chrono::Utc;
use rorm::and;
use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::*;
use rorm::query;
use thiserror::Error;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::models::Attack;
use crate::models::AttackType;
use crate::models::User;
use crate::models::Workspace;
use crate::models::WorkspaceMember;

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
    /// The given user is not valid
    #[error("The given user is not valid")]
    UserInvalid,
}

impl From<InsertAttackError> for ApiError {
    fn from(value: InsertAttackError) -> Self {
        match value {
            InsertAttackError::DatabaseError(x) => ApiError::DatabaseError(x),
            InsertAttackError::WorkspaceInvalid => ApiError::InvalidWorkspace,
            InsertAttackError::UserInvalid => ApiError::InternalServerError,
        }
    }
}

impl Attack {
    /// Does the user have access to the attack's workspace?
    /// I.e. is owner or member?
    pub async fn has_access(
        executor: impl Executor<'_>,
        attack_uuid: Uuid,
        user_uuid: Uuid,
    ) -> Result<bool, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let Some((workspace, owner)) = query!(&mut *tx, (Workspace::F.uuid, Workspace::F.owner))
            .condition(Workspace::F.attacks.uuid.equals(attack_uuid))
            .optional()
            .await?
        else {
            return Ok(false);
        };

        if *owner.key() == user_uuid {
            return Ok(true);
        }

        let access = query!(&mut *tx, (WorkspaceMember::F.id,))
            .condition(and!(
                WorkspaceMember::F.workspace.equals(workspace),
                WorkspaceMember::F.member.equals(user_uuid),
            ))
            .optional()
            .await?
            .is_some();

        guard.commit().await?;
        Ok(access)
    }

    /// Insert a new [Attack]
    pub async fn insert(
        executor: impl Executor<'_>,
        attack_type: AttackType,
        started_by: Uuid,
        workspace: Uuid,
    ) -> Result<Attack, InsertAttackError> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        if !Workspace::exists(&mut *tx, workspace).await? {
            return Err(InsertAttackError::WorkspaceInvalid);
        }

        let attack = insert!(&mut *tx, AttackInsert)
            .return_patch()
            .single(&AttackInsert {
                uuid: Uuid::new_v4(),
                attack_type,
                started_by: ForeignModelByField::Key(started_by),
                workspace: ForeignModelByField::Key(workspace),
                finished_at: None,
            })
            .await?;

        guard.commit().await?;
        Ok(attack)
    }
}
