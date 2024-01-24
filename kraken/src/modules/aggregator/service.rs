use rorm::fields::traits::FieldType;
use rorm::prelude::{ForeignModel, ForeignModelByField};
use rorm::{and, insert, query, update, FieldAccess, Model, Patch};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::api::handler::services::schema::SimpleService;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::{Host, Port, Service, ServiceCertainty, Workspace};
use crate::modules::aggregator::ServiceAggregationData;

pub async fn run_service_aggregator(
    mut rx: mpsc::Receiver<(
        ServiceAggregationData,
        oneshot::Sender<Result<Uuid, rorm::Error>>,
    )>,
) {
    while let Some((data, tx)) = rx.recv().await {
        let _ = tx.send(aggregate(data).await);
    }
}

#[derive(Patch)]
#[rorm(model = "Service")]
struct ServiceInsert {
    uuid: Uuid,
    name: String,
    version: Option<String>,
    host: ForeignModel<Host>,
    port: Option<ForeignModel<Port>>,
    certainty: ServiceCertainty,
    comment: String,
    workspace: ForeignModel<Workspace>,
}

async fn aggregate(data: ServiceAggregationData) -> Result<Uuid, rorm::Error> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let service_uuid = if let Some((service_uuid, old_certainty)) =
        query!(&mut tx, (Service::F.uuid, Service::F.certainty))
            .condition(and![
                Service::F.workspace.equals(data.workspace),
                Service::F.name.equals(&data.name),
                Service::F.host.equals(data.host),
                // Sadly rorm fails at `Service::F.port.equals(port)`
                rorm::conditions::Binary {
                    operator: rorm::conditions::BinaryOperator::Equals,
                    fst_arg: rorm::conditions::Column(Service::F.port),
                    snd_arg: data.port.into_values()[0].clone(),
                },
            ])
            .optional()
            .await?
    {
        if old_certainty < data.certainty {
            update!(&mut tx, Service)
                .set(Service::F.certainty, data.certainty)
                .condition(Service::F.uuid.equals(service_uuid))
                .await?;
        }
        service_uuid
    } else {
        let service = insert!(&mut tx, Service)
            .single(&ServiceInsert {
                uuid: Uuid::new_v4(),
                name: data.name,
                version: None,
                host: ForeignModelByField::Key(data.host),
                comment: String::new(),
                workspace: ForeignModelByField::Key(data.workspace),
                port: data.port.map(ForeignModelByField::Key),
                certainty: data.certainty,
            })
            .await?;

        GLOBAL
            .ws
            .message_workspace(
                data.workspace,
                WsMessage::NewService {
                    workspace: data.workspace,
                    service: SimpleService {
                        workspace: data.workspace,
                        port: data.port,
                        host: data.host,
                        uuid: service.uuid,
                        certainty: data.certainty,
                        name: service.name,
                        comment: String::new(),
                        version: None,
                        created_at: service.created_at,
                    },
                },
            )
            .await;

        service.uuid
    };

    tx.commit().await?;

    Ok(service_uuid)
}
