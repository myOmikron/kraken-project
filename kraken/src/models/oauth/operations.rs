use rorm::db::Executor;
use rorm::prelude::*;
use rorm::{and, insert, query};
use thiserror::Error;
use uuid::Uuid;

use crate::api::handler::ApiError;
use crate::models::{OAuthDecision, OAuthDecisionAction, Workspace};
use crate::modules::oauth::OAuthScope;

impl OAuthDecision {
    /// Insert a new [`OAuthDecision`]
    pub async fn insert(
        executor: impl Executor<'_>,
        user: Uuid,
        app: Uuid,
        scope: OAuthScope,
        action: OAuthDecisionAction,
    ) -> Result<Uuid, InsertOAuthDecisionError> {
        let OAuthScope { workspace } = scope;

        let mut guard = executor.ensure_transaction().await?;

        if !Workspace::is_user_member_or_owner(guard.get_transaction(), workspace, user).await? {
            return Err(InsertOAuthDecisionError::MissingPrivileges);
        }

        let uuid = insert!(guard.get_transaction(), OAuthDecision)
            .return_primary_key()
            .single(&OAuthDecision {
                uuid: Uuid::new_v4(),
                user: ForeignModelByField::Key(user),
                application: ForeignModelByField::Key(app),
                workspace: ForeignModelByField::Key(workspace),
                action,
            })
            .await?;

        guard.commit().await?;

        Ok(uuid)
    }

    /// Get a [`OAuthDecision`]'s action (if it exists)
    pub async fn get(
        executor: impl Executor<'_>,
        user: Uuid,
        app: Uuid,
        scope: OAuthScope,
    ) -> Result<Option<OAuthDecisionAction>, rorm::Error> {
        let OAuthScope { workspace } = scope;
        let option = query!(executor, (OAuthDecision::F.action,))
            .condition(and![
                OAuthDecision::F.application.equals(app),
                OAuthDecision::F.user.equals(user),
                OAuthDecision::F.workspace.equals(workspace)
            ])
            .optional()
            .await?;
        Ok(option.map(|(action,)| action))
    }
}

/// The errors that can occur when inserting an oauth decision
#[derive(Error, Debug)]
pub enum InsertOAuthDecisionError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] rorm::Error),
    /// Workspace does not exist or the user doesn't has privileges to access it
    #[error("The provided workspace is invalid or not accessible")]
    MissingPrivileges,
}

impl From<InsertOAuthDecisionError> for ApiError {
    fn from(value: InsertOAuthDecisionError) -> Self {
        match value {
            InsertOAuthDecisionError::Database(x) => ApiError::DatabaseError(x),
            InsertOAuthDecisionError::MissingPrivileges => ApiError::MissingPrivileges,
        }
    }
}
