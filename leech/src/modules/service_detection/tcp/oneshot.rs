use std::io;
use std::net::SocketAddr;
use std::time::Duration;

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

/// Custom `Result` returned by [`OneShotTcpSettings::probe_tcp`]
pub enum ProbeTcpResult {
    /// No errors
    Ok(Vec<u8>),
    /// Error during TCP handshake
    ErrConnect(io::Error),
    /// Error happened after the handshake
    ErrOther(Vec<u8>, ProbeIoError),
}

/// Custom `Result` returned by [`OneShotTcpSettings::probe_tls`]
pub enum ProbeTlsResult {
    /// No errors
    Ok(Vec<u8>),
    /// Error during TCP handshake
    ErrConnect(io::Error),
    /// Error during TLS handshake
    ErrTls(native_tls::Error),
    /// Error happened after the handshakes
    ErrOther(Vec<u8>, ProbeIoError),
}

impl OneShotTcpSettings {
    /// Send `payload` and receive answer over TCP
    pub async fn probe_tcp(&self, payload: &[u8]) -> ProbeTcpResult {
        trace!("Sending data: {:?}", DebuggableBytes(payload));

        let mut place = ProbeErrorPlace::Connect;
        let mut data = Vec::new();

        let raw_result = async {
            place = ProbeErrorPlace::Connect;
            // TCP Handshake
            let tcp = timeout(self.connect_timeout, TcpStream::connect(self.socket)).await??;

            self.process_stream(payload, &mut place, &mut data, tcp)
                .await
        }
        .await;

        let result = match (raw_result, place) {
            (Ok(()), _) => ProbeTcpResult::Ok(data),
            (Err(error), ProbeErrorPlace::Connect) => ProbeTcpResult::ErrConnect(error),
            (Err(source), _) => ProbeTcpResult::ErrOther(data, ProbeIoError { source, place }),
        };

        if let ProbeTcpResult::Ok(data) | ProbeTcpResult::ErrOther(data, _) = &result {
            trace!("Received data: {:?}", DebuggableBytes(data));
        }

        result
    }

    /// Send `payload` and receive answer over TLS
    pub async fn probe_tls(&self, payload: &[u8], alpn: Option<&str>) -> ProbeTlsResult {
        trace!("Sending data: {:?}", DebuggableBytes(payload));

        let mut place = ProbeErrorPlace::Connect;
        let mut data = Vec::new();

        let raw_result = async {
            // Configure TLS
            let connector = tokio_native_tls::TlsConnector::from(
                native_tls::TlsConnector::builder()
                    .danger_accept_invalid_certs(true)
                    .danger_accept_invalid_hostnames(true)
                    .use_sni(false)
                    .request_alpns(alpn.as_slice())
                    .build()
                    .map_err(io::Error::other)?,
            );

            place = ProbeErrorPlace::Connect;
            let tls_result = timeout(self.connect_timeout, async {
                // TCP Handshake
                let tcp = TcpStream::connect(self.socket).await?;

                // TLS Handshake
                Ok::<_, io::Error>(connector.connect("<ignored>", tcp).await)
            })
            .await??;
            let tls = match tls_result {
                Ok(tls) => tls,
                Err(err) => return Ok(Err(err)),
            };

            self.process_stream(payload, &mut place, &mut data, tls)
                .await
                .map(Ok)
        }
        .await;

        let result = match (raw_result, place) {
            (Ok(Ok(())), _) => ProbeTlsResult::Ok(data),
            (Ok(Err(error)), _) => ProbeTlsResult::ErrTls(error),
            (Err(error), ProbeErrorPlace::Connect) => ProbeTlsResult::ErrConnect(error),
            (Err(source), _) => ProbeTlsResult::ErrOther(data, ProbeIoError { source, place }),
        };

        if let ProbeTlsResult::Ok(data) | ProbeTlsResult::ErrOther(data, _) = &result {
            trace!("Received data: {:?}", DebuggableBytes(data));
        }

        result
    }

    /// Code common to `probe_tcp` and `probe_tls` i.e. after the handshake
    async fn process_stream(
        &self,
        payload: &[u8],
        place: &mut ProbeErrorPlace,
        data: &mut Vec<u8>,
        mut stream: impl AsyncWriteExt + AsyncReadExt + Unpin,
    ) -> Result<(), io::Error> {
        let timeout_result = timeout(self.recv_timeout, async {
            if !payload.is_empty() {
                *place = ProbeErrorPlace::Write;
                stream.write_all(payload).await?;

                *place = ProbeErrorPlace::Flush;
                stream.flush().await?;
            }

            *place = ProbeErrorPlace::Read;
            stream.read_to_end(&mut *data).await?;

            Ok::<(), io::Error>(())
        })
        .await;

        match timeout_result {
            // Timeout elapsed: This is the expected case
            Err(_) => {}
            // `read_to_end` actually completed: Unexpected but also fine
            Ok(Ok(())) => {}
            // An io error occurred
            Ok(Err(error)) => {
                return Err(error);
            }
        }

        *place = ProbeErrorPlace::Shutdown;
        stream.shutdown().await?;

        Ok(())
    }
}

/// The [`io::Error`] returned by [`OneShotTcpSettings::probe_tcp`] or [`OneShotTcpSettings::probe_tls`]
#[derive(Debug, Error)]
#[error("{source} @ {place:?}")]
pub struct ProbeIoError {
    /// The error
    #[source]
    pub source: io::Error,

    /// The place the error occurred
    pub place: ProbeErrorPlace,
}

/// The places where [`OneShotTcpSettings::probe_tcp`] and [`OneShotTcpSettings::probe_tls`] might produce an [`io::Error`]
#[derive(Debug, Copy, Clone)]
pub enum ProbeErrorPlace {
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
