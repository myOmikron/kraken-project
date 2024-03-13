use futures::TryStreamExt;
use rorm::and;
use rorm::conditions::Condition;
use rorm::db::Executor;
use rorm::prelude::*;
use rorm::query;
use uuid::Uuid;

pub use crate::api::handler::common::error::ApiError;
pub use crate::api::handler::common::error::ApiResult;
use crate::api::handler::findings::schema::ListFindings;
use crate::api::handler::findings::schema::SimpleFinding;
use crate::api::handler::findings::schema::SimpleFindingAffected;
use crate::chan::ws_manager::schema::AggregationType;
use crate::models::FindingAffected;

/// Convert a [`FindingAffected`] into a [`SimpleFindingAffected`]
///
/// # Errors
/// if the database state is invalid i.e. more than one or zero objects are set in [`FindingAffected`].
pub fn finding_affected_into_simple(affected: FindingAffected) -> ApiResult<SimpleFindingAffected> {
    let (affected_type, &affected_uuid) = match &affected {
        FindingAffected {
            domain: Some(obj),
            host: None,
            port: None,
            service: None,
            ..
        } => Ok((AggregationType::Domain, obj.key())),
        FindingAffected {
            domain: None,
            host: Some(obj),
            port: None,
            service: None,
            ..
        } => Ok((AggregationType::Host, obj.key())),
        FindingAffected {
            domain: None,
            host: None,
            port: Some(obj),
            service: None,
            ..
        } => Ok((AggregationType::Port, obj.key())),
        FindingAffected {
            domain: None,
            host: None,
            port: None,
            service: Some(obj),
            ..
        } => Ok((AggregationType::Service, obj.key())),
        FindingAffected {
            domain: None,
            host: None,
            port: None,
            service: None,
            ..
        } => Err(ApiError::InternalServerError),
        _ => Err(ApiError::InternalServerError),
    }?;
    Ok(SimpleFindingAffected {
        finding: *affected.finding.key(),
        affected_type,
        affected_uuid,
    })
}

impl ListFindings {
    /// Query all findings affecting an object
    pub async fn query_through_affected<'ex: 'co, 'co>(
        executor: impl Executor<'ex>,
        workspace: Uuid,
        condition: impl Condition<'co>,
    ) -> Result<ListFindings, rorm::Error> {
        query!(
            executor,
            (
                FindingAffected::F.finding.uuid,
                FindingAffected::F.finding.definition.uuid,
                FindingAffected::F.finding.definition.name,
                FindingAffected::F.finding.definition.cve,
                FindingAffected::F.finding.severity,
                FindingAffected::F.finding.created_at
            )
        )
        .condition(and![
            condition,
            FindingAffected::F.workspace.equals(workspace)
        ])
        .stream()
        .and_then(|(uuid, definition, name, cve, severity, created_at)| {
            std::future::ready(Ok(SimpleFinding {
                uuid,
                definition,
                name,
                cve,
                severity,
                created_at,
            }))
        })
        .try_collect()
        .await
        .map(|findings| ListFindings { findings })
    }
}
