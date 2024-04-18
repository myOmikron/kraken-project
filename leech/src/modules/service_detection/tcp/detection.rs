use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::time::Duration;

use log::debug;
use log::trace;
use log::warn;

use crate::modules::service_detection::generated;
use crate::modules::service_detection::generated::Match;
use crate::modules::service_detection::tcp::oneshot::ProbeTcpError;
use crate::modules::service_detection::tcp::oneshot::ProbeTcpErrorPlace;
use crate::modules::service_detection::tcp::OneShotTcpSettings;
use crate::modules::service_detection::DynError;
use crate::modules::service_detection::DynResult;
use crate::modules::service_detection::ProtocolSet;
use crate::modules::service_detection::Service;
use crate::utils::DebuggableBytes;

/// Runs service detection on a single tcp port
///
/// Returns `Ok(None)` if the port is closed.
pub async fn detect_service(
    socket: SocketAddr,
    recv_timeout: Duration,
    connect_timeout: Duration,
) -> DynResult<Option<Service>> {
    match find_exact_match(socket, recv_timeout, connect_timeout).await {
        ControlFlow::Continue(partial_matches) => Ok(Some(if partial_matches.is_empty() {
            Service::Unknown
        } else {
            Service::Maybe(partial_matches)
        })),
        ControlFlow::Break(BreakReason::Found(service, protocol)) => {
            let protocols =
                find_all_protocols(socket, recv_timeout, connect_timeout, service, protocol)
                    .await?;
            Ok(Some(Service::Definitely { service, protocols }))
        }
        ControlFlow::Break(BreakReason::TcpError(err)) => {
            if matches!(err.place, ProbeTcpErrorPlace::Connect) {
                Ok(None)
            } else {
                Err(err.into())
            }
        }
        ControlFlow::Break(BreakReason::DynError(err)) => Err(err),
    }
}

/// The reason why [`find_exact_match`] might have exited early
#[derive(Debug)]
pub enum BreakReason {
    /// One probe had an exact match
    Found(&'static str, ProtocolSet),

    /// An error occurred which might be mapped to [`Service::Failed`]
    TcpError(ProbeTcpError),

    /// An unrecoverable error occurred
    DynError(DynError),
}

/// Trys every probe ordered by prevalence until one matches.
///
/// If an exact match is found, this function will exit early with [`BreakReason::Found`].
///
/// Otherwise, the function will continue through all probes and return a set of partial matches.
async fn find_exact_match(
    socket: SocketAddr,
    recv_timeout: Duration,
    connect_timeout: Duration,
) -> ControlFlow<BreakReason, BTreeMap<&'static str, ProtocolSet>> {
    let settings = OneShotTcpSettings {
        socket,
        recv_timeout,
        connect_timeout,
    };
    let mut partial_matches: BTreeMap<&'static str, ProtocolSet> = BTreeMap::new();

    debug!("Retrieving tcp banner");
    let result = settings.probe_tcp(b"").await;
    let tcp_banner = convert_result(result)?;

    debug!("Retrieving tls banner");
    let result = settings.probe_tls(b"", None).await;
    let tls_banner = convert_result(result)?
        .inspect_err(|err| debug!(target: "tls", "TLS error: {err:?}"))
        .ok()
        .flatten();

    for prevalence in 0..3 {
        if let Some(tcp_banner) = tcp_banner.as_deref() {
            debug!("Starting tcp banner scans #{prevalence}");
            for probe in &generated::PROBES.empty_tcp_probes[prevalence] {
                trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(tcp_banner));
                check_match(
                    &mut partial_matches,
                    probe.is_match(tcp_banner),
                    probe.service,
                    ProtocolSet::TCP,
                )?;
            }

            debug!("Starting tcp payload scans #{prevalence}");
            for probe in &generated::PROBES.payload_tcp_probes[prevalence] {
                let result = settings.probe_tcp(probe.payload).await;
                if let Some(data) = convert_result(result)? {
                    trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(&data));
                    check_match(
                        &mut partial_matches,
                        probe.is_match(&data),
                        probe.service,
                        ProtocolSet::TCP,
                    )?;
                }
            }

            debug!("Starting tcp rust scans #{prevalence}");
            for probe in &generated::PROBES.rust_tcp_probes[prevalence] {
                check_match(
                    &mut partial_matches,
                    convert_result((probe.function)(&settings).await)?,
                    probe.service,
                    ProtocolSet::TCP,
                )?;
            }

            if let Some(tls_banner) = tls_banner.as_deref() {
                debug!("Starting tls banner scans #{prevalence}");
                for probe in &generated::PROBES.empty_tls_probes[prevalence] {
                    trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(tls_banner));
                    check_match(
                        &mut partial_matches,
                        probe.is_match(tls_banner),
                        probe.service,
                        ProtocolSet::TLS,
                    )?;
                }

                debug!("Starting tls rust scans #{prevalence}");
                for probe in &generated::PROBES.rust_tls_probes[prevalence] {
                    check_match(
                        &mut partial_matches,
                        convert_result((probe.function)(&settings, probe.alpn).await)?,
                        probe.service,
                        ProtocolSet::TLS,
                    )?;
                }

                debug!("Starting tls payload scans #{prevalence}");
                for probe in &generated::PROBES.payload_tls_probes[prevalence] {
                    let result = settings.probe_tls(probe.payload, probe.alpn).await;
                    match convert_result(result)? {
                        Ok(Some(data)) => {
                            trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(&data));
                            check_match(
                                &mut partial_matches,
                                probe.is_match(&data),
                                probe.service,
                                ProtocolSet::TLS,
                            )?;
                        }
                        Ok(None) => continue,
                        Err(err) => {
                            warn!(target: "tls", "Failed to connect while probing {}: {err}", probe.service)
                        }
                    }
                }
            }
        }
    }

    ControlFlow::Continue(partial_matches)
}

/// Checks all of a single service's probes to detect all its protocols
async fn find_all_protocols(
    socket: SocketAddr,
    recv_timeout: Duration,
    connect_timeout: Duration,
    service: &'static str,
    mut already_found: ProtocolSet,
) -> DynResult<ProtocolSet> {
    let settings = OneShotTcpSettings {
        socket,
        recv_timeout,
        connect_timeout,
    };
    fn iter_all<T>(probes: &[Vec<T>; 3]) -> impl Iterator<Item = &T> {
        probes.iter().flat_map(|vec| vec.iter())
    }

    debug!("Testing all remaining protocols for {service}");
    trace!("Already found: {already_found:?}");

    // Test tcp banner
    if !already_found.tcp {
        debug!("Testing {service}'s tcp banner probes");
        if let Some(banner) = settings.probe_tcp(b"").await? {
            for probe in iter_all(&generated::PROBES.empty_tcp_probes)
                .filter(|probe| probe.service == service)
            {
                if matches!(probe.is_match(&banner), Match::Exact) {
                    already_found.tcp = true;
                    break;
                }
            }
        }
    }

    // Test tcp payload
    if !already_found.tcp {
        debug!("Testing {service}'s tcp payload probes");
        for probe in
            iter_all(&generated::PROBES.payload_tcp_probes).filter(|probe| probe.service == service)
        {
            if let Some(data) = settings.probe_tcp(probe.payload).await? {
                if matches!(probe.is_match(&data), Match::Exact) {
                    already_found.tcp = true;
                    break;
                }
            }
        }
    }

    // Test tcp rust
    if !already_found.tcp {
        debug!("Testing {service}'s tcp rust probes");
        for probe in
            iter_all(&generated::PROBES.rust_tcp_probes).filter(|probe| probe.service == service)
        {
            if matches!((probe.function)(&settings).await?, Match::Exact) {
                already_found.tcp = true;
                break;
            }
        }
    }

    // Test tls banner
    if !already_found.tls {
        debug!("Testing {service}'s tls banner probes");
        if let Ok(Some(banner)) = settings.probe_tls(b"", None).await? {
            for probe in iter_all(&generated::PROBES.empty_tls_probes)
                .filter(|probe| probe.service == service)
            {
                if matches!(probe.is_match(&banner), Match::Exact) {
                    already_found.tls = true;
                    break;
                }
            }
        }
    }

    // Test tls payload
    if !already_found.tls {
        debug!("Testing {service}'s tls payload probes");
        for probe in
            iter_all(&generated::PROBES.payload_tls_probes).filter(|probe| probe.service == service)
        {
            if let Ok(Some(data)) = settings.probe_tls(probe.payload, probe.alpn).await? {
                if matches!(probe.is_match(&data), Match::Exact) {
                    already_found.tls = true;
                    break;
                }
            }
        }
    }

    // Test tls rust
    if !already_found.tls {
        debug!("Testing {service}'s tls rust probes");
        for probe in
            iter_all(&generated::PROBES.rust_tls_probes).filter(|probe| probe.service == service)
        {
            if matches!((probe.function)(&settings, probe.alpn).await?, Match::Exact) {
                already_found.tls = true;
                break;
            }
        }
    }

    Ok(already_found)
}

fn check_match(
    partial_matches: &mut BTreeMap<&'static str, ProtocolSet>,
    r#match: Match,
    service: &'static str,
    protocols: ProtocolSet,
) -> ControlFlow<BreakReason> {
    match r#match {
        Match::No => ControlFlow::Continue(()),
        Match::Partial => {
            partial_matches
                .entry(service)
                .or_default()
                .update(protocols);
            ControlFlow::Continue(())
        }
        Match::Exact => ControlFlow::Break(BreakReason::Found(service, protocols)),
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
impl From<DynError> for BreakReason {
    fn from(value: DynError) -> Self {
        BreakReason::DynError(value)
    }
}
