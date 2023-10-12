//! All code for dns resolution resides in this module

pub mod errors;

use std::future::Future;
use std::net::{Ipv4Addr, Ipv6Addr};

use futures::{stream, StreamExt};
use log::{debug, error, info, warn};
use tokio::sync::mpsc::Sender;
use trust_dns_resolver::error::{ResolveError, ResolveErrorKind};
use trust_dns_resolver::proto::rr::{Record, RecordType};
use trust_dns_resolver::TokioAsyncResolver;

use crate::modules::dns::errors::DnsResolutionError;

/// Result of a subdomain
#[derive(Debug)]
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
    /// Maximum of concurrent tasks that should be spawned
    ///
    /// 0 means, that there should be no limit.
    pub concurrent_limit: u32,
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

    // TODO: hard limit MAX workers
    let chunk_count =
        (settings.domains.len() as f32 / settings.concurrent_limit as f32).ceil() as usize;

    stream::iter(settings.domains)
        .chunks(chunk_count)
        .for_each_concurrent(settings.concurrent_limit as usize, move |chunk_domains| {
            let resolver = resolver.clone();
            let tx = tx.clone();
            async move {
                for domain in chunk_domains {
                    debug!("Started dns resolution for {}", domain);

                    // A, AAAA, CNAME
                    if let Ok(res) = resolve(resolver.lookup_ip(&domain)).await {
                        send_records(&domain, &tx, res.as_lookup().records()).await;
                    }

                    if let Ok(res) = resolve(resolver.mx_lookup(&domain)).await {
                        send_records(&domain, &tx, res.as_lookup().records()).await;
                    }

                    if let Ok(res) = resolve(resolver.tlsa_lookup(&domain)).await {
                        send_records(&domain, &tx, res.as_lookup().records()).await;
                    }

                    if let Ok(res) = resolve(resolver.txt_lookup(&domain)).await {
                        send_records(&domain, &tx, res.as_lookup().records()).await;
                    }

                    // TODO: noted as unstable interface
                    if let Ok(res) = resolver.lookup(&domain, RecordType::CAA).await {
                        send_records(&domain, &tx, res.records()).await;
                    }

                    debug!("Finished dns resolution for {}", domain);
                }
            }
        })
        .await;

    info!("Finished DNS resolution");

    Ok(())
}

async fn resolve<T, Func>(resolver: Func) -> Result<T, ()>
where
    Func: Future<Output = Result<T, ResolveError>>,
{
    return resolver.await.map_err(|err| match err.kind() {
        ResolveErrorKind::Message(err) => error!("Message: {err}"),
        ResolveErrorKind::Msg(err) => error!("Msg: {err}"),
        ResolveErrorKind::NoConnections => {
            error!("There are no resolvers available")
        }
        ResolveErrorKind::Io(err) => error!("IO error: {err}"),
        ResolveErrorKind::Proto(err) => error!("Proto error {err}"),
        ResolveErrorKind::Timeout => error!("Timeout while query"),
        ResolveErrorKind::NoRecordsFound { .. } => {
            debug!("Wildcard test: no wildcard")
        }
        _ => error!("Unknown error"),
    });
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
