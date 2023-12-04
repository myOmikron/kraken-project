use ipnetwork::IpNetwork;
use rorm::prelude::{ForeignModel, ForeignModelByField};
use rorm::{and, insert, query, update, FieldAccess, Model, Patch};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::chan::GLOBAL;
use crate::models::{Host, HostCertainty, OsType, Workspace};
use crate::modules::aggregator::HostAggregationData;

pub async fn run_host_aggregator(
    mut rx: mpsc::Receiver<(
        HostAggregationData,
        oneshot::Sender<Result<Uuid, rorm::Error>>,
    )>,
) {
    while let Some((data, tx)) = rx.recv().await {
        let _ = tx.send(aggregate(data).await);
    }
}
#[derive(Patch)]
#[rorm(model = "Host")]
struct HostInsert {
    uuid: Uuid,
    ip_addr: IpNetwork,
    os_type: OsType,
    response_time: Option<i32>,
    comment: String,
    certainty: HostCertainty,
    workspace: ForeignModel<Workspace>,
}

async fn aggregate(data: HostAggregationData) -> Result<Uuid, rorm::Error> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let uuid = if let Some((uuid, old_certainty)) =
        query!(&mut tx, (Host::F.uuid, Host::F.certainty))
            .condition(and![
                Host::F.ip_addr.equals(data.ip_addr),
                Host::F.workspace.equals(data.workspace)
            ])
            .optional()
            .await?
    {
        if old_certainty < data.certainty {
            update!(&mut tx, Host)
                .set(Host::F.certainty, data.certainty)
                .condition(Host::F.uuid.equals(uuid))
                .await?;
        }
        uuid
    } else {
        insert!(&mut tx, HostInsert)
            .return_primary_key()
            .single(&HostInsert {
                uuid: Uuid::new_v4(),
                ip_addr: data.ip_addr,
                os_type: OsType::Unknown,
                response_time: None,
                comment: String::new(),
                certainty: HostCertainty::Verified,
                workspace: ForeignModelByField::Key(data.workspace),
            })
            .await?
    };

    tx.commit().await?;

    Ok(uuid)
}
