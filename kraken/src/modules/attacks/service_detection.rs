use crate::chan::{LeechClient, GLOBAL};
use crate::models::ServiceCertainty;
use crate::modules::attack_results::store_service_detection_result;
use crate::modules::attacks::{AttackContext, AttackError, ServiceDetectionParams};
use crate::rpc::rpc_definitions::{ServiceDetectionRequest, ServiceDetectionResponse};

impl AttackContext {
    /// Executes the "service detection" attack
    pub async fn service_detection(
        &self,
        mut leech: LeechClient,
        params: ServiceDetectionParams,
    ) -> Result<(), AttackError> {
        let request = ServiceDetectionRequest {
            attack_uuid: self.attack_uuid.to_string(),
            address: Some(params.target.into()),
            port: params.port as u32,
            timeout: params.timeout,
        };
        let ServiceDetectionResponse {
            services,
            response_type,
        } = leech.service_detection(request).await?.into_inner();

        let certainty = match response_type {
            1 => ServiceCertainty::MaybeVerified,
            2 => ServiceCertainty::DefinitelyVerified,
            _ => {
                return Err(AttackError::Custom("Retrieved certainty Unknown".into()));
            }
        };

        store_service_detection_result(
            &GLOBAL.db,
            self.attack_uuid,
            self.workspace_uuid,
            &services,
            certainty,
            params.target.into(),
            params.port,
        )
        .await?;

        Ok(())
    }
}
