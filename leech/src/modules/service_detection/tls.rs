use std::net::SocketAddr;

use log::trace;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_native_tls::{native_tls, TlsConnector};

use super::{DebuggableBytes, DynResult};

pub async fn probe(
    socket: SocketAddr,
    payload: &[u8],
) -> DynResult<Result<Vec<u8>, native_tls::Error>> {
    let connector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .use_sni(false)
            .build()?,
    );
    let tcp = TcpStream::connect(socket).await?;
    let mut tls = match connector.connect("<ignored>", tcp).await {
        Ok(tls) => tls,
        Err(err) => return Ok(Err(err)),
    };

    tls.write_all(payload).await?;
    tls.shutdown().await?;

    let mut data = Vec::new();
    tls.read_to_end(&mut data).await?;
    trace!(target: "tls", "Got data: {:?}", DebuggableBytes(&data));

    Ok(Ok(data))
}
