//! The user deletion lives here

use std::fmt::{Display, Formatter};

use rorm::transaction::Transaction;
use rorm::{delete, query, Database, Model};

use crate::models::User;

/**
Deletes a user in a transaction.

**Parameter**:
- `username`: Username of the user
- `db`: Reference of a [Database] instance
 */
pub async fn delete_user_transaction(
    username: String,
    db: &Database,
) -> Result<(), DeleteUserError> {
    let mut tx = db.start_transaction().await?;

    delete_user(username, db, &mut tx).await?;

    tx.commit().await?;

    Ok(())
}

/**
Deletes a user

**Parameter**:
- `username`: Username of the user
- `db`: Reference of a [Database] instance
- `tx`: A mutable reference to a [Transaction]
 */
pub async fn delete_user<'db>(
    username: String,
    db: &'db Database,
    tx: &mut Transaction<'db>,
) -> Result<(), DeleteUserError> {
    query!(db, (User::F.uuid,))
        .transaction(tx)
        .condition(User::F.username.equals(&username))
        .optional()
        .await?
        .ok_or(DeleteUserError::InvalidUsername)?;

    delete!(db, User)
        .transaction(tx)
        .condition(User::F.username.equals(&username))
        .await?;

    Ok(())
}

/// Errors that can occur when deleting users
#[derive(Debug)]
pub enum DeleteUserError {
    /// Database error
    DatabaseError(rorm::Error),
    /// A user with that username does not exists
    InvalidUsername,
}

impl Display for DeleteUserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteUserError::DatabaseError(err) => write!(f, "Database error {err}"),
            DeleteUserError::InvalidUsername => write!(f, "Invalid username"),
        }
    }
}

impl From<rorm::Error> for DeleteUserError {
    fn from(value: rorm::Error) -> Self {
        Self::DatabaseError(value)
    }
}
