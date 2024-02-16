use std::collections::BTreeMap;
use std::ops::ControlFlow;

use log::{debug, trace, warn};
use probe_config::generated::{BaseProbe, Match};

use crate::modules::service_detection::tcp::TcpServiceDetectionSettings;
use crate::modules::service_detection::{generated, DynError, DynResult};
use crate::utils::DebuggableBytes;

#[derive(Default, Copy, Clone)]
pub struct Protocols {
    pub tcp: bool,
    pub tls: bool,
}

/// The reason why [`find_exact_match`] might have exited early
pub enum BreakReason {
    /// One probe had an exact match
    Found(&'static str, Protocols),

    /// An unrecoverable error occurred
    Error(DynError),
}

/// Trys every probe ordered by prevalence until one matches.
///
/// If an exact match is found, this function will exit early with [`BreakReason::Found`].
///
/// Otherwise, the function will continue through all probes and return a set of partial matches.
pub async fn find_exact_match(
    settings: &TcpServiceDetectionSettings,
) -> ControlFlow<BreakReason, BTreeMap<&'static str, Protocols>> {
    let mut partial_matches: BTreeMap<&'static str, Protocols> = BTreeMap::new();

    debug!("Retrieving tcp banner");
    let result = settings.probe_tcp(b"").await;
    let tcp_banner = convert_result(result)?;

    debug!("Retrieving tls banner");
    let result = settings.probe_tls(b"", None).await;
    let tls_banner = convert_result(result)?
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
                if let Some(data) = convert_result(result)? {
                    check_match(&mut partial_matches, &*probe, &data, Protocols::TCP)?
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
                    match convert_result(result)? {
                        Ok(data) => {
                            check_match(&mut partial_matches, &*probe, &data, Protocols::TCP)?
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
pub async fn find_all_protocols(
    settings: &TcpServiceDetectionSettings,
    service: &'static str,
    mut already_found: Protocols,
) -> DynResult<Protocols> {
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
        if let Some(banner) = settings.probe_tls(b"", None).await?.ok() {
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
            if let Some(data) = settings.probe_tls(probe.payload, probe.alpn).await?.ok() {
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
                .or_insert(Default::default())
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

fn convert_result<T>(result: Result<T, impl Into<DynError>>) -> ControlFlow<BreakReason, T> {
    match result {
        Ok(value) => ControlFlow::Continue(value),
        Err(error) => ControlFlow::Break(BreakReason::Error(error.into())),
    }
}
