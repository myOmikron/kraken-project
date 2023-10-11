//! All code for dns resolution resides in this module

pub mod errors;

use std::net::{Ipv4Addr, Ipv6Addr};

use futures::{stream, StreamExt};
use log::{debug, error, info, warn};
use tokio::sync::mpsc::Sender;
use trust_dns_resolver::error::ResolveErrorKind;
use trust_dns_resolver::proto::rr::RecordType;
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
                    match resolver.lookup_ip(&domain).await {
                        Err(err) => match err.kind() {
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
                        },
                        Ok(answer) => {
                            for record in answer.as_lookup().records() {
                                let record = record.clone();
                                match record.record_type() {
                                    RecordType::A => {
                                        let r = record.into_data().unwrap().into_a().unwrap();
                                        let res = DnsRecordResult::A {
                                            source: domain.clone(),
                                            target: *r,
                                        };
                                        if let Err(err) = tx.send(res).await {
                                            warn!("Could not send result to tx: {err}");
                                        }
                                    }
                                    RecordType::AAAA => {
                                        let r = record.into_data().unwrap().into_aaaa().unwrap();
                                        let res = DnsRecordResult::Aaaa {
                                            source: domain.clone(),
                                            target: *r,
                                        };
                                        if let Err(err) = tx.send(res).await {
                                            warn!("Could not send result to tx: {err}");
                                        }
                                    }
                                    RecordType::CAA => {
                                        let r = record.into_data().unwrap().into_caa().unwrap();
                                        let res = DnsRecordResult::CAA {
                                            source: domain.clone(),
                                            target: r.value().to_string(),
                                        };
                                        if let Err(err) = tx.send(res).await {
                                            warn!("Could not send result to tx: {err}");
                                        }
                                    }
                                    RecordType::CNAME => {
                                        let r = record.into_data().unwrap().into_cname().unwrap();
                                        let target =
                                            r.to_string().strip_suffix('.').unwrap().to_owned();
                                        let res = DnsRecordResult::Cname {
                                            source: domain.clone(),
                                            target,
                                        };
                                        if let Err(err) = tx.send(res).await {
                                            warn!("Could not send result to tx: {err}");
                                        }
                                    }
                                    RecordType::MX => {
                                        let r = record.into_data().unwrap().into_mx().unwrap();
                                        let res = DnsRecordResult::Mx {
                                            source: domain.clone(),
                                            target: r.exchange().to_string(),
                                        };
                                        if let Err(err) = tx.send(res).await {
                                            warn!("Could not send result to tx: {err}");
                                        }
                                    }
                                    RecordType::TLSA => {
                                        let r = record.into_data().unwrap().into_tlsa().unwrap();
                                        let res = DnsRecordResult::Tlsa {
                                            source: domain.clone(),
                                            target: r.to_string(),
                                        };
                                        if let Err(err) = tx.send(res).await {
                                            warn!("Could not send result to tx: {err}");
                                        }
                                    }
                                    RecordType::TXT => {
                                        let r = record.into_data().unwrap().into_txt().unwrap();
                                        let res = DnsRecordResult::Txt {
                                            source: domain.clone(),
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
                    }
                    debug!("Finished dns resolution for {}", domain);
                }
            }
        })
        .await;

    info!("Finished DNS resolution");

    Ok(())
}
