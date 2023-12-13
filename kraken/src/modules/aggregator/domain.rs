use log::warn;
use rorm::prelude::{ForeignModel, ForeignModelByField};
use rorm::{and, insert, query, update, FieldAccess, Model, Patch};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::api::handler::domains::SimpleDomain;
use crate::chan::{WsMessage, GLOBAL};
use crate::models::{Domain, DomainCertainty, InsertAttackError, Workspace};
use crate::modules::aggregator::DomainAggregationData;
use crate::modules::attacks::{start_dns_resolution, DnsResolutionParams};

pub async fn run_domain_aggregator(
    mut rx: mpsc::Receiver<(
        DomainAggregationData,
        oneshot::Sender<Result<Uuid, rorm::Error>>,
    )>,
) {
    while let Some((data, tx)) = rx.recv().await {
        match aggregate(data).await {
            Ok((uuid, None)) => {
                let _ = tx.send(Ok(uuid));
            }
            Ok((uuid, Some(attack))) => {
                // Await the attack in a new task to avoid blocking the aggregator
                tokio::spawn(async move {
                    let _ = attack.await;
                    let _ = tx.send(Ok(uuid));
                });
            }
            Err(error) => {
                let _ = tx.send(Err(error));
            }
        }
    }
}

#[derive(Patch)]
#[rorm(model = "Domain")]
struct DomainInsert {
    uuid: Uuid,
    domain: String,
    certainty: DomainCertainty,
    comment: String,
    workspace: ForeignModel<Workspace>,
}

async fn aggregate(
    data: DomainAggregationData,
) -> Result<(Uuid, Option<JoinHandle<()>>), rorm::Error> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let mut attack_handle = None;
    let uuid = if let Some((uuid, old_certainty)) =
        query!(&mut tx, (Domain::F.uuid, Domain::F.certainty))
            .condition(and![
                Domain::F.workspace.equals(data.workspace),
                Domain::F.domain.equals(&data.domain)
            ])
            .optional()
            .await?
    {
        if old_certainty < data.certainty {
            update!(&mut tx, Domain)
                .set(Domain::F.certainty, data.certainty)
                .condition(Domain::F.uuid.equals(uuid))
                .await?;
        }
        uuid
    } else {
        let domain = insert!(&mut tx, Domain)
            .single(&DomainInsert {
                uuid: Uuid::new_v4(),
                domain: data.domain.clone(),
                certainty: data.certainty,
                comment: String::new(),
                workspace: ForeignModelByField::Key(data.workspace),
            })
            .await?;

        GLOBAL
            .ws
            .message_workspace(
                data.workspace,
                WsMessage::NewDomain {
                    workspace: data.workspace,
                    domain: SimpleDomain {
                        uuid: domain.uuid,
                        domain: domain.domain,
                        comment: domain.comment,
                        workspace: *domain.workspace.key(),
                        created_at: domain.created_at,
                    },
                },
            )
            .await;

        if let Ok(leech) = GLOBAL.leeches.random_leech() {
            let (_, handle) = start_dns_resolution(
                data.workspace,
                data.user,
                leech,
                DnsResolutionParams {
                    targets: vec![data.domain.clone()],
                    concurrent_limit: 1,
                },
            )
            .await
            .map_err(|err| match err {
                InsertAttackError::DatabaseError(err) => err,
                InsertAttackError::WorkspaceInvalid => unreachable!("Workspace already used above"),
                InsertAttackError::UserInvalid => unreachable!("User already used above"),
            })?;
            attack_handle = Some(handle);
        } else {
            warn!(
                "Couldn't resolve new domain \"{domain}\" automatically: No leech",
                domain = data.domain
            );
        }
        domain.uuid
    };

    tx.commit().await?;
    Ok((uuid, attack_handle))
}
