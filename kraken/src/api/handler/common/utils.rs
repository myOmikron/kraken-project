use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::schema::PageParams;
use crate::api::handler::common::schema::SimpleTag;
use crate::api::handler::common::schema::TagType;
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
            color: tag.color.into(),
            tag_type: TagType::Workspace,
        }
    }
}

impl From<GlobalTag> for SimpleTag {
    fn from(tag: GlobalTag) -> Self {
        SimpleTag {
            uuid: tag.uuid,
            name: tag.name,
            color: tag.color.into(),
            tag_type: TagType::Global,
        }
    }
}
