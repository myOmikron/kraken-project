use rorm::and;
use rorm::fields::traits::FieldType;
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

use crate::api::handler::http_services::schema::SimpleHttpService;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::convert::FromDb;
use crate::models::Domain;
use crate::models::Host;
use crate::models::HttpService;
use crate::models::HttpServiceCertainty;
use crate::models::Port;
use crate::models::Workspace;
use crate::modules::aggregator::HttpServiceAggregationData;

pub async fn run_http_service_aggregator(
    mut rx: mpsc::Receiver<(
        HttpServiceAggregationData,
        oneshot::Sender<Result<Uuid, rorm::Error>>,
    )>,
) {
    while let Some((data, tx)) = rx.recv().await {
        let _ = tx.send(aggregate(data).await);
    }
}

#[derive(Patch)]
#[rorm(model = "HttpService")]
struct HttpServiceInsert {
    uuid: Uuid,
    name: String,
    version: Option<String>,
    host: ForeignModel<Host>,
    port: ForeignModel<Port>,
    domain: Option<ForeignModel<Domain>>,
    base_path: String,
    tls: bool,
    sni_required: bool,
    certainty: HttpServiceCertainty,
    comment: String,
    workspace: ForeignModel<Workspace>,
}

async fn aggregate(data: HttpServiceAggregationData) -> Result<Uuid, rorm::Error> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let existing = query!(&mut tx, (HttpService::F.uuid, HttpService::F.certainty,))
        .condition(and![
            HttpService::F.workspace.equals(data.workspace),
            HttpService::F.name.equals(&data.name),
            HttpService::F.host.equals(&data.host),
            HttpService::F.port.equals(&data.port),
            // Sadly rorm fails at `HttpService::F.domain.equals(domain)`
            rorm::conditions::Binary {
                operator: rorm::conditions::BinaryOperator::Equals,
                fst_arg: rorm::conditions::Column(HttpService::F.port),
                snd_arg: data.domain.into_values()[0].clone(),
            },
            HttpService::F.base_path.equals(&data.base_path),
            HttpService::F.tls.equals(data.tls),
            HttpService::F.sni_required.equals(data.sni_required),
        ])
        .optional()
        .await?;

    let http_service_uuid = if let Some((uuid, old_certainty)) = existing {
        if old_certainty < data.certainty {
            update!(&mut tx, HttpService)
                .set(HttpService::F.certainty, data.certainty)
                .condition(HttpService::F.uuid.equals(uuid))
                .await?;
        }
        uuid
    } else {
        let http_service = insert!(&mut tx, HttpService)
            .single(&HttpServiceInsert {
                uuid: Uuid::new_v4(),
                name: data.name,
                version: data.version.clone(),
                host: ForeignModelByField::Key(data.host),
                port: ForeignModelByField::Key(data.port),
                domain: data.domain.map(ForeignModelByField::Key),
                base_path: data.base_path,
                tls: data.tls,
                sni_required: data.sni_required,
                certainty: data.certainty,
                comment: String::new(),
                workspace: ForeignModelByField::Key(data.workspace),
            })
            .await?;

        GLOBAL
            .ws
            .message_workspace(
                data.workspace,
                WsMessage::NewHttpService {
                    workspace: data.workspace,
                    http_service: SimpleHttpService {
                        uuid: http_service.uuid,
                        name: http_service.name,
                        version: http_service.version,
                        domain: http_service.domain.map(|fm| *fm.key()),
                        host: *http_service.host.key(),
                        port: *http_service.port.key(),
                        base_path: http_service.base_path,
                        tls: http_service.tls,
                        sni_required: http_service.sni_required,
                        comment: http_service.comment,
                        certainty: FromDb::from_db(http_service.certainty),
                        workspace: *http_service.workspace.key(),
                        created_at: http_service.created_at,
                    },
                },
            )
            .await;

        http_service.uuid
    };

    tx.commit().await?;

    Ok(http_service_uuid)
}
