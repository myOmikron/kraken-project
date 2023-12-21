use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use uuid::Uuid;

use crate::models::TestSSLResult;
use crate::rpc::rpc_definitions::TestSslResponse;

/// Store a query certificate transparency's result and update the aggregated domains and hosts
pub async fn store_testssl_result(
    executor: impl Executor<'_>,
    attack_uuid: Uuid,
    workspace_uuid: Uuid,
    result: TestSslResponse,
) -> Result<(), rorm::Error> {
    insert!(executor, TestSSLResult)
        .return_nothing()
        .single(&TestSSLResult {
            uuid: Uuid::new_v4(),
            attack: ForeignModelByField::Key(attack_uuid),
        })
        .await
    // TODO
}
