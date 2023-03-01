//! This module uses a wordlist to bruteforce subdomains of a target domain.
//!
//! It requests A and AAAA records of the constructed domain of a DNS server.

use std::fs;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;

use futures::{stream, StreamExt};
use log::{debug, error, info, trace, warn};
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use tokio::sync::mpsc::Sender;
use trust_dns_resolver::config::{LookupIpStrategy, ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::ResolveErrorKind;
use trust_dns_resolver::proto::rr::{RData, RecordType};
use trust_dns_resolver::TokioAsyncResolver;

/// The settings to configure a subdomain bruteforce
#[derive(Debug)]
pub struct BruteforceSubdomainsSettings {
    /// The domain to use as base name. It shouldn't end in a . like DNS names.
    pub domain: String,
    /// Path to a wordlist that can be used for subdomain enumeration.
    ///
    /// The entries in the wordlist are assumed to be line seperated.
    pub wordlist_path: PathBuf,
    /// Maximum of concurrent tasks that should be spawned
    ///
    /// 0 means, that there should be no limit.
    pub concurrent_limit: usize,
}

/// Result of a subdomain
#[derive(Debug)]
pub enum BruteforceSubdomainResult {
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
    /// CNAME record
    Cname {
        /// Source domain
        source: String,
        /// Target domain
        target: String,
    },
}

/// Enumerates subdomains by brute forcing dns records with a wordlist.
///
/// **Parameter**:
/// - `settings`: [BruteforceSubdomainsSettings]
/// - `tx`: [Sender] of [BruteforceSubdomainResult]
pub async fn bruteforce_subdomains(
    settings: BruteforceSubdomainsSettings,
    tx: Sender<BruteforceSubdomainResult>,
) -> Result<(), String> {
    info!("Started subdomain enumeration for {}", settings.domain);

    let mut opts = ResolverOpts::default();
    opts.ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    opts.preserve_intermediates = true;
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::cloudflare_https(), opts).unwrap();

    let wordlist = fs::read_to_string(&settings.wordlist_path).map_err(|e| e.to_string())?;

    let mut wildcard_v4 = None;
    let mut wildcard_v6 = None;
    let mut wildcard_cname = None;

    let r = Alphanumeric.sample_string(&mut thread_rng(), 32);
    let search = format!("{r}.{}.", settings.domain);
    match resolver.lookup_ip(&search).await {
        Ok(res) => {
            for record in res.as_lookup().records() {
                let record = record.clone();
                match record.record_type() {
                    RecordType::CNAME => {
                        let r = record.into_data().unwrap().into_cname().unwrap();
                        wildcard_cname = Some(r.clone());
                        let target = r.to_string().strip_suffix('.').unwrap().to_owned();
                        let res = BruteforceSubdomainResult::Cname {
                            source: search.clone(),
                            target,
                        };
                        if let Err(err) = tx.send(res).await {
                            warn!("Could not send result to tx: {err}");
                        }
                    }
                    RecordType::A => {
                        let r = record.into_data().unwrap().into_a().unwrap();
                        wildcard_v4 = Some(r);
                        let res = BruteforceSubdomainResult::A {
                            source: search.clone(),
                            target: r,
                        };
                        if let Err(err) = tx.send(res).await {
                            warn!("Could not send result to tx: {err}");
                        }
                    }
                    RecordType::AAAA => {
                        let r = record.into_data().unwrap().into_aaaa().unwrap();
                        wildcard_v6 = Some(r);
                        let res = BruteforceSubdomainResult::Aaaa {
                            source: search.clone(),
                            target: r,
                        };
                        if let Err(err) = tx.send(res).await {
                            warn!("Could not send result to tx: {err}");
                        }
                    }
                    _ => {
                        error!("Got unexpected record type");
                    }
                };
            }
        }
        Err(err) => match err.kind() {
            ResolveErrorKind::Message(err) => error!("Message: {err}"),
            ResolveErrorKind::Msg(err) => error!("Msg: {err}"),
            ResolveErrorKind::NoConnections => error!("There are no resolvers available"),
            ResolveErrorKind::Io(err) => error!("IO error: {err}"),
            ResolveErrorKind::Proto(err) => error!("Proto error {err}"),
            ResolveErrorKind::Timeout => error!("Timeout while query"),
            ResolveErrorKind::NoRecordsFound { .. } => debug!("Wildcard test: no wildcard"),
            _ => error!("Unknown error"),
        },
    }

    stream::iter(wordlist.lines())
        .chunks((wordlist.len() as f32 / settings.concurrent_limit as f32).ceil() as usize)
        .for_each_concurrent(settings.concurrent_limit, move |chunk| {
            let c = chunk;
            let resolver = resolver.clone();
            let domain = settings.domain.clone();
            let wildcard_cname = wildcard_cname.clone();
            let tx = tx.clone();

            async move {
                for entry in c {
                    let search = format!("{entry}.{}.", &domain);
                    match resolver.lookup_ip(&search).await {
                        Ok(answer) => {
                            for record in answer.as_lookup().records() {
                                let domain = record
                                    .name()
                                    .to_string()
                                    .strip_suffix('.')
                                    .unwrap()
                                    .to_string();
                                let target = record.data().unwrap().to_string();
                                match record.record_type() {
                                    RecordType::CNAME => {
                                        if let Some(wildcard) = &wildcard_cname {
                                            if wildcard.to_string() == target {
                                                continue;
                                            }
                                        }

                                        let target = target.strip_suffix('.').unwrap().to_string();
                                        let res = BruteforceSubdomainResult::Cname {
                                            source: domain,
                                            target,
                                        };
                                        if let Err(err) = tx.send(res).await {
                                            warn!("Could not send result to tx: {err}");
                                        }
                                    }
                                    RecordType::A => {
                                        if let Some(wildcard) = wildcard_v4 {
                                            if wildcard.to_string() == target {
                                                continue;
                                            }
                                        }
                                        if let Some(RData::A(target)) = record.data() {
                                            let res = BruteforceSubdomainResult::A {
                                                source: domain,
                                                target: *target,
                                            };
                                            if let Err(err) = tx.send(res).await {
                                                warn!("Could not send result to tx: {err}");
                                            }
                                        }
                                    }
                                    RecordType::AAAA => {
                                        if let Some(wildcard) = wildcard_v6 {
                                            if wildcard.to_string() == target {
                                                continue;
                                            }
                                        }
                                        if let Some(RData::AAAA(target)) = record.data() {
                                            let res = BruteforceSubdomainResult::Aaaa {
                                                source: domain,
                                                target: *target,
                                            };
                                            if let Err(err) = tx.send(res).await {
                                                warn!("Could not send result to tx: {err}");
                                            }
                                        }
                                    }
                                    _ => {
                                        error!("Got unexpected record type")
                                    }
                                }
                            }
                        }
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
                                trace!("No record found: {search}");
                            }
                            _ => error!("Unknown error"),
                        },
                    }
                }
            }
        })
        .await;

    info!("Finished subdomain enumeration");

    Ok(())
}
