use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::thread_rng;
use rorm::db::Executor;
use rorm::fields::types::Json;
use rorm::prelude::*;
use rorm::{insert, query};
use thiserror::Error;
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

use super::{LocalUser, LocalUserKey, User};
use crate::api::handler::common::error::ApiError;
use crate::models::UserPermission;

#[derive(Patch)]
#[rorm(model = "User")]
struct UserInsert {
    uuid: Uuid,
    username: String,
    display_name: String,
    permission: UserPermission,
}

#[derive(Patch)]
#[rorm(model = "LocalUser")]
struct LocalUserInsert {
    uuid: Uuid,
    user: ForeignModel<User>,
    password_hash: String,
}

#[derive(Patch)]
#[rorm(model = "LocalUserKey")]
struct LocalUserKeyInsert {
    uuid: Uuid,
    name: String,
    user: ForeignModel<LocalUser>,
    key: Json<Passkey>,
}

/// The errors that can occur when inserting a [LocalUserKey]
#[derive(Debug, Error)]
pub enum InsertUserKeyError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rorm::Error),
}

impl From<InsertUserKeyError> for ApiError {
    fn from(value: InsertUserKeyError) -> Self {
        match value {
            InsertUserKeyError::DatabaseError(x) => ApiError::DatabaseError(x),
        }
    }
}

impl LocalUserKey {
    /// Insert a new [LocalUserKey]
    pub async fn insert(
        executor: impl Executor<'_>,
        user: Uuid,
        name: String,
        key: Passkey,
    ) -> Result<Uuid, InsertUserKeyError> {
        let uuid = Uuid::new_v4();

        insert!(executor, LocalUserKeyInsert)
            .single(&LocalUserKeyInsert {
                uuid,
                user: ForeignModelByField::Key(user),
                key: Json(key),
                name,
            })
            .await?;

        Ok(uuid)
    }
}

/// The errors that can occur when inserting a [User]
#[derive(Debug, Error)]
pub enum InsertUserError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rorm::Error),
    #[error("Password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::errors::Error),
    #[error("Username already occupied")]
    UsernameAlreadyOccupied,
    #[error("The chosen username is invalid")]
    InvalidUsername,
}

impl From<InsertUserError> for ApiError {
    fn from(value: InsertUserError) -> Self {
        match value {
            InsertUserError::DatabaseError(x) => ApiError::DatabaseError(x),
            InsertUserError::PasswordHashError(x) => ApiError::InvalidHash(x),
            InsertUserError::UsernameAlreadyOccupied => ApiError::UsernameAlreadyOccupied,
            InsertUserError::InvalidUsername => ApiError::InvalidUsername,
        }
    }
}

impl User {
    /// Checks whether a user exists
    pub async fn exists(executor: impl Executor<'_>, uuid: Uuid) -> Result<bool, rorm::Error> {
        query!(executor, (User::F.uuid,))
            .condition(User::F.uuid.equals(uuid))
            .optional()
            .await
            .map(|x| x.is_some())
    }

    /// Check whether a user exists when querying by username
    pub async fn exists_by_username(
        executor: impl Executor<'_>,
        username: &str,
    ) -> Result<bool, rorm::Error> {
        Ok(query!(executor, (User::F.uuid,))
            .condition(User::F.username.equals(username))
            .optional()
            .await?
            .is_some())
    }

    /// Insert a new [User]
    pub async fn insert_local_user(
        executor: impl Executor<'_>,
        username: String,
        display_name: String,
        password: String,
        permission: UserPermission,
    ) -> Result<Uuid, InsertUserError> {
        if username.is_empty() {
            return Err(InsertUserError::InvalidUsername);
        }

        let mut guard = executor.ensure_transaction().await?;

        let exists = User::exists_by_username(guard.get_transaction(), &username).await?;
        if exists {
            return Err(InsertUserError::UsernameAlreadyOccupied);
        }

        let uuid = Uuid::new_v4();

        let salt = SaltString::generate(&mut thread_rng());
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        insert!(guard.get_transaction(), UserInsert)
            .return_nothing()
            .single(&UserInsert {
                uuid,
                username,
                display_name,
                permission,
            })
            .await?;

        insert!(guard.get_transaction(), LocalUserInsert)
            .return_nothing()
            .single(&LocalUserInsert {
                uuid: Uuid::new_v4(),
                user: ForeignModelByField::Key(uuid),
                password_hash,
            })
            .await?;

        guard.commit().await?;

        Ok(uuid)
    }
}
