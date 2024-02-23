//! All code for dns resolution resides in this module

pub mod errors;
pub mod spf;
pub mod txt;

use std::future::Future;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use futures::stream;
use futures::StreamExt;
use log::debug;
use log::error;
use log::info;
use log::warn;
use tokio::sync::mpsc::Sender;
use trust_dns_resolver::error::ResolveError;
use trust_dns_resolver::error::ResolveErrorKind;
use trust_dns_resolver::proto::rr::Record;
use trust_dns_resolver::proto::rr::RecordType;
use trust_dns_resolver::TokioAsyncResolver;

use crate::modules::dns::errors::DnsResolutionError;
use crate::modules::dns::errors::LookupResultStatus;
use crate::modules::dns::errors::ResolutionStatus;

/// Result of a subdomain
#[derive(Debug, Clone)]
pub enum DnsRecordResult {
    /// A record
    A {
        /// Source domain
        source: String,
        /// Target address
        target: Ipv4Addr,
    },
    /// AAAA record
    Aaaa {
        /// Source domain
        source: String,
        /// Target address
        target: Ipv6Addr,
    },
    /// CAA record
    CAA {
        /// Source domain
        source: String,
        /// Target domain
        target: String,
    },
    /// CNAME record
    Cname {
        /// Source domain
        source: String,
        /// Target domain
        target: String,
    },
    /// Mx record
    Mx {
        /// Source domain
        source: String,
        /// Target domain
        target: String,
    },
    /// Tlsa record
    Tlsa {
        /// Source domain
        source: String,
        /// Target domain
        target: String,
    },
    /// Txt record
    Txt {
        /// Source domain
        source: String,
        /// Target domain
        target: String,
    },
}

/// DNS resolution settings
///
/// This will use `/etc/resolv.conf` on Unix OSes and the registry on Windows
pub struct DnsResolutionSettings {
    /// The domains to resolve
    pub domains: Vec<String>,
}

/// DNS resolution
///
/// **Parameter**:
/// - `settings`: [DnsResolutionSettings]
/// - `tx`: [Sender] of [DnsRecordResult]
pub async fn dns_resolution(
    settings: DnsResolutionSettings,
    tx: Sender<DnsRecordResult>,
) -> Result<(), DnsResolutionError> {
    info!("Started DNS resolution");

    let resolver = TokioAsyncResolver::tokio_from_system_conf()
        .map_err(DnsResolutionError::CreateSystemResolver)?;

    let mut failed = stream::iter(settings.domains)
        .map(move |domain| {
            let resolver = resolver.clone();
            let tx = tx.clone();
            resolve_single(domain.clone(), resolver, tx)
        })
        .buffer_unordered(8)
        .collect::<Vec<_>>()
        .await;
    failed.retain(|v| v.has_error() || !v.has_records());

    info!(
        "Finished DNS resolution with {} failed domains",
        failed.len()
    );

    if !failed.is_empty() {
        Err(DnsResolutionError::SomeFailed(failed))
    } else {
        Ok(())
    }
}

async fn resolve_single(
    domain: String,
    resolver: TokioAsyncResolver,
    tx: Sender<DnsRecordResult>,
) -> ResolutionStatus {
    debug!("Started dns resolution for {}", domain);

    macro_rules! run {
        ( $resolveFn:expr ) => {
            match resolve($resolveFn).await {
                Ok(res) => {
                    if let Some(res) = res {
                        send_records(&domain, &tx, res.as_lookup().records()).await;
                        LookupResultStatus::Success
                    } else {
                        LookupResultStatus::NoRecords
                    }
                }
                Err(err) => LookupResultStatus::Error(err),
            }
        };
    }

    macro_rules! run_manual {
        ( $resolveFn:expr ) => {
            match resolve($resolveFn).await {
                Ok(res) => {
                    if let Some(res) = res {
                        send_records(&domain, &tx, res.records()).await;
                        LookupResultStatus::Success
                    } else {
                        LookupResultStatus::NoRecords
                    }
                }
                Err(err) => LookupResultStatus::Error(err),
            }
        };
    }

    // A, AAAA, CNAME
    let res = ResolutionStatus {
        ip: run!(resolver.lookup_ip(&domain)),
        mx: run!(resolver.mx_lookup(&domain)),
        tlsa: run!(resolver.tlsa_lookup(&domain)),
        txt: run!(resolver.txt_lookup(&domain)),
        // TODO: noted as unstable interface
        caa: run_manual!(resolver.lookup(&domain, RecordType::CAA)),
        domain,
    };
    debug!("Finished dns resolution: {res}");
    res
}

async fn resolve<T, Func>(resolver: Func) -> Result<Option<T>, ResolveError>
where
    Func: Future<Output = Result<T, ResolveError>>,
{
    let r = resolver.await;
    match r {
        Ok(v) => Ok(Some(v)),
        Err(err) => {
            match err.kind() {
                ResolveErrorKind::Message(err) => error!("Message: {err}"),
                ResolveErrorKind::Msg(err) => error!("Msg: {err}"),
                ResolveErrorKind::NoConnections => {
                    error!("There are no resolvers available")
                }
                ResolveErrorKind::Io(err) => error!("IO error: {err}"),
                ResolveErrorKind::Proto(err) => error!("Proto error {err}"),
                ResolveErrorKind::Timeout => error!("Timeout while query"),
                ResolveErrorKind::NoRecordsFound { .. } => {
                    debug!("Wildcard test: no wildcard");
                    return Ok(None);
                }
                _ => error!("Unknown error"),
            }
            Err(err)
        }
    }
}

async fn send_records(domain: &str, tx: &Sender<DnsRecordResult>, records: &[Record]) {
    for record in records {
        let record = record.clone();
        match record.record_type() {
            RecordType::A => {
                let r = record.into_data().unwrap().into_a().unwrap();
                let res = DnsRecordResult::A {
                    source: domain.into(),
                    target: *r,
                };
                if let Err(err) = tx.send(res).await {
                    warn!("Could not send result to tx: {err}");
                }
            }
            RecordType::AAAA => {
                let r = record.into_data().unwrap().into_aaaa().unwrap();
                let res = DnsRecordResult::Aaaa {
                    source: domain.into(),
                    target: *r,
                };
                if let Err(err) = tx.send(res).await {
                    warn!("Could not send result to tx: {err}");
                }
            }
            RecordType::CAA => {
                let r = record.into_data().unwrap().into_caa().unwrap();
                let res = DnsRecordResult::CAA {
                    source: domain.into(),
                    target: r.to_string(),
                };
                if let Err(err) = tx.send(res).await {
                    warn!("Could not send result to tx: {err}");
                }
            }
            RecordType::CNAME => {
                let r = record.into_data().unwrap().into_cname().unwrap();
                let target = r.to_string().strip_suffix('.').unwrap().to_owned();
                let res = DnsRecordResult::Cname {
                    source: domain.into(),
                    target,
                };
                if let Err(err) = tx.send(res).await {
                    warn!("Could not send result to tx: {err}");
                }
            }
            RecordType::MX => {
                let r = record.into_data().unwrap().into_mx().unwrap();
                let target = r.to_string().strip_suffix('.').unwrap().to_owned();
                let res = DnsRecordResult::Mx {
                    source: domain.into(),
                    target,
                };
                if let Err(err) = tx.send(res).await {
                    warn!("Could not send result to tx: {err}");
                }
            }
            RecordType::TLSA => {
                let r = record.into_data().unwrap().into_tlsa().unwrap();
                let res = DnsRecordResult::Tlsa {
                    source: domain.into(),
                    target: r.to_string(),
                };
                if let Err(err) = tx.send(res).await {
                    warn!("Could not send result to tx: {err}");
                }
            }
            RecordType::TXT => {
                let r = record.into_data().unwrap().into_txt().unwrap();
                let res = DnsRecordResult::Txt {
                    source: domain.into(),
                    target: r.to_string(),
                };
                if let Err(err) = tx.send(res).await {
                    warn!("Could not send result to tx: {err}");
                }
            }
            _ => {
                error!("Got unexpected record type");
            }
        }
    }
}
