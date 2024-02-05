use rorm::db::Executor;
use rorm::{insert, query, FieldAccess, Model, Patch};
use thiserror::Error;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::models::{FindingDefinition, FindingSeverity};

impl FindingDefinition {
    /// Insert a new [FindingDefinition]
    pub(crate) async fn insert(
        executor: impl Executor<'_>,
        name: String,
        summary: String,
        severity: FindingSeverity,
        cve: Option<String>,
        description: String,
        impact: String,
        remediation: String,
        references: String,
    ) -> Result<FindingDefinition, FindingDefinitionInsertError> {
        let uuid = Uuid::new_v4();

        let mut guard = executor.ensure_transaction().await?;

        if query!(guard.get_transaction(), (FindingDefinition::F.uuid,))
            .condition(FindingDefinition::F.name.equals(&name))
            .optional()
            .await?
            .is_some()
        {
            return Err(FindingDefinitionInsertError::NameAlreadyExists);
        }

        let fd = insert!(guard.get_transaction(), InsertFindingDefinition)
            .single(&InsertFindingDefinition {
                uuid,
                name,
                summary,
                severity,
                cve,
                description,
                impact,
                remediation,
                references,
            })
            .await?;

        guard.commit().await?;

        Ok(fd)
    }
}

#[derive(Patch)]
#[rorm(model = "FindingDefinition")]
struct InsertFindingDefinition {
    uuid: Uuid,
    name: String,
    summary: String,
    severity: FindingSeverity,
    cve: Option<String>,
    description: String,
    impact: String,
    remediation: String,
    references: String,
}

/// The errors that can occur when inserting a new [FindingDefinition]
#[derive(Debug, Error)]
pub enum FindingDefinitionInsertError {
    #[error("Database error: {0}")]
    Database(#[from] rorm::Error),
    #[error("Name already exists")]
    NameAlreadyExists,
}

impl From<FindingDefinitionInsertError> for ApiError {
    fn from(value: FindingDefinitionInsertError) -> Self {
        match value {
            FindingDefinitionInsertError::Database(x) => ApiError::DatabaseError(x),
            FindingDefinitionInsertError::NameAlreadyExists => ApiError::NameAlreadyExists,
        }
    }
}
