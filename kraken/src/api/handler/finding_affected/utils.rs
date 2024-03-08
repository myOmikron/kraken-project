use rorm::and;
use rorm::crud::query::QueryBuilder;
use rorm::crud::selector::Selector;
use rorm::db::Executor;
use rorm::or;
use rorm::prelude::*;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::models::FindingAffected;

pub async fn query_finding_affected<S: Selector<Model = FindingAffected>>(
    executor: impl Executor<'_>,
    selector: S,
    f_uuid: Uuid,
    a_uuid: Uuid,
) -> ApiResult<S::Result> {
    QueryBuilder::new(executor, selector)
        .condition(and![
            FindingAffected::F.finding.equals(f_uuid),
            or![
                FindingAffected::F.domain.equals(a_uuid),
                FindingAffected::F.host.equals(a_uuid),
                FindingAffected::F.port.equals(a_uuid),
                FindingAffected::F.service.equals(a_uuid),
            ]
        ])
        .optional()
        .await?
        .ok_or(ApiError::NotFound)
}
