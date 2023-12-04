use log::warn;
use rorm::prelude::{ForeignModel, ForeignModelByField};
use rorm::{and, insert, query, update, FieldAccess, Model, Patch};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::chan::GLOBAL;
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
        let _ = tx.send(aggregate(data).await);
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

async fn aggregate(data: DomainAggregationData) -> Result<Uuid, rorm::Error> {
    let mut tx = GLOBAL.db.start_transaction().await?;

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
        let domain_uuid = insert!(&mut tx, Domain)
            .return_primary_key()
            .single(&DomainInsert {
                uuid: Uuid::new_v4(),
                domain: data.domain.clone(),
                certainty: data.certainty,
                comment: String::new(),
                workspace: ForeignModelByField::Key(data.workspace),
            })
            .await?;

        if let Ok(leech) = GLOBAL.leeches.random_leech() {
            start_dns_resolution(
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
                InsertAttackError::WorkspaceInvalid => {
                    unreachable!("Workspace already used above")
                }
            })?;
        } else {
            warn!(
                "Couldn't resolve new domain \"{domain}\" automatically: No leech",
                domain = data.domain
            );
        }
        domain_uuid
    };

    tx.commit().await?;

    Ok(uuid)
}
