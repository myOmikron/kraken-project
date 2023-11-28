//! This module uses a wordlist to bruteforce subdomains of a target domain.
//!
//! It requests A and AAAA records of the constructed domain of a DNS server.

use std::collections::HashSet;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, panic};

use itertools::Itertools;
use log::{debug, error, info, trace};
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinSet;
use trust_dns_resolver::config::{LookupIpStrategy, ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::{ResolveError, ResolveErrorKind};
use trust_dns_resolver::proto::rr::rdata::{A, AAAA, CNAME};
use trust_dns_resolver::proto::rr::{RData, Record, RecordType};
use trust_dns_resolver::TokioAsyncResolver;

use crate::modules::bruteforce_subdomains::error::BruteforceSubdomainError;

pub mod error;

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
    pub concurrent_limit: u32,
}

/// Result of a subdomain
#[derive(Debug, Clone)]
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
) -> Result<(), BruteforceSubdomainError> {
    info!("Started subdomain enumeration for {}", settings.domain);

    let mut opts = ResolverOpts::default();
    opts.ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    opts.preserve_intermediates = true;
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::cloudflare_https(), opts);

    let wordlist = fs::read_to_string(&settings.wordlist_path)
        .map_err(BruteforceSubdomainError::WordlistRead)?;

    let wildcard = Wildcards::new(&resolver, &settings.domain)
        .await
        .unwrap_or_else(|err| {
            error!("Failed to query the wildcard test: {err}");
            None
        });
    if let Some(wildcard) = wildcard.as_ref() {
        for result in wildcard.as_results() {
            tx.send(result).await?;
        }
    }

    // Collection of `Sync` type which are shared by all tasks
    let ctx = Arc::new((resolver, settings.domain, tx, wildcard));

    let mut tasks = JoinSet::new();
    let num_lines = wordlist.lines().count();
    let chunk_size = if settings.concurrent_limit == 0 {
        num_lines
    } else if settings.concurrent_limit as usize > num_lines {
        1
    } else {
        num_lines / settings.concurrent_limit as usize
    };
    for chunk in &wordlist.lines().chunks(chunk_size) {
        let ctx = ctx.clone();
        let entries: Vec<_> = chunk.map(ToString::to_string).collect();

        tasks.spawn({
            async move {
                let (resolver, domain, tx, wildcard) = &*ctx;
                let wildcard = wildcard.as_ref();

                for entry in entries {
                    let search = format!("{entry}.{domain}.");
                    match resolver.lookup_ip(&search).await {
                        Ok(answer) => {
                            for record in answer.as_lookup().records() {
                                let domain = record
                                    .name()
                                    .to_string()
                                    .strip_suffix('.')
                                    .unwrap()
                                    .to_string();

                                match record.record_type() {
                                    RecordType::CNAME => {
                                        if wildcard.map(|wc| wc.matches(record)).unwrap_or(false) {
                                            continue;
                                        }

                                        if let Some(RData::CNAME(target)) = record.data() {
                                            let mut target = target.to_string();
                                            if target.ends_with(',') {
                                                target.pop();
                                            }
                                            let res = BruteforceSubdomainResult::Cname {
                                                source: domain,
                                                target,
                                            };
                                            tx.send(res).await?;
                                        }
                                    }
                                    RecordType::A => {
                                        if wildcard.map(|wc| wc.matches(record)).unwrap_or(false) {
                                            continue;
                                        }

                                        if let Some(RData::A(target)) = record.data() {
                                            let res = BruteforceSubdomainResult::A {
                                                source: domain,
                                                target: **target,
                                            };
                                            tx.send(res).await?;
                                        }
                                    }
                                    RecordType::AAAA => {
                                        if wildcard.map(|wc| wc.matches(record)).unwrap_or(false) {
                                            continue;
                                        }

                                        if let Some(RData::AAAA(target)) = record.data() {
                                            let res = BruteforceSubdomainResult::Aaaa {
                                                source: domain,
                                                target: **target,
                                            };
                                            tx.send(res).await?;
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
                Result::<(), BruteforceSubdomainError>::Ok(())
            }
        });
    }
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(error),
            Err(join_error) => panic::resume_unwind(
                join_error
                    .try_into_panic()
                    .expect("The tasks are never canceled"),
            ),
        }
    }

    info!("Finished subdomain enumeration");

    Ok(())
}

#[derive(Debug)]
pub struct Wildcards {
    domain: String,
    a: HashSet<A>,
    aaaa: HashSet<AAAA>,
    cname: HashSet<CNAME>,
}
impl Wildcards {
    pub async fn new(
        resolver: &TokioAsyncResolver,
        domain: &str,
    ) -> Result<Option<Self>, ResolveError> {
        let mut wildcards = Self {
            domain: domain.to_string(),
            a: HashSet::new(),
            aaaa: HashSet::new(),
            cname: HashSet::new(),
        };
        let test_domain = format!(
            "{random}.{domain}.",
            random = Alphanumeric.sample_string(&mut thread_rng(), 32)
        );
        debug!("Querying \"{test_domain}\" to detect a wildcard",);
        match resolver.lookup_ip(&test_domain).await {
            Ok(res) => {
                for record in res.as_lookup().records() {
                    match record.data() {
                        Some(RData::A(a)) => {
                            wildcards.a.insert(*a);
                        }
                        Some(RData::AAAA(aaaa)) => {
                            wildcards.aaaa.insert(*aaaa);
                        }
                        Some(RData::CNAME(cname)) => {
                            wildcards.cname.insert(cname.clone());
                        }
                        _ => {}
                    }
                }
                debug!(
                    "Found wildcard: A={:?} AAAA={:?} CNAME={:?}",
                    wildcards.a, wildcards.aaaa, wildcards.cname
                );
                Ok(Some(wildcards))
            }
            Err(err) => match err.kind() {
                ResolveErrorKind::NoRecordsFound { .. } => {
                    debug!("No wildcard was found");
                    Ok(None)
                }
                _ => Err(err),
            },
        }
    }

    pub fn matches(&self, record: &Record) -> bool {
        match record.data() {
            Some(RData::A(a)) => self.a.contains(a),
            Some(RData::AAAA(aaaa)) => self.aaaa.contains(aaaa),
            Some(RData::CNAME(cname)) => self.cname.contains(cname),
            _ => false,
        }
    }

    pub fn as_results(&self) -> impl Iterator<Item = BruteforceSubdomainResult> + '_ {
        let a = self.a.iter().map(|target| BruteforceSubdomainResult::A {
            source: format!("*.{}", self.domain),
            target: **target,
        });
        let aaaa = self
            .aaaa
            .iter()
            .map(|target| BruteforceSubdomainResult::Aaaa {
                source: format!("*.{}", self.domain),
                target: **target,
            });
        let cname = self
            .cname
            .iter()
            .map(|target| BruteforceSubdomainResult::Cname {
                source: format!("*.{}", self.domain),
                target: {
                    let mut target = target.to_string();
                    if target.ends_with('.') {
                        target.pop();
                    }
                    target
                },
            });
        a.chain(aaaa).chain(cname)
    }
}
