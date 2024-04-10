use std::cmp;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use futures::TryStreamExt;
use rorm::conditions::DynamicCollection;
use rorm::crud::query::QueryBuilder;
use rorm::db::Executor;
use rorm::internal::field::Field;
use rorm::internal::field::FieldProxy;
use rorm::model::GetField;
use rorm::prelude::*;
use rorm::query;
use uuid::Uuid;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::schema::PageParams;
use crate::api::handler::common::schema::SimpleTag;
use crate::api::handler::common::schema::TagType;
use crate::models::convert::FromDb;
use crate::models::FindingAffected;
use crate::models::FindingSeverity;
use crate::models::GlobalTag;
use crate::models::WorkspaceTag;

const QUERY_LIMIT_MAX: u64 = 1000;

pub(crate) async fn get_page_params(query: PageParams) -> Result<(u64, u64), ApiError> {
    let PageParams { limit, offset } = query;
    if limit > QUERY_LIMIT_MAX || limit == 0 {
        Err(ApiError::InvalidQueryLimit)
    } else {
        Ok((limit, offset))
    }
}

/// Query a single object's (domain, host, etc.) severity
///
/// I.e. the severest finding's severity the object is affected by
///
/// Example invocation: `query_single_severity(&mut tx, FindingAffected::F.host, host_uuid)`
pub(crate) async fn query_single_severity<FMF, PF>(
    executor: impl Executor<'_>,
    foreign_model_field: FieldProxy<FMF, FindingAffected>,
    uuid: Uuid,
) -> Result<Option<FindingSeverity>, rorm::Error>
where
    FMF: Field<Model = FindingAffected, Type = Option<ForeignModelByField<PF>>>,
    PF: Field<Type = Uuid>,
    <PF as Field>::Model: GetField<PF>, // always true
{
    query!(executor, (FindingAffected::F.finding.severity,))
        .condition(foreign_model_field.equals(uuid))
        .stream()
        .try_fold(None::<FindingSeverity>, |max, (severity,)| async move {
            Ok(Some(if let Some(max) = max {
                cmp::max(max, severity)
            } else {
                severity
            }))
        })
        .await
}

pub(crate) async fn query_many_severities<FMF, PF>(
    executor: impl Executor<'_>,
    foreign_model_field: FieldProxy<FMF, FindingAffected>,
    uuids: impl IntoIterator<Item = Uuid>,
) -> Result<HashMap<Uuid, FindingSeverity>, rorm::Error>
where
    FMF: Field<Model = FindingAffected, Type = Option<ForeignModelByField<PF>>>,
    PF: Field<Type = Uuid>,
    <PF as Field>::Model: GetField<PF>, // always true
{
    let conditions: Vec<_> = uuids
        .into_iter()
        .map(|uuid| foreign_model_field.equals(uuid))
        .collect();

    if conditions.is_empty() {
        return Ok(HashMap::new());
    }

    let mut severities = HashMap::new();
    let mut stream = QueryBuilder::new(
        executor,
        (foreign_model_field, FindingAffected::F.finding.severity),
    )
    .condition(DynamicCollection::or(conditions))
    .stream();
    while let Some((uuid, severity)) = stream.try_next().await? {
        #[allow(clippy::expect_used)]
        let uuid = *uuid.expect("We only queried those how are not null").key();
        match severities.entry(uuid) {
            Entry::Occupied(mut current_max) => {
                current_max.insert(cmp::max(*current_max.get(), severity));
            }
            Entry::Vacant(current_max) => {
                current_max.insert(severity);
            }
        }
    }
    Ok(severities)
}

/// Query all tags related to a list of aggregated results
///
/// Create a HashMap and provide it has $map
/// Provide the Transaction as $tx
/// Provide the Query for the WorkspaceTags that provides a tuple of the WorkspaceTag and the Item struct
/// Provide the field the condition should be built on for querying WorkspaceTags
/// Provide the Query for the GlobalTags that provides a tuple of the GlobalTag and the Item struct
/// Provide the field the condition should be built on for querying GlobalTags
/// Provide an iterator over the list of Item Uuids as $items
#[macro_export]
macro_rules! query_tags {
    ($map: ident, $tx: ident, $workspace_query: tt, $workspace_cond: expr, $global_query: tt, $global_cond: expr, $items: expr) => {{
        {
            let workspace_conditions: Vec<_> = $items.map(|x| $workspace_cond.equals(x)).collect();

            if !workspace_conditions.is_empty() {
                let mut workspace_tag_stream = query!(&mut $tx, $workspace_query)
                    .condition(DynamicCollection::or(workspace_conditions))
                    .stream();

                while let Some((tag, item)) = workspace_tag_stream.try_next().await? {
                    $map.entry(*item.key())
                        .or_insert(vec![])
                        .push(SimpleTag::from(tag));
                }
            }
        }

        {
            let global_conditions: Vec<_> = $items.map(|x| $global_cond.equals(x)).collect();

            if !global_conditions.is_empty() {
                let mut global_tag_stream = query!(&mut $tx, $global_query)
                    .condition(DynamicCollection::or(global_conditions))
                    .stream();

                while let Some((tag, item)) = global_tag_stream.try_next().await? {
                    $map.entry(*item.key())
                        .or_insert(vec![])
                        .push(SimpleTag::from(tag));
                }
            }
        }
    }};
}

impl From<WorkspaceTag> for SimpleTag {
    fn from(tag: WorkspaceTag) -> Self {
        SimpleTag {
            uuid: tag.uuid,
            name: tag.name,
            color: FromDb::from_db(tag.color),
            tag_type: TagType::Workspace,
        }
    }
}

impl From<GlobalTag> for SimpleTag {
    fn from(tag: GlobalTag) -> Self {
        SimpleTag {
            uuid: tag.uuid,
            name: tag.name,
            color: FromDb::from_db(tag.color),
            tag_type: TagType::Global,
        }
    }
}
