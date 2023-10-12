use rorm::db::Executor;
use rorm::prelude::*;
use rorm::{insert, or, query, update};
use thiserror::Error;
use uuid::Uuid;

use crate::api::handler::ApiError;
use crate::models::WordList;

impl WordList {
    /// Insert a new wordlist checking the two unique constraints on `name` and `path`
    pub async fn insert(
        executor: impl Executor<'_>,
        name: String,
        description: String,
        path: String,
    ) -> Result<Uuid, InsertWordlistError> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        let uuid = Uuid::new_v4();
        if let Some(wordlist) = query!(&mut *tx, WordList)
            .condition(or![
                WordList::F.name.equals(&name),
                WordList::F.path.equals(&description)
            ])
            .optional()
            .await?
        {
            if wordlist.name == name {
                return Err(InsertWordlistError::NameAlreadyExists);
            }
            if wordlist.path == path {
                return Err(InsertWordlistError::PathAlreadyExists);
            }
            unreachable!("A wordlist should only have been queried it its name or path matches");
        } else {
            insert!(&mut *tx, WordList)
                .return_nothing()
                .single(&WordList {
                    uuid,
                    name,
                    description,
                    path,
                })
                .await?;
        }

        guard.commit().await?;
        Ok(uuid)
    }

    /// Update an existing wordlist checking the two unique constraints on `name` and `path`
    pub async fn update(
        executor: impl Executor<'_>,
        uuid: Uuid,
        name: Option<String>,
        description: Option<String>,
        path: Option<String>,
    ) -> Result<(), UpdateWordlistError> {
        let mut guard = executor.ensure_transaction().await?;
        let tx = guard.get_transaction();

        if let Some(name) = name.as_ref() {
            if query!(&mut *tx, (WordList::F.uuid,))
                .condition(WordList::F.name.equals(name))
                .optional()
                .await?
                .is_some()
            {
                return Err(UpdateWordlistError::NameAlreadyExists);
            }
        }
        if let Some(path) = path.as_ref() {
            if query!(&mut *tx, (WordList::F.uuid,))
                .condition(WordList::F.path.equals(path))
                .optional()
                .await?
                .is_some()
            {
                return Err(UpdateWordlistError::PathAlreadyExists);
            }
        }

        update!(&mut *tx, WordList)
            .begin_dyn_set()
            .set_if(WordList::F.name, name)
            .set_if(WordList::F.description, description)
            .set_if(WordList::F.path, path)
            .finish_dyn_set()
            .map_err(|_| UpdateWordlistError::NothingToDo)?
            .condition(WordList::F.uuid.equals(uuid))
            .await?;

        guard.commit().await?;
        Ok(())
    }
}

/// The errors that can occur while inserting a wordlist
#[derive(Error, Debug)]
pub enum InsertWordlistError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rorm::Error),
    #[error("The address exists already")]
    NameAlreadyExists,
    #[error("The name exists already")]
    PathAlreadyExists,
}

impl From<InsertWordlistError> for ApiError {
    fn from(value: InsertWordlistError) -> Self {
        match value {
            InsertWordlistError::DatabaseError(error) => ApiError::DatabaseError(error),
            InsertWordlistError::NameAlreadyExists => ApiError::NameAlreadyExists,
            InsertWordlistError::PathAlreadyExists => ApiError::PathAlreadyExists,
        }
    }
}

/// The errors that can occur while updating a wordlist
#[derive(Error, Debug)]
pub enum UpdateWordlistError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rorm::Error),
    #[error("The address exists already")]
    NameAlreadyExists,
    #[error("The name exists already")]
    PathAlreadyExists,
    #[error("No fields to update were provided")]
    NothingToDo,
}

impl From<UpdateWordlistError> for ApiError {
    fn from(value: UpdateWordlistError) -> Self {
        match value {
            UpdateWordlistError::DatabaseError(error) => ApiError::DatabaseError(error),
            UpdateWordlistError::NameAlreadyExists => ApiError::NameAlreadyExists,
            UpdateWordlistError::PathAlreadyExists => ApiError::PathAlreadyExists,
            UpdateWordlistError::NothingToDo => ApiError::EmptyJson,
        }
    }
}
