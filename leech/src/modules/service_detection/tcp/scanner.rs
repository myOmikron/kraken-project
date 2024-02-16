use std::net::SocketAddr;
use std::time::Duration;

use log::{debug, trace, warn};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
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
        if let Ok(result) = timeout(connect_timeout, TcpStream::connect(socket)).await {
            match result {
                Ok(mut stream) => {
                    if let Err(err) = stream.shutdown().await {
                        debug!("{socket} couldn't shut down tcp stream: {err}");
                    }

                    trace!("{socket} is open");
                    return true;
                }
                Err(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("refused") {
                        trace!("{socket} refused connection: {err}");
                    } else if err_str.contains("No route to host") {
                        trace!("{socket} has no route to host: {err}");
                    } else {
                        warn!("{socket} had an unknown error: {err}");
                    }
                }
            }
        } else {
            trace!("{socket} reached timeout (#{i})");
        }

        sleep(retry_interval).await;
    }

    trace!("{socket} is closed");
    false
}
