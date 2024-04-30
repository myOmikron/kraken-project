use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::time::Duration;

use log::debug;
use log::trace;
use log::warn;
use ssl_errors::NativeTlsError;

use crate::modules::service_detection::generated;
use crate::modules::service_detection::generated::Match;
use crate::modules::service_detection::tcp::oneshot::ProbeTcpError;
use crate::modules::service_detection::tcp::oneshot::ProbeTcpErrorPlace;
use crate::modules::service_detection::tcp::OneShotTcpSettings;
use crate::modules::service_detection::tcp::TcpServiceDetectionResult;
use crate::modules::service_detection::DynError;
use crate::modules::service_detection::DynResult;
use crate::modules::service_detection::Service;
use crate::utils::DebuggableBytes;

/// Runs service detection on a single tcp port
///
/// Returns `Ok(None)` if the port is closed.
pub async fn detect_service(
    socket: SocketAddr,
    recv_timeout: Duration,
    connect_timeout: Duration,
) -> DynResult<Option<TcpServiceDetectionResult>> {
    let [tcp, tls] = [
        detect_tcp_service(socket, recv_timeout, connect_timeout).await,
        detect_tls_service(socket, recv_timeout, connect_timeout).await,
    ]
    .map(|cf| match cf {
        ControlFlow::Continue(partial_matches) => Ok(Some(if partial_matches.is_empty() {
            Service::Unknown
        } else {
            Service::Maybe(partial_matches)
        })),
        ControlFlow::Break(BreakReason::Found(service)) => Ok(Some(Service::Definitely(service))),
        ControlFlow::Break(BreakReason::TcpError(err)) => {
            if matches!(err.place, ProbeTcpErrorPlace::Connect) {
                Ok(None)
            } else {
                Err(err.into())
            }
        }
        ControlFlow::Break(BreakReason::TlsError(err)) => match NativeTlsError::new(&err) {
            NativeTlsError::NotSsl => Ok(None),
            NativeTlsError::UnrecognizedName => Ok(Some(Service::Unknown)),
            _ => {
                warn!("Unhandled ssl error: {err}");
                Ok(None)
            }
        },
        ControlFlow::Break(BreakReason::DynError(err)) => Err(err),
    });
    match tcp {
        Ok(Some(tcp_service)) => Ok(Some(TcpServiceDetectionResult {
            addr: socket,
            tcp_service,
            tls_service: tls?,
        })),
        Ok(None) => Ok(None),
        Err(err) => Err(err),
    }
}

/// The reason why [`find_exact_match`] might have exited early
#[derive(Debug)]
pub enum BreakReason {
    /// One probe had an exact match
    Found(&'static str),

    /// An error occurred which might be mapped to [`Service::Failed`]
    TcpError(ProbeTcpError),

    /// An error occurred which might be mapped to [`Service::Failed`]
    TlsError(native_tls::Error),

    /// An unrecoverable error occurred
    DynError(DynError),
}

/// Trys every probe ordered by prevalence until one matches.
///
/// If an exact match is found, this function will exit early with [`BreakReason::Found`].
///
/// Otherwise, the function will continue through all probes and return a set of partial matches.
async fn detect_tcp_service(
    socket: SocketAddr,
    recv_timeout: Duration,
    connect_timeout: Duration,
) -> ControlFlow<BreakReason, BTreeSet<&'static str>> {
    let settings = OneShotTcpSettings {
        socket,
        recv_timeout,
        connect_timeout,
    };
    let mut partial_matches: BTreeSet<&'static str> = BTreeSet::new();

    debug!("Retrieving tcp banner");
    let result = settings.probe_tcp(b"").await;
    let tcp_banner = convert_result(result)?;

    for prevalence in 0..3 {
        if let Some(tcp_banner) = tcp_banner.as_deref() {
            debug!("Starting tcp banner scans #{prevalence}");
            for probe in &generated::PROBES.empty_tcp_probes[prevalence] {
                trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(tcp_banner));
                check_match(
                    &mut partial_matches,
                    probe.is_match(tcp_banner),
                    probe.service,
                )?;
            }

            debug!("Starting tcp payload scans #{prevalence}");
            for probe in &generated::PROBES.payload_tcp_probes[prevalence] {
                let result = settings.probe_tcp(probe.payload).await;
                if let Some(data) = convert_result(result)? {
                    trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(&data));
                    check_match(&mut partial_matches, probe.is_match(&data), probe.service)?;
                }
            }

            debug!("Starting tcp rust scans #{prevalence}");
            for probe in &generated::PROBES.rust_tcp_probes[prevalence] {
                check_match(
                    &mut partial_matches,
                    convert_result((probe.function)(&settings).await)?,
                    probe.service,
                )?;
            }
        }
    }

    ControlFlow::Continue(partial_matches)
}

/// Trys every probe ordered by prevalence until one matches.
///
/// If an exact match is found, this function will exit early with [`BreakReason::Found`].
///
/// Otherwise, the function will continue through all probes and return a set of partial matches.
async fn detect_tls_service(
    socket: SocketAddr,
    recv_timeout: Duration,
    connect_timeout: Duration,
) -> ControlFlow<BreakReason, BTreeSet<&'static str>> {
    let settings = OneShotTcpSettings {
        socket,
        recv_timeout,
        connect_timeout,
    };
    let mut partial_matches: BTreeSet<&'static str> = BTreeSet::new();

    debug!("Retrieving tls banner");
    let tcp_result = settings.probe_tls(b"", None).await;
    let tls_result = convert_result(tcp_result)?;
    let tls_banner = convert_result(tls_result)?;

    for prevalence in 0..3 {
        if let Some(tls_banner) = tls_banner.as_ref() {
            debug!("Starting tls banner scans #{prevalence}");
            for probe in &generated::PROBES.empty_tls_probes[prevalence] {
                trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(tls_banner));
                check_match(
                    &mut partial_matches,
                    probe.is_match(tls_banner),
                    probe.service,
                )?;
            }
        }

        debug!("Starting tls rust scans #{prevalence}");
        for probe in &generated::PROBES.rust_tls_probes[prevalence] {
            check_match(
                &mut partial_matches,
                convert_result((probe.function)(&settings, probe.alpn).await)?,
                probe.service,
            )?;
        }

        debug!("Starting tls payload scans #{prevalence}");
        for probe in &generated::PROBES.payload_tls_probes[prevalence] {
            let result = settings.probe_tls(probe.payload, probe.alpn).await;
            match convert_result(result)? {
                Ok(Some(data)) => {
                    trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(&data));
                    check_match(&mut partial_matches, probe.is_match(&data), probe.service)?;
                }
                Ok(None) => continue,
                Err(err) => {
                    warn!(target: "tls", "Failed to connect while probing {}: {err}", probe.service)
                }
            }
        }
    }

    ControlFlow::Continue(partial_matches)
}

fn check_match(
    partial_matches: &mut BTreeSet<&'static str>,
    r#match: Match,
    service: &'static str,
) -> ControlFlow<BreakReason> {
    match r#match {
        Match::No => ControlFlow::Continue(()),
        Match::Partial => {
            partial_matches.insert(service);
            ControlFlow::Continue(())
        }
        Match::Exact => ControlFlow::Break(BreakReason::Found(service)),
    }
}

fn convert_result<T, E>(result: Result<T, E>) -> ControlFlow<BreakReason, T>
where
    E: Into<BreakReason>,
{
    match result {
        Ok(value) => ControlFlow::Continue(value),
        Err(error) => ControlFlow::Break(error.into()),
    }
}
impl From<ProbeTcpError> for BreakReason {
    fn from(value: ProbeTcpError) -> Self {
        BreakReason::TcpError(value)
    }
}
impl From<native_tls::Error> for BreakReason {
    fn from(value: native_tls::Error) -> Self {
        BreakReason::TlsError(value)
    }
}
impl From<DynError> for BreakReason {
    fn from(value: DynError) -> Self {
        BreakReason::DynError(value)
    }
}
