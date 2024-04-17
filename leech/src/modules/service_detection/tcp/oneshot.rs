use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use log::debug;
use log::trace;
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::utils::DebuggableBytes;

/// Settings for creating "one-shot" tcp connections i.e. which send and receive at most once.
pub struct OneShotTcpSettings {
    /// Socket to scan
    pub socket: SocketAddr,

    /// Time to wait for a connection to establish
    pub connect_timeout: Duration,

    /// Time to wait for a response after sending the payload
    /// (or after establishing a connection, if not payload is to be sent)
    pub recv_timeout: Duration,
}

impl OneShotTcpSettings {
    /// Send `payload` and receive answer over TCP
    ///
    /// Errors when an unrecoverable error occurred.
    /// Returns `Ok(None)` when the service refused to respond to the payload.
    pub async fn probe_tcp(&self, payload: &[u8]) -> Result<Option<Vec<u8>>, ProbeTcpError> {
        let mut place = ProbeTcpErrorPlace::Connect;
        match self.raw_probe_tcp(payload, &mut place).await {
            Ok(data) => Ok(Some(data)),
            Err(error) => match error.kind() {
                io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted => Ok(None),
                io::ErrorKind::NotConnected if matches!(place, ProbeTcpErrorPlace::Shutdown) => {
                    Ok(None)
                }
                // TODO a connection timeout might be interesting during detection
                //      since the scanner has already detected the post to be open once.
                io::ErrorKind::TimedOut if matches!(place, ProbeTcpErrorPlace::Connect) => Ok(None),
                _ => Err(ProbeTcpError {
                    source: error,
                    place,
                }),
            },
        }
    }

    /// 1. Connect to the socket using tcp
    /// 2. Send `payload`
    /// 3. Wait for the configured amount of time
    /// 4. Return everything which has been received
    async fn raw_probe_tcp(
        &self,
        payload: &[u8],
        place: &mut ProbeTcpErrorPlace,
    ) -> Result<Vec<u8>, io::Error> {
        // Connect
        *place = ProbeTcpErrorPlace::Connect;
        let mut tcp = timeout(self.connect_timeout, TcpStream::connect(self.socket)).await??;

        let mut data = Vec::new();
        if let Ok(Err(error)) = timeout(self.recv_timeout, async {
            // Send payload
            if !payload.is_empty() {
                *place = ProbeTcpErrorPlace::Write;
                tcp.write_all(payload).await?;

                *place = ProbeTcpErrorPlace::Flush;
                tcp.flush().await?;

                trace!(target: "tcp", "Send data: {:?}", DebuggableBytes(payload));
            }

            // Read
            *place = ProbeTcpErrorPlace::Read;
            tcp.read_to_end(&mut data).await?;
            Ok::<(), io::Error>(())
        })
        .await
        {
            return Err(error);
        }

        *place = ProbeTcpErrorPlace::Shutdown;
        tcp.shutdown().await?;

        // Log and Return
        trace!(target: "tcp", "Received data: {:?}", DebuggableBytes(&data));
        Ok(data)
    }

    /// Send `payload` and receive answer over TLS
    pub async fn probe_tls(
        &self,
        payload: &[u8],
        alpn: Option<&str>,
    ) -> Result<Result<Option<Vec<u8>>, native_tls::Error>, ProbeTcpError> {
        let mut place = ProbeTcpErrorPlace::Connect;
        match self.raw_probe_tls(payload, alpn, &mut place).await {
            Ok(Ok(data)) => Ok(Ok(Some(data))),
            Ok(Err(tls_error)) => Ok(Err(tls_error)),
            Err(io_error) => match io_error.kind() {
                io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted => Ok(Ok(None)),
                io::ErrorKind::NotConnected if matches!(place, ProbeTcpErrorPlace::Shutdown) => {
                    Ok(Ok(None))
                }
                io::ErrorKind::TimedOut if matches!(place, ProbeTcpErrorPlace::Connect) => {
                    Ok(Ok(None))
                }
                _ => Err(ProbeTcpError {
                    source: io_error,
                    place,
                }),
            },
        }
    }

    /// 1. Connect to the socket using tls over tcp
    /// 2. Send `payload`
    /// 3. Wait for the configured amount of time
    /// 4. Return everything which has been received
    async fn raw_probe_tls(
        &self,
        payload: &[u8],
        alpn: Option<&str>,
        place: &mut ProbeTcpErrorPlace,
    ) -> Result<Result<Vec<u8>, native_tls::Error>, io::Error> {
        // Configure TLS
        let alpns = alpn.as_ref().map(std::slice::from_ref).unwrap_or(&[]);
        let connector = tokio_native_tls::TlsConnector::from(
            native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .use_sni(false)
                .request_alpns(alpns)
                .build()
                .map_err(io::Error::other)?,
        );

        // Connect
        *place = ProbeTcpErrorPlace::Connect;
        let tls_result = timeout(self.connect_timeout, async {
            let tcp = TcpStream::connect(self.socket).await?;
            Ok::<_, io::Error>(connector.connect("<ignored>", tcp).await)
        })
        .await??;
        let mut tls = match tls_result {
            Ok(tls) => tls,
            Err(err) => return Ok(Err(err)),
        };

        let mut data = Vec::new();
        if let Ok(Err(error)) = timeout(self.recv_timeout, async {
            // Send payload
            if !payload.is_empty() {
                *place = ProbeTcpErrorPlace::Write;
                tls.write_all(payload).await?;

                *place = ProbeTcpErrorPlace::Flush;
                tls.flush().await?;
            }

            // Read
            *place = ProbeTcpErrorPlace::Read;
            if let Err(err) = tls.read_to_end(&mut data).await {
                debug!(target: "tls", "TLS read failed: {err}");
            }
            Ok::<(), io::Error>(())
        })
        .await
        {
            return Err(error);
        }

        // Close
        *place = ProbeTcpErrorPlace::Shutdown;
        if let Err(err) = tls.shutdown().await {
            debug!(target: "tls", "TLS shutdown failed: {err}");
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
