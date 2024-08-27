use futures::TryStreamExt;
use rorm::db::Executor;
use rorm::fields::traits::FieldEq;
use rorm::internal::field::Field;
use rorm::internal::field::FieldProxy;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::handler::finding_categories::schema::SimpleFindingCategory;
use crate::models::convert::FromDb;
use crate::models::FindingCategory;

impl SimpleFindingCategory {
    /// Query the list of `SimpleFindingCategory`s for a single `FindingDefinition` or `Finding`
    pub async fn query_for_single<M2M, FC, FOD, Any>(
        executor: impl Executor<'_>,
        finding_category: FieldProxy<FC, M2M>,
        finding_or_definition: FieldProxy<FOD, M2M>,
        uuid: Uuid,
    ) -> Result<Vec<Self>, rorm::Error>
    where
        M2M: Model,
        FC: Field<Model = M2M, Type = ForeignModelByField<<FindingCategory as Model>::Primary>>,
        FOD: Field<Model = M2M>,
        FOD::Type: for<'rhs> FieldEq<'rhs, Uuid, Any>,
    {
        query!(
            executor,
            (
                finding_category.uuid,
                finding_category.name,
                finding_category.color,
            )
        )
        .condition(finding_or_definition.equals(uuid))
        .stream()
        .map_ok(|(uuid, name, color)| SimpleFindingCategory {
            uuid,
            name,
            color: FromDb::from_db(color),
        })
        .try_collect()
        .await
    }
}
