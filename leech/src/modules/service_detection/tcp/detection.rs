use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::time::Duration;

use log::{debug, trace, warn};
use probe_config::generated::{BaseProbe, Match};

use crate::modules::service_detection::tcp::oneshot::{ProbeTcpError, ProbeTcpErrorPlace};
use crate::modules::service_detection::tcp::OneShotTcpSettings;
use crate::modules::service_detection::{generated, DynError, DynResult, Service};
use crate::utils::DebuggableBytes;

#[derive(Default, Copy, Clone, Debug)]
pub struct Protocols {
    pub tcp: bool,
    pub tls: bool,
}

/// Runs service detection on a single tcp port
///
/// Returns `Ok(None)` if the port is closed.
pub async fn detect_service(socket: SocketAddr, timeout: Duration) -> DynResult<Option<Service>> {
    match find_exact_match(socket, timeout).await {
        ControlFlow::Continue(partial_matches) => Ok(Some(if partial_matches.is_empty() {
            Service::Unknown
        } else {
            Service::Maybe(partial_matches.into_keys().collect())
        })),
        ControlFlow::Break(BreakReason::Found(service, protocol)) => {
            let protocols = find_all_protocols(socket, timeout, service, protocol).await?;
            Ok(Some(Service::Definitely(service)))
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
    Found(&'static str, Protocols),

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
    timeout: Duration,
) -> ControlFlow<BreakReason, BTreeMap<&'static str, Protocols>> {
    let settings = OneShotTcpSettings { socket, timeout };
    let mut partial_matches: BTreeMap<&'static str, Protocols> = BTreeMap::new();

    debug!("Retrieving tcp banner");
    let result = settings.probe_tcp(b"").await;
    let tcp_banner = convert_tcp(result)?;

    debug!("Retrieving tls banner");
    let result = settings.probe_tls(b"", None).await;
    let tls_banner = convert_tls(result)?
        .inspect_err(|err| debug!(target: "tls", "TLS error: {err:?}"))
        .ok();

    for prevalence in 0..3 {
        if let Some(tcp_banner) = tcp_banner.as_deref() {
            debug!("Starting tcp banner scans #{prevalence}");
            for probe in &generated::PROBES.empty_tcp_probes[prevalence] {
                check_match(&mut partial_matches, probe, tcp_banner, Protocols::TCP)?;
            }

            debug!("Starting tcp payload scans #{prevalence}");
            for probe in &generated::PROBES.payload_tcp_probes[prevalence] {
                let result = settings.probe_tcp(probe.payload).await;
                if let Some(data) = convert_tcp(result)? {
                    check_match(&mut partial_matches, probe, &data, Protocols::TCP)?
                }
            }

            if let Some(tls_banner) = tls_banner.as_deref() {
                debug!("Starting tls banner scans #{prevalence}");
                for probe in &generated::PROBES.empty_tls_probes[prevalence] {
                    check_match(&mut partial_matches, probe, tls_banner, Protocols::TLS)?;
                }

                debug!("Starting tls payload scans #{prevalence}");
                for probe in &generated::PROBES.payload_tls_probes[prevalence] {
                    let result = settings.probe_tls(probe.payload, probe.alpn).await;
                    match convert_tls(result)? {
                        Ok(data) => {
                            check_match(&mut partial_matches, probe, &data, Protocols::TCP)?
                        }
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
    timeout: Duration,
    service: &'static str,
    mut already_found: Protocols,
) -> DynResult<Protocols> {
    let settings = OneShotTcpSettings { socket, timeout };
    fn iter_all<T>(probes: &[Vec<T>; 3]) -> impl Iterator<Item = &T> {
        probes.iter().flat_map(|vec| vec.iter())
    }

    // Test tcp banner
    if !already_found.tcp {
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

    // Test tls banner
    if !already_found.tls {
        if let Ok(banner) = settings.probe_tls(b"", None).await? {
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
        for probe in
            iter_all(&generated::PROBES.payload_tls_probes).filter(|probe| probe.service == service)
        {
            if let Ok(data) = settings.probe_tls(probe.payload, probe.alpn).await? {
                if matches!(probe.is_match(&data), Match::Exact) {
                    already_found.tls = true;
                    break;
                }
            }
        }
    }

    Ok(already_found)
}

fn check_match(
    partial_matches: &mut BTreeMap<&'static str, Protocols>,
    probe: &BaseProbe,
    haystack: &[u8],
    protocols: Protocols,
) -> ControlFlow<BreakReason> {
    trace!(target: probe.service, "Got haystack: {:?}", DebuggableBytes(haystack));
    match probe.is_match(haystack) {
        Match::No => ControlFlow::Continue(()),
        Match::Partial => {
            partial_matches
                .entry(probe.service)
                .or_default()
                .update(protocols);
            ControlFlow::Continue(())
        }
        Match::Exact => ControlFlow::Break(BreakReason::Found(probe.service, protocols)),
    }
}

impl Protocols {
    const TCP: Self = Self {
        tcp: true,
        tls: false,
    };
    const TLS: Self = Self {
        tcp: false,
        tls: true,
    };
    fn update(&mut self, other: Self) {
        self.tcp |= other.tcp;
        self.tls |= other.tls;
    }
}

fn convert_tcp<T>(result: Result<T, ProbeTcpError>) -> ControlFlow<BreakReason, T> {
    match result {
        Ok(value) => ControlFlow::Continue(value),
        Err(error) => ControlFlow::Break(BreakReason::TcpError(error)),
    }
}
fn convert_tls<T>(result: Result<T, DynError>) -> ControlFlow<BreakReason, T> {
    match result {
        Ok(value) => ControlFlow::Continue(value),
        Err(error) => ControlFlow::Break(BreakReason::DynError(error)),
    }
}
