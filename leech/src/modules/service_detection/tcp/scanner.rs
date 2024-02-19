use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use log::{debug, trace, warn};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::error::Elapsed;
use tokio::time::{sleep, timeout};

/// Tries up to `max_retries + 1` times to connect to `socket` using tcp.
pub async fn is_port_open(
    socket: SocketAddr,
    connect_timeout: Duration,
    max_retries: u32,
    retry_interval: Duration,
) -> bool {
    trace!("{socket} is being tested...");

    for i in 0..=max_retries {
        match tcp_connect(socket, connect_timeout).await {
            Ok(mut stream) => {
                if let Err(err) = stream.shutdown().await {
                    debug!("{socket} couldn't shut down tcp stream: {err}");
                }

                trace!("{socket} is open");
                return true;
            }
            Err(TcpConnectError::Timeout(_)) => {
                trace!("{socket} reached timeout (#{i})");
            }
            Err(TcpConnectError::Refused(err)) => {
                trace!("{socket} refused connection (#{i}): {err:?}");
            }
            Err(TcpConnectError::NoRoute(err)) => {
                trace!("{socket} has no route to host (#{i}): {err:?}");
            }
            Err(TcpConnectError::Other(err)) => {
                warn!("{socket} had an unknown error: {err}");
                trace!("{socket} had an unknown error: {err}");
            }
        }

        sleep(retry_interval).await;
    }

    trace!("{socket} is closed");
    false
}

pub async fn tcp_connect(
    socket: SocketAddr,
    connect_timeout: Duration,
) -> Result<TcpStream, TcpConnectError> {
    timeout(connect_timeout, TcpStream::connect(socket))
        .await?
        .map_err(|err| {
            let err_str = err.to_string();
            if err_str.contains("refused") {
                TcpConnectError::Refused(err)
            } else if err_str.contains("No route to host") {
                TcpConnectError::NoRoute(err)
            } else {
                TcpConnectError::Other(err)
            }
        })
}

/// Error returned by [`tcp_connect`]
#[derive(Debug, Error)]
pub enum TcpConnectError {
    #[error("Timeout was reached")]
    Timeout(
        #[from]
        #[source]
        Elapsed,
    ),
    #[error("Connection refused")]
    Refused(#[source] io::Error),
    #[error("No route to host")]
    NoRoute(#[source] io::Error),
    #[error("{0}")]
    Other(#[source] io::Error),
}
