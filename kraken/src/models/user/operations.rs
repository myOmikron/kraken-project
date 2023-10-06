use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono::{DateTime, Utc};
use rand::thread_rng;
use rorm::db::Executor;
use rorm::fields::types::Json;
use rorm::prelude::*;
use rorm::{insert, query};
use thiserror::Error;
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

use super::{User, UserKey};
use crate::api::handler::ApiError;

#[derive(Patch)]
#[rorm(model = "User")]
struct UserInsert {
    uuid: Uuid,
    username: String,
    display_name: String,
    password_hash: String,
    admin: bool,
    last_login: Option<DateTime<Utc>>,
}

#[derive(Patch)]
#[rorm(model = "UserKey")]
struct UserKeyInsert {
    uuid: Uuid,
    name: String,
    user: ForeignModel<User>,
    key: Json<Passkey>,
}

/// The errors that can occur when inserting a [UserKey]
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

impl UserKey {
    /// Insert a new [UserKey]
    pub async fn insert(
        executor: impl Executor<'_>,
        user: Uuid,
        name: String,
        key: Passkey,
    ) -> Result<Uuid, InsertUserKeyError> {
        let uuid = Uuid::new_v4();

        insert!(executor, UserKeyInsert)
            .single(&UserKeyInsert {
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
    pub async fn insert(
        executor: impl Executor<'_>,
        username: String,
        display_name: String,
        password: String,
        admin: bool,
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
            .single(&UserInsert {
                uuid,
                username,
                display_name,
                password_hash,
                admin,
                last_login: None,
            })
            .await?;

        guard.commit().await?;

        Ok(uuid)
    }
}
