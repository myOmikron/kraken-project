use std::collections::HashMap;

use rorm::and;
use rorm::conditions::Condition;
use rorm::db::transaction::Transaction;
use rorm::prelude::*;
use rorm::query;
use uuid::Uuid;

pub use crate::api::handler::common::error::ApiError;
pub use crate::api::handler::common::error::ApiResult;
use crate::api::handler::findings::schema::ListFindings;
use crate::api::handler::findings::schema::SimpleFinding;
use crate::api::handler::findings::schema::SimpleFindingAffected;
use crate::chan::ws_manager::schema::AggregationType;
use crate::models::convert::FromDb;
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
    pub async fn query_through_affected<'exe: 'co, 'co>(
        tx: &'exe mut Transaction,
        workspace: Uuid,
        condition: impl Condition<'co>,
    ) -> Result<ListFindings, rorm::Error> {
        let mut affected_lookup = HashMap::new();

        let affected = query!(&mut *tx, (FindingAffected::F.finding,))
            .condition(FindingAffected::F.workspace.equals(workspace))
            .all()
            .await?;
        for (finding,) in affected {
            affected_lookup
                .entry(*finding.key())
                .and_modify(|x| *x += 1)
                .or_insert(1);
        }

        let findings = query!(
            &mut *tx,
            (
                FindingAffected::F.finding.uuid,
                FindingAffected::F.finding.definition.uuid,
                FindingAffected::F.finding.definition.name,
                FindingAffected::F.finding.definition.cve,
                FindingAffected::F.finding.severity,
                FindingAffected::F.finding.created_at,
            )
        )
        .condition(and![
            condition,
            FindingAffected::F.workspace.equals(workspace)
        ])
        .all()
        .await?;

        let simple_findings = findings
            .into_iter()
            .map(
                |(uuid, definition, name, cve, severity, created_at)| SimpleFinding {
                    uuid,
                    definition,
                    name,
                    cve,
                    severity: FromDb::from_db(severity),
                    created_at,
                    affected_count: affected_lookup.get(&uuid).copied().unwrap_or(0),
                },
            )
            .collect();

        Ok(ListFindings {
            findings: simple_findings,
        })
    }
}
