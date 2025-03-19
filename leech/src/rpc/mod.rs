//! The gRPC part of the leech.
//!
//! In server mode, the leech has a grpc server running to receive requests from kraken.
//! If the connection drops or the leech can't send the data, it will be saved in the local
//! database and pushing the data to the server is tried regularly.
//!
//! In cli mode, the leech can push the results to kraken if desired.

use std::error::Error;
use std::net::SocketAddr;

use kraken_proto::req_attack_service_server::ReqAttackServiceServer;
use log::info;
use tonic::transport::Identity;
use tonic::transport::Server;
use tonic::transport::ServerTlsConfig;

use crate::backlog::Backlog;
use crate::config::GRPC_LISTEN_ADDRESS;
use crate::config::GRPC_LISTEN_PORT;
use crate::config::LEECH_CERT;
use crate::config::LEECH_KEY;
use crate::rpc::attacks::Attacks;

pub mod attacks;

/// Starts the gRPC server
///
/// **Parameter**:
/// - `config`: Reference to [Config]
pub async fn start_rpc_server(backlog: Backlog) -> Result<(), Box<dyn Error>> {
    info!("Starting Server");
    Server::builder()
        .tls_config(
            ServerTlsConfig::new().identity(Identity::from_pem(LEECH_CERT.get(), LEECH_KEY.get())),
        )?
        .add_service(ReqAttackServiceServer::new(Attacks { backlog }))
        .serve(SocketAddr::new(*GRPC_LISTEN_ADDRESS, *GRPC_LISTEN_PORT))
        .await?;

    Ok(())
}
