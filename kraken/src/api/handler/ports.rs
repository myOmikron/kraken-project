use actix_toolbox::tb_middleware::Session;
use actix_web::get;
use actix_web::web::{Data, Json, Path};
use futures::TryStreamExt;
use rorm::{query, Database, FieldAccess, Model};
use serde::Serialize;
use tonic::codegen::tokio_stream::StreamExt;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::{workspaces, ApiError, ApiResult, PathUuid};
use crate::models::{Port, PortProtocol};

/// The simple representation of a port
#[derive(Serialize, ToSchema)]
pub struct SimplePort {
    /// Uuid of the port
    pub uuid: Uuid,
    /// Port number
    #[schema(example = 1337)]
    pub port: u16,
    /// Port protocol
    pub protocol: PortProtocol,
    /// The host this port is assigned to
    pub host: Uuid,
    /// A comment to the port
    pub comment: String,
}

/// All ports of a workspace
#[derive(Serialize, ToSchema)]
pub struct GetAllPortsResponse {
    ports: Vec<SimplePort>,
}

/// List the ports of a workspace
#[utoipa::path(
    tag = "Ports",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve all ports of a workspace", body = GetAllPortsResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/ports")]
pub async fn get_all_ports(
    path: Path<PathUuid>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<GetAllPortsResponse>> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;
    let path = path.into_inner();

    let mut tx = db.start_transaction().await?;

    if !workspaces::is_user_member_or_owner(&mut tx, user_uuid, path.uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let ports: Vec<_> = query!(&mut tx, Port)
        .condition(Port::F.workspace.equals(path.uuid))
        .stream()
        .map(|x| -> ApiResult<SimplePort> {
            let x = x?;
            Ok(SimplePort {
                uuid: x.uuid,
                port: u16::from_ne_bytes(x.port.to_ne_bytes()),
                protocol: x.protocol,
                comment: x.comment,
                host: *x.host.key(),
            })
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(GetAllPortsResponse { ports }))
}
