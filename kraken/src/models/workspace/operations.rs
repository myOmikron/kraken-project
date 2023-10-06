use chrono::{DateTime, Utc};
use rorm::db::Executor;
use rorm::prelude::*;
use rorm::{and, insert, query};
use thiserror::Error;
use uuid::Uuid;

use crate::api::handler::ApiError;
use crate::models::{OauthClient, User, Workspace, WorkspaceAccessToken, WorkspaceMember};

#[derive(Patch)]
#[rorm(model = "WorkspaceMember")]
struct WorkspaceMemberInsert {
    member: ForeignModel<User>,
    workspace: ForeignModel<Workspace>,
}

#[derive(Patch)]
#[rorm(model = "Workspace")]
struct WorkspaceInsert {
    uuid: Uuid,
    name: String,
    description: Option<String>,
    owner: ForeignModel<User>,
}

#[derive(Patch)]
#[rorm(model = "WorkspaceAccessToken")]
struct WorkspaceAccessTokenInsert {
    token: String,
    user: ForeignModel<User>,
    workspace: ForeignModel<Workspace>,
    expires_at: DateTime<Utc>,
    application: ForeignModel<OauthClient>,
}

/// The errors can occur while inserting a new workspace
#[derive(Error, Debug)]
pub enum InsertWorkspaceError {
    #[error("An database error occurred")]
    DatabaseError(#[from] rorm::Error),
    #[error("An empty name was given")]
    EmptyName,
}

impl From<InsertWorkspaceError> for ApiError {
    fn from(value: InsertWorkspaceError) -> Self {
        match value {
            InsertWorkspaceError::DatabaseError(x) => ApiError::DatabaseError(x),
            InsertWorkspaceError::EmptyName => ApiError::InvalidName,
        }
    }
}

impl Workspace {
    /// Check if a user is owner or member of a workspace
    pub async fn is_user_member_or_owner(
        executor: impl Executor<'_>,
        workspace: Uuid,
        user: Uuid,
    ) -> Result<bool, rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;

        // Check existence of workspace
        let Some((owner,)) = query!(guard.get_transaction(), (Workspace::F.owner,))
            .condition(Workspace::F.uuid.equals(workspace))
            .optional()
            .await?
        else {
            return Ok(false);
        };

        // Check if user is owner or member
        if *owner.key() != user {
            let existent = query!(guard.get_transaction(), (WorkspaceMember::F.id,))
                .condition(and!(
                    WorkspaceMember::F.member.equals(user),
                    WorkspaceMember::F.workspace.equals(workspace)
                ))
                .optional()
                .await?;

            if existent.is_none() {
                return Ok(false);
            }
        }

        guard.commit().await?;

        Ok(true)
    }

    /// Check whether a workspace exists
    pub async fn exists(executor: impl Executor<'_>, uuid: Uuid) -> Result<bool, rorm::Error> {
        Ok(query!(executor, (Workspace::F.uuid,))
            .condition(Workspace::F.uuid.equals(uuid))
            .optional()
            .await?
            .is_some())
    }

    /// Insert a new workspace for an user
    pub async fn insert(
        executor: impl Executor<'_>,
        name: String,
        description: Option<String>,
        owner: Uuid,
    ) -> Result<Uuid, InsertWorkspaceError> {
        let uuid = Uuid::new_v4();

        if name.is_empty() {
            return Err(InsertWorkspaceError::EmptyName);
        }

        insert!(executor, WorkspaceInsert)
            .return_nothing()
            .single(&WorkspaceInsert {
                uuid,
                name,
                description,
                owner: ForeignModelByField::Key(owner),
            })
            .await?;

        Ok(uuid)
    }
}

impl WorkspaceAccessToken {
    /// Insert a workspace access token
    pub async fn insert(
        executor: impl Executor<'_>,
        token: String,
        expires_at: DateTime<Utc>,
        user: Uuid,
        workspace: Uuid,
        application: Uuid,
    ) -> Result<i64, rorm::Error> {
        insert!(executor, WorkspaceAccessTokenInsert)
            .return_primary_key()
            .single(&WorkspaceAccessTokenInsert {
                token,
                expires_at,
                user: ForeignModelByField::Key(user),
                workspace: ForeignModelByField::Key(workspace),
                application: ForeignModelByField::Key(application),
            })
            .await
    }
}
