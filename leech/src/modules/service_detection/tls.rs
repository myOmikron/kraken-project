use std::net::SocketAddr;
use std::slice;
use std::time::Duration;

use log::trace;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_native_tls::{native_tls, TlsConnector};

use super::{DebuggableBytes, DynResult};

pub async fn probe(
    socket: SocketAddr,
    payload: &[u8],
    alpn: Option<&str>,
) -> DynResult<Result<Vec<u8>, native_tls::Error>> {
    let alpns = alpn.as_ref().map(slice::from_ref).unwrap_or(&[]);
    let connector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .use_sni(false)
            .request_alpns(alpns)
            .build()?,
    );
    let tcp = TcpStream::connect(socket).await?;
    let mut tls = match connector.connect("<ignored>", tcp).await {
        Ok(tls) => tls,
        Err(err) => return Ok(Err(err)),
    };

    tls.write_all(payload).await?;
    sleep(Duration::from_secs(1)).await;
    tls.shutdown().await?;

    let mut data = Vec::new();
    tls.read_to_end(&mut data).await?;
    trace!(target: "tls", "Got data: {:?}", DebuggableBytes(&data));

    Ok(Ok(data))
}
