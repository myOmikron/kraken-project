use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use log::{debug, trace};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

use crate::modules::service_detection::error::ResultExt;
use crate::modules::service_detection::DynResult;
use crate::utils::DebuggableBytes;

/// Settings for creating "one-shot" tcp connections i.e. which send and receive at most once.
pub struct OneShotTcpSettings {
    /// Socket to scan
    pub socket: SocketAddr,

    /// Time to wait for a response after sending the payload
    /// (or after establishing a connection, if not payload is to be sent)
    pub timeout: Duration,
}

impl OneShotTcpSettings {
    /// Send `payload` and receive answer over TCP
    ///
    /// Errors when an unrecoverable error occurred.
    /// Returns `Ok(None)` when the service refused to respond to the payload.
    pub async fn probe_tcp(&self, payload: &[u8]) -> Result<Option<Vec<u8>>, ProbeTcpError> {
        match self.raw_probe_tcp(payload).await {
            Ok(data) => Ok(Some(data)),
            Err(error) => match error.source.kind() {
                io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted => Ok(None),
                io::ErrorKind::NotConnected
                    if matches!(error.place, ProbeTcpErrorPlace::Shutdown) =>
                {
                    Ok(None)
                }
                _ => Err(error),
            },
        }
    }

    /// 1. Connect to the socket using tcp
    /// 2. Send `payload`
    /// 3. Wait for the configured amount of time
    /// 4. Return everything which has been received
    async fn raw_probe_tcp(&self, payload: &[u8]) -> Result<Vec<u8>, ProbeTcpError> {
        // Connect
        let mut tcp = TcpStream::connect(self.socket)
            .await
            .map_err(ProbeTcpErrorPlace::Connect.wrap())?;

        // Send payload
        if !payload.is_empty() {
            tcp.write_all(payload)
                .await
                .map_err(ProbeTcpErrorPlace::Write.wrap())?;
            tcp.flush()
                .await
                .map_err(ProbeTcpErrorPlace::Flush.wrap())?;
            trace!(target: "tcp", "Send data: {:?}", DebuggableBytes(payload));
        }

        // Wait
        sleep(self.timeout).await;

        // Read
        tcp.shutdown()
            .await
            .map_err(ProbeTcpErrorPlace::Shutdown.wrap())?;
        let mut data = Vec::new();
        tcp.read_to_end(&mut data)
            .await
            .map_err(ProbeTcpErrorPlace::Write.wrap())?;

        // Log and Return
        trace!(target: "tcp", "Received data: {:?}", DebuggableBytes(&data));
        Ok(data)
    }

    /// 1. Connect to the socket using tls over tcp
    /// 2. Send `payload`
    /// 3. Wait for the configured amount of time
    /// 4. Return everything which has been received
    pub async fn probe_tls(
        &self,
        payload: &[u8],
        alpn: Option<&str>,
    ) -> DynResult<Result<Vec<u8>, native_tls::Error>> {
        // Configure TLS
        let alpns = alpn.as_ref().map(std::slice::from_ref).unwrap_or(&[]);
        let connector = tokio_native_tls::TlsConnector::from(
            native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .use_sni(false)
                .request_alpns(alpns)
                .build()?,
        );

        // Connect
        let tcp = TcpStream::connect(self.socket)
            .await
            .context("TcpStream::connect")?;
        let mut tls = match connector.connect("<ignored>", tcp).await {
            Ok(tls) => tls,
            Err(err) => return Ok(Err(err)),
        };

        // Send payload
        if !payload.is_empty() {
            tls.write_all(payload)
                .await
                .context("TlsStream::write_all")?;
            tls.flush().await.context("TlsStream::flush")?;
        }

        // Wait
        sleep(self.timeout).await;

        // Read and Close
        if let Err(err) = tls.shutdown().await {
            debug!(target: "tls", "TLS shutdown failed: {err}");
        }
        let mut data = Vec::new();
        if let Err(err) = tls.read_to_end(&mut data).await {
            debug!(target: "tls", "TLS read failed: {err}");
        }

        // Log and Return
        trace!(target: "tls", "Received data: {:?}", DebuggableBytes(&data));
        Ok(Ok(data))
    }
}

/// The error returned by [`OneShotTcpSettings::probe_tcp`]
#[derive(Debug, Error)]
#[error("{source} @ {place:?}")]
pub struct ProbeTcpError {
    /// The error
    #[source]
    pub source: io::Error,

    /// The place the error occurred
    pub place: ProbeTcpErrorPlace,
}

/// The places where [`OneShotTcpSettings::probe_tcp`] might produce an error
#[derive(Debug)]
pub enum ProbeTcpErrorPlace {
    /// During [`TcpStream::connect`]
    Connect,
    /// During [`AsyncWriteExt::write_all`]
    Write,
    /// During [`AsyncWriteExt::flush`]
    Flush,
    /// During [`AsyncWriteExt::shutdown`]
    Shutdown,
    /// During [`AsyncReadExt::read_to_end`]
    Read,
}

impl ProbeTcpErrorPlace {
    fn wrap(self) -> impl FnOnce(io::Error) -> ProbeTcpError {
        |source| ProbeTcpError {
            source,
            place: self,
        }
    }
}
