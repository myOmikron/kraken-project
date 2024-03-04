use ipnetwork::IpNetwork;
use rorm::and;
use rorm::insert;
use rorm::prelude::ForeignModel;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;
use rorm::Patch;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::api::handler::hosts::schema::SimpleHost;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::Host;
use crate::models::HostCertainty;
use crate::models::OsType;
use crate::models::Workspace;
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
        if let Some(os_type) = data.os_type {
            update!(&mut tx, Host)
                .set(Host::F.os_type, os_type)
                .condition(Host::F.uuid.equals(uuid))
                .await?;
        }
        uuid
    } else {
        let host = insert!(&mut tx, HostInsert)
            .single(&HostInsert {
                uuid: Uuid::new_v4(),
                ip_addr: data.ip_addr,
                os_type: data.os_type.unwrap_or(OsType::Unknown),
                response_time: None,
                comment: String::new(),
                certainty: data.certainty,
                workspace: ForeignModelByField::Key(data.workspace),
            })
            .await?;

        GLOBAL
            .ws
            .message_workspace(
                data.workspace,
                WsMessage::NewHost {
                    workspace: data.workspace,
                    host: SimpleHost {
                        uuid: host.uuid,
                        ip_addr: host.ip_addr.ip(),
                        os_type: host.os_type,
                        response_time: host.response_time,
                        certainty: host.certainty,
                        comment: host.comment,
                        workspace: *host.workspace.key(),
                        created_at: host.created_at,
                    },
                },
            )
            .await;

        host.uuid
    };

    tx.commit().await?;

    Ok(uuid)
}
