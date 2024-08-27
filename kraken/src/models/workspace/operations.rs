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
use crate::chan::global::GLOBAL;
use crate::models::OauthClient;
use crate::models::User;
use crate::models::Workspace;
use crate::models::WorkspaceAccessToken;
use crate::models::WorkspaceInvitation;
use crate::models::WorkspaceMember;
use crate::models::WorkspaceMemberPermission;
use crate::models::WorkspaceNotesInsert;

#[derive(Patch)]
#[rorm(model = "WorkspaceMember")]
struct WorkspaceMemberInsert {
    member: ForeignModel<User>,
    permission: WorkspaceMemberPermission,
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
    /// Add a member to a workspace
    pub async fn add_member(
        executor: impl Executor<'_>,
        workspace_uuid: Uuid,
        user: Uuid,
        permission: WorkspaceMemberPermission,
    ) -> Result<(), AddMemberError> {
        let mut guard = executor.ensure_transaction().await?;

        let workspace = query!(guard.get_transaction(), Workspace)
            .condition(Workspace::F.uuid.equals(workspace_uuid))
            .optional()
            .await?
            .ok_or(AddMemberError::InvalidWorkspace)?;

        if !User::exists(guard.get_transaction(), user).await? {
            return Err(AddMemberError::InvalidUser);
        }

        if *workspace.owner.key() == user {
            return Err(AddMemberError::IsOwner);
        }

        // Check if the user is already member of the workspace
        if query!(guard.get_transaction(), (WorkspaceMember::F.id,))
            .condition(and!(
                WorkspaceMember::F.workspace.equals(workspace_uuid),
                WorkspaceMember::F.member.equals(user)
            ))
            .optional()
            .await?
            .is_some()
        {
            return Err(AddMemberError::AlreadyMember);
        }

        insert!(guard.get_transaction(), WorkspaceMemberInsert)
            .single(&WorkspaceMemberInsert {
                member: ForeignModelByField::Key(user),
                workspace: ForeignModelByField::Key(workspace_uuid),
                permission,
            })
            .await?;

        guard.commit().await?;

        Ok(())
    }

    /// Check if a user is owner or member of a workspace
    pub async fn is_user_member_or_owner(
        executor: impl Executor<'_>,
        workspace: Uuid,
        user: Uuid,
    ) -> Result<bool, rorm::Error> {
        if let Some(users) = GLOBAL
            .workspace_users_cache
            .get_users(workspace, executor)
            .await?
        {
            Ok(users.contains(&user))
        } else {
            Ok(false)
        }
    }

    /// Checks whether a user is owner of a specific workspace
    pub async fn is_owner(
        executor: impl Executor<'_>,
        workspace: Uuid,
        user: Uuid,
    ) -> Result<bool, rorm::Error> {
        Ok(query!(executor, (Workspace::F.owner,))
            .condition(and!(
                Workspace::F.uuid.equals(workspace),
                Workspace::F.owner.equals(user)
            ))
            .optional()
            .await?
            .is_some())
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

        let mut guard = executor.ensure_transaction().await?;

        insert!(guard.get_transaction(), WorkspaceInsert)
            .return_nothing()
            .single(&WorkspaceInsert {
                uuid,
                name,
                description,
                owner: ForeignModelByField::Key(owner),
            })
            .await?;

        insert!(guard.get_transaction(), WorkspaceNotesInsert)
            .single(&WorkspaceNotesInsert {
                uuid,
                notes: "".to_string(),
                workspace: ForeignModelByField::Key(uuid),
            })
            .await?;

        guard.commit().await?;

        Ok(uuid)
    }
}

/// The errors that can occur while adding a member to a workspace
#[derive(Debug, Error)]
pub enum AddMemberError {
    /// Database error
    #[error("Database error occurred: {0}")]
    Database(#[from] rorm::Error),
    /// Invalid workspace
    #[error("Invalid Workspace")]
    InvalidWorkspace,
    /// Invalid user
    #[error("Invalid User")]
    InvalidUser,
    /// The user is already member
    #[error("The user is already member of the workspace")]
    AlreadyMember,
    /// The user is owner of the workspace
    #[error("The user is owner of the workspace")]
    IsOwner,
}

impl From<AddMemberError> for ApiError {
    /// Can always be mapped to internal server error for the api, as the database must be
    /// corrupt the error occurs
    fn from(value: AddMemberError) -> Self {
        match value {
            AddMemberError::Database(x) => ApiError::DatabaseError(x),
            _ => ApiError::InternalServerError,
        }
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

impl WorkspaceInvitation {
    /// Insert a new invitation for the workspace
    pub async fn insert(
        executor: impl Executor<'_>,
        workspace: Uuid,
        from: Uuid,
        target: Uuid,
    ) -> Result<Uuid, InsertWorkspaceInvitationError> {
        if from == target {
            return Err(InsertWorkspaceInvitationError::InvalidTarget);
        }

        let mut guard = executor.ensure_transaction().await?;

        if !Workspace::exists(guard.get_transaction(), workspace).await? {
            return Err(InsertWorkspaceInvitationError::InvalidWorkspace);
        }

        if !Workspace::is_owner(guard.get_transaction(), workspace, from).await? {
            return Err(InsertWorkspaceInvitationError::MissingPrivileges);
        }

        if !User::exists(guard.get_transaction(), target).await? {
            return Err(InsertWorkspaceInvitationError::InvalidTarget);
        }

        // Check if target is already part of the workspace
        if query!(guard.get_transaction(), (WorkspaceMember::F.id,))
            .condition(and!(
                WorkspaceMember::F.workspace.equals(workspace),
                WorkspaceMember::F.member.equals(target)
            ))
            .optional()
            .await?
            .is_some()
        {
            return Err(InsertWorkspaceInvitationError::AlreadyInWorkspace);
        }

        // Check if the user was already invited
        if query!(guard.get_transaction(), (WorkspaceInvitation::F.uuid,))
            .condition(and!(
                WorkspaceInvitation::F.workspace.equals(workspace),
                WorkspaceInvitation::F.target.equals(target),
                WorkspaceInvitation::F.from.equals(from)
            ))
            .optional()
            .await?
            .is_some()
        {
            return Err(InsertWorkspaceInvitationError::InvalidTarget);
        }

        // Insert the invitation
        let invitation_uuid = insert!(guard.get_transaction(), WorkspaceInvitationInsert)
            .return_primary_key()
            .single(&WorkspaceInvitationInsert {
                uuid: Uuid::new_v4(),
                workspace: ForeignModelByField::Key(workspace),
                target: ForeignModelByField::Key(target),
                from: ForeignModelByField::Key(from),
            })
            .await?;

        guard.commit().await?;

        Ok(invitation_uuid)
    }
}

#[derive(Patch)]
#[rorm(model = "WorkspaceInvitation")]
struct WorkspaceInvitationInsert {
    uuid: Uuid,
    workspace: ForeignModel<Workspace>,
    from: ForeignModel<User>,
    target: ForeignModel<User>,
}

/// The errors that can occur when inserting an invitation to an workspace
#[derive(Debug, Error)]
pub enum InsertWorkspaceInvitationError {
    /// A database error
    #[error("Database error occurred: {0}")]
    Database(#[from] rorm::Error),
    /// Invalid workspace
    #[error("Invalid workspace")]
    InvalidWorkspace,
    /// Missing privileges
    #[error("Missing privileges")]
    MissingPrivileges,
    /// Invalid target user
    #[error("Invalid target user")]
    InvalidTarget,
    /// The target is already part of the workspace
    #[error("The target is already part of the workspace")]
    AlreadyInWorkspace,
    /// The user was already invited
    #[error("The user was already invited")]
    AlreadyInvited,
}

impl From<InsertWorkspaceInvitationError> for ApiError {
    fn from(value: InsertWorkspaceInvitationError) -> Self {
        match value {
            InsertWorkspaceInvitationError::Database(x) => ApiError::DatabaseError(x),
            InsertWorkspaceInvitationError::InvalidWorkspace => ApiError::InvalidWorkspace,
            InsertWorkspaceInvitationError::MissingPrivileges => ApiError::MissingPrivileges,
            InsertWorkspaceInvitationError::InvalidTarget => ApiError::InvalidTarget,
            InsertWorkspaceInvitationError::AlreadyInWorkspace => ApiError::AlreadyMember,
            InsertWorkspaceInvitationError::AlreadyInvited => ApiError::AlreadyInvited,
        }
    }
}
