//! The user creation lives here

use std::fmt::{Display, Formatter};

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::thread_rng;
use rorm::transaction::Transaction;
use rorm::{insert, query, Database, Model};
use webauthn_rs::prelude::Uuid;

use crate::models::{User, UserInsert};

/**
Create a user in a transaction.

Returns the [Uuid] of the user.

**Parameter**:
- `username`: Username of the user
- `display_name`: The name that will be used for displaying purposes.
- `password`: Password of the user
- `admin`: Flag if the user has administrative privileges
- `db`: Reference of a [Database] instance
 */
pub async fn create_user_transaction(
    username: String,
    display_name: String,
    password: String,
    admin: bool,
    db: &Database,
) -> Result<Uuid, CreateUserError> {
    let mut tx = db.start_transaction().await?;

    let uuid = create_user(username, display_name, password, admin, &mut tx).await?;

    tx.commit().await?;

    Ok(uuid)
}

/**
Create a user in a given transaction.

Returns the [Uuid] of the user.

**Parameter**:
- `username`: Username of the user
- `display_name`: The name that will be used for displaying purposes.
- `password`: Password of the user
- `admin`: Flag if the user has administrative privileges
- `tx`: A mutable reference to a [Transaction]
 */
pub async fn create_user(
    username: String,
    display_name: String,
    password: String,
    admin: bool,
    tx: &mut Transaction,
) -> Result<Uuid, CreateUserError> {
    query!(&mut *tx, (User::F.uuid,))
        .optional()
        .await?
        .ok_or(CreateUserError::UsernameAlreadyExists)?;

    let salt = SaltString::generate(&mut thread_rng());
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    let uuid = Uuid::new_v4();

    insert!(&mut *tx, UserInsert)
        .single(&UserInsert {
            uuid,
            username,
            display_name,
            password_hash,
            admin,
            last_login: None,
        })
        .await?;

    Ok(uuid)
}

/// The possibles errors that may occur, when creating a user
#[derive(Debug)]
pub enum CreateUserError {
    /// Database error
    DatabaseError(rorm::Error),
    /// A user with that username already exists
    UsernameAlreadyExists,
    /// An error occurred while hashing the password
    HashError(argon2::password_hash::Error),
}

impl Display for CreateUserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateUserError::DatabaseError(err) => write!(f, "Database error: {err}"),
            CreateUserError::UsernameAlreadyExists => write!(f, "Username already exists"),
            CreateUserError::HashError(err) => write!(f, "Error while hashing: {err}"),
        }
    }
}

impl From<rorm::Error> for CreateUserError {
    fn from(value: rorm::Error) -> Self {
        Self::DatabaseError(value)
    }
}

impl From<argon2::password_hash::Error> for CreateUserError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::HashError(value)
    }
}
