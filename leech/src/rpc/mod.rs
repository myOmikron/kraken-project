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
use tonic::transport::{Identity, Server, ServerTlsConfig};

use crate::backlog::Backlog;
use crate::config::Config;
use crate::rpc::attacks::Attacks;

pub mod attacks;

/// Starts the gRPC server
///
/// **Parameter**:
/// - `config`: Reference to [Config]
pub async fn start_rpc_server(config: &Config, backlog: Backlog) -> Result<(), Box<dyn Error>> {
    info!("Starting Server");
    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(Identity::from_pem(
            &config.kraken.leech_cert,
            &config.kraken.leech_key,
        )))?
        .add_service(ReqAttackServiceServer::new(Attacks { backlog }))
        .serve(SocketAddr::new(
            config.server.listen_address.parse().unwrap(),
            config.server.listen_port,
        ))
        .await?;

    Ok(())
}
