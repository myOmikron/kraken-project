use rorm::prelude::{ForeignModel, ForeignModelByField};
use rorm::{and, insert, query, update, FieldAccess, Model, Patch};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::api::handler::ports::SimplePort;
use crate::chan::{WsMessage, GLOBAL};
use crate::models::{Host, Port, PortCertainty, PortProtocol, Workspace};
use crate::modules::aggregator::PortAggregationData;

pub async fn run_port_aggregator(
    mut rx: mpsc::Receiver<(
        PortAggregationData,
        oneshot::Sender<Result<Uuid, rorm::Error>>,
    )>,
) {
    while let Some((data, tx)) = rx.recv().await {
        let _ = tx.send(aggregate(data).await);
    }
}

#[derive(Patch)]
#[rorm(model = "Port")]
struct PortInsert {
    uuid: Uuid,
    port: i32,
    protocol: PortProtocol,
    certainty: PortCertainty,
    host: ForeignModel<Host>,
    comment: String,
    workspace: ForeignModel<Workspace>,
}

async fn aggregate(data: PortAggregationData) -> Result<Uuid, rorm::Error> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let port_uuid = if let Some((port_uuid, old_certainty)) =
        query!(&mut tx, (Port::F.uuid, Port::F.certainty))
            .condition(and![
                Port::F.port.equals(data.port as i32),
                Port::F.protocol.equals(data.protocol),
                Port::F.host.equals(data.host),
                Port::F.workspace.equals(data.workspace),
            ])
            .optional()
            .await?
    {
        if old_certainty < data.certainty {
            update!(&mut tx, Port)
                .set(Port::F.certainty, data.certainty)
                .condition(Port::F.uuid.equals(data.host))
                .await?;
        }
        port_uuid
    } else {
        let port = insert!(&mut tx, Port)
            .single(&PortInsert {
                uuid: Uuid::new_v4(),
                port: data.port as i32,
                protocol: data.protocol,
                certainty: data.certainty,
                host: ForeignModelByField::Key(data.host),
                comment: String::new(),
                workspace: ForeignModelByField::Key(data.workspace),
            })
            .await?;

        GLOBAL
            .ws
            .message_workspace(
                data.workspace,
                WsMessage::NewPort {
                    workspace: data.workspace,
                    port: SimplePort {
                        uuid: port.uuid,
                        port: data.port,
                        protocol: data.protocol,
                        host: data.host,
                        comment: String::new(),
                        workspace: data.workspace,
                        created_at: port.created_at,
                    },
                },
            )
            .await;

        port.uuid
    };

    tx.commit().await?;

    Ok(port_uuid)
}
