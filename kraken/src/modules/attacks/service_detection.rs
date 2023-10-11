use std::net::IpAddr;

use ipnetwork::IpNetwork;
use rorm::insert;
use rorm::prelude::*;
use uuid::Uuid;

use crate::models::{Certainty, Service, ServiceDetectionName, ServiceDetectionResultInsert};
use crate::modules::attacks::{AttackError, LeechAttackContext};
use crate::rpc::rpc_definitions::{ServiceDetectionRequest, ServiceDetectionResponse};

impl LeechAttackContext {
    /// Check what services are running on a specific port
    ///
    /// See [`handler::attacks::service_detection`] for more information.
    pub async fn service_detection(
        mut self,
        req: ServiceDetectionRequest,
        host: IpAddr,
        port: u16,
    ) {
        match self.leech.service_detection(req).await {
            Ok(v) => {
                let ServiceDetectionResponse {
                    services,
                    response_type,
                } = v.into_inner();

                let certainty = match response_type {
                    1 => Certainty::Maybe,
                    2 => Certainty::Definitely,
                    _ => Certainty::Unknown,
                };

                self.set_finished(
                    self.insert_service_detection_result(&services, certainty, host.into(), port)
                        .await
                        .map_err(AttackError::from)
                        .err(),
                )
                .await;
            }
            Err(status) => {
                self.set_finished(Some(AttackError::Grpc(status))).await;
            }
        }
    }

    async fn insert_service_detection_result(
        &self,
        service_names: &[String],
        certainty: Certainty,
        host: IpNetwork,
        port: u16,
    ) -> Result<(), rorm::Error> {
        let port: i16 = i16::from_ne_bytes(port.to_ne_bytes());

        let mut tx = self.db.start_transaction().await?;

        let uuid = insert!(&mut tx, ServiceDetectionResultInsert)
            .return_primary_key()
            .single(&ServiceDetectionResultInsert {
                uuid: Uuid::new_v4(),
                attack: ForeignModelByField::Key(self.attack_uuid),
                certainty,
                host,
                port,
            })
            .await?;

        insert!(&mut tx, ServiceDetectionName)
            .return_nothing()
            .bulk(
                &service_names
                    .iter()
                    .map(|x| ServiceDetectionName {
                        uuid: Uuid::new_v4(),
                        name: x.to_string(),
                        result: ForeignModelByField::Key(uuid),
                    })
                    .collect::<Vec<_>>(),
            )
            .await?;

        for x in service_names {
            Service::update_or_insert(&mut tx, self.workspace_uuid, x, host, Some(port), certainty)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
