use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::time::Duration;

use log::debug;
use log::trace;
use log::warn;
use ssl_errors::NativeTlsError;
use tokio::time::sleep;

use crate::modules::service_detection::generated;
use crate::modules::service_detection::generated::Match;
use crate::modules::service_detection::tcp::OneShotTcpSettings;
use crate::modules::service_detection::tcp::ProbeTcpResult;
use crate::modules::service_detection::tcp::ProbeTlsResult;
use crate::modules::service_detection::tcp::TcpServiceDetectionResult;
use crate::modules::service_detection::Service;
use crate::utils::DebuggableBytes;

/// Runs service detection on a single tcp port
///
/// Returns `Ok(None)` if the port is closed.
pub async fn detect_service(
    addr: SocketAddr,
    recv_timeout: Duration,
    connect_timeout: Duration,
) -> TcpServiceDetectionResult {
    let mut tcp_failed = false;

    let tcp_service = match detect_tcp_service(addr, recv_timeout, connect_timeout).await {
        ControlFlow::Continue(matches) if matches.is_empty() => Service::Unknown,
        ControlFlow::Continue(matches) => Service::Maybe(matches),
        ControlFlow::Break(BreakReason::Found(service)) => Service::Definitely(service),
        ControlFlow::Break(BreakReason::Failed) => {
            tcp_failed = true;
            Service::Unknown
        }
        ControlFlow::Break(BreakReason::DeniedBySNI) => unreachable!(),
    };

    let tls_service = if !tcp_failed {
        match detect_tls_service(addr, recv_timeout, connect_timeout).await {
            ControlFlow::Continue(matches) if matches.is_empty() => Some(Service::Unknown),
            ControlFlow::Continue(matches) => Some(Service::Maybe(matches)),
            ControlFlow::Break(BreakReason::Found(service)) => Some(Service::Definitely(service)),
            ControlFlow::Break(BreakReason::Failed) => None,
            ControlFlow::Break(BreakReason::DeniedBySNI) => Some(Service::Unknown),
        }
    } else {
        None
    };

    TcpServiceDetectionResult {
        addr,
        tcp_service,
        tls_service,
    }
}

/// The reason why [`detect_tcp_service`] or [`detect_tls_service`] might have exited early
#[derive(Debug)]
pub enum BreakReason {
    /// One probe had an exact match
    Found(&'static str),

    /// The detection had to be aborted.
    ///
    /// This might have several reasons (logged at appropriate levels)
    /// but the result is the same: no statement about the service is possible.
    ///
    /// The (hopefully up-to-date) reasons are:
    /// - The port can't establish a TCP connection anymore
    /// - The port doesn't run TLS
    /// - TLS handshake had an unhandled error
    Failed,

    /// The port aborts the TLS handshake due to an unrecognized name
    ///
    /// This is a slightly different case than `Failed` because
    /// one statement can be made about the service: it's running TLS.
    DeniedBySNI,
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
    let tcp_banner = convert_tcp_result(&socket, settings.probe_tcp(b"").await)?;

    dirty_timeout().await;

    for prevalence in 0..3 {
        if !tcp_banner.is_empty() {
            debug!("Starting tcp banner scans #{prevalence}");
            for probe in &generated::PROBES.empty_tcp_probes[prevalence] {
                trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(&tcp_banner));
                check_match(
                    &mut partial_matches,
                    probe.is_match(&tcp_banner),
                    probe.service,
                )?;
            }
        }

        debug!("Starting tcp payload scans #{prevalence}");
        for probe in &generated::PROBES.payload_tcp_probes[prevalence] {
            let response = convert_tcp_result(&socket, settings.probe_tcp(probe.payload).await)?;
            if !response.is_empty() {
                trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(&response));
                check_match(
                    &mut partial_matches,
                    probe.is_match(&response),
                    probe.service,
                )?;
            }

            dirty_timeout().await;
        }

        debug!("Starting tcp rust scans #{prevalence}");
        for probe in &generated::PROBES.rust_tcp_probes[prevalence] {
            check_match(
                &mut partial_matches,
                (probe.function)(&settings).await,
                probe.service,
            )?;

            dirty_timeout().await;
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
    let tls_banner = convert_tls_result(&socket, settings.probe_tls(b"", None).await)?;

    dirty_timeout().await;

    for prevalence in 0..3 {
        if !tls_banner.is_empty() {
            debug!("Starting tls banner scans #{prevalence}");
            for probe in &generated::PROBES.empty_tls_probes[prevalence] {
                trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(&tls_banner));
                check_match(
                    &mut partial_matches,
                    probe.is_match(&tls_banner),
                    probe.service,
                )?;
            }
        }

        debug!("Starting tls rust scans #{prevalence}");
        for probe in &generated::PROBES.rust_tls_probes[prevalence] {
            check_match(
                &mut partial_matches,
                (probe.function)(&settings, probe.alpn).await,
                probe.service,
            )?;

            dirty_timeout().await;
        }

        debug!("Starting tls payload scans #{prevalence}");
        for probe in &generated::PROBES.payload_tls_probes[prevalence] {
            let response =
                convert_tls_result(&socket, settings.probe_tls(probe.payload, probe.alpn).await)?;
            if !response.is_empty() {
                trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(&response));
                check_match(
                    &mut partial_matches,
                    probe.is_match(&response),
                    probe.service,
                )?;
            }

            dirty_timeout().await;
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

fn convert_tcp_result(
    addr: &SocketAddr,
    result: ProbeTcpResult,
) -> ControlFlow<BreakReason, Vec<u8>> {
    match result {
        ProbeTcpResult::Ok(data) => ControlFlow::Continue(data),
        ProbeTcpResult::ErrOther(data, error) => {
            warn!("Continuing although port {addr} errored: {error}");
            ControlFlow::Continue(data)
        }
        ProbeTcpResult::ErrConnect(error) => {
            warn!("Aborting because port {addr} couldn't connect anymore: {error}");
            ControlFlow::Break(BreakReason::Failed)
        }
    }
}
fn convert_tls_result(
    addr: &SocketAddr,
    result: ProbeTlsResult,
) -> ControlFlow<BreakReason, Vec<u8>> {
    match result {
        ProbeTlsResult::Ok(data) => ControlFlow::Continue(data),
        ProbeTlsResult::ErrTls(error) => match NativeTlsError::new(&error) {
            NativeTlsError::NotSsl => {
                debug!("Aborting because port {addr} isn't running ssl");
                ControlFlow::Break(BreakReason::Failed)
            }
            NativeTlsError::UnrecognizedName => ControlFlow::Break(BreakReason::DeniedBySNI),
            _ => {
                warn!("Aborting because port {addr} has unhandled ssl error: {error}");
                ControlFlow::Break(BreakReason::Failed)
            }
        },
        ProbeTlsResult::ErrOther(data, error) => {
            warn!("Continuing although port {addr} errored: {error}");
            ControlFlow::Continue(data)
        }
        ProbeTlsResult::ErrConnect(error) => {
            warn!("Aborting because port {addr} couldn't connect anymore: {error}");
            ControlFlow::Break(BreakReason::Failed)
        }
    }
}

/// Dirty "hot-fix" against anti port scanning
///
/// TODO: replace this with proper solution
async fn dirty_timeout() {
    sleep(Duration::from_secs(1)).await
}
