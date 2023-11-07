use std::net::IpAddr;

use crate::models::ServiceCertainty;
use crate::modules::attack_results::store_service_detection_result;
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
                    1 => ServiceCertainty::MaybeVerified,
                    2 => ServiceCertainty::DefinitelyVerified,
                    _ => {
                        self.set_finished(Some(AttackError::Custom(
                            "Retrieved certainty Unknown".into(),
                        )))
                        .await;
                        return;
                    }
                };

                self.set_finished(
                    store_service_detection_result(
                        &self.db,
                        self.attack_uuid,
                        self.workspace_uuid,
                        &services,
                        certainty,
                        host.into(),
                        port,
                    )
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
}
