use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use rorm::db::Executor;
use rorm::insert;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use thiserror::Error;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::models::BearerToken;

/// The possible errors that occur when inserting a bearer token
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum InsertBearerTokenError {
    #[error("Database error: {0}")]
    Database(#[from] rorm::Error),
    #[error("Name must not be empty")]
    EmptyName,
    #[error("Name already exists")]
    NameAlreadyExists,
}

impl From<InsertBearerTokenError> for ApiError {
    fn from(value: InsertBearerTokenError) -> Self {
        match value {
            InsertBearerTokenError::Database(x) => ApiError::DatabaseError(x),
            InsertBearerTokenError::EmptyName => ApiError::InvalidName,
            InsertBearerTokenError::NameAlreadyExists => ApiError::NameAlreadyExists,
        }
    }
}

impl BearerToken {
    /// Insert a new bearer token
    pub async fn insert(
        exe: impl Executor<'_>,
        name: String,
    ) -> Result<Uuid, InsertBearerTokenError> {
        if name.is_empty() {
            return Err(InsertBearerTokenError::EmptyName);
        }

        let mut tx = exe.ensure_transaction().await?;
        let name_exists = query!(tx.get_transaction(), (BearerToken::F.uuid,))
            .condition(BearerToken::F.name.equals(&name))
            .optional()
            .await?
            .is_some();
        if name_exists {
            return Err(InsertBearerTokenError::NameAlreadyExists);
        }

        let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);

        let uuid = insert!(tx.get_transaction(), BearerToken)
            .return_primary_key()
            .single(&BearerToken {
                uuid: Uuid::new_v4(),
                name,
                token,
            })
            .await?;

        tx.commit().await?;

        Ok(uuid)
    }
}
