//! This module uses a wordlist to bruteforce subdomains of a target domain.
//!
//! It requests A and AAAA records of the constructed domain of a DNS server.

use std::collections::HashSet;
use std::fs;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::panic;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use itertools::Itertools;
use log::debug;
use log::error;
use log::info;
use log::trace;
use log::warn;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use rand::thread_rng;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::sleep;
use trust_dns_resolver::config::LookupIpStrategy;
use trust_dns_resolver::config::ResolverConfig;
use trust_dns_resolver::config::ResolverOpts;
use trust_dns_resolver::error::ResolveError;
use trust_dns_resolver::error::ResolveErrorKind;
use trust_dns_resolver::proto::rr::rdata::A;
use trust_dns_resolver::proto::rr::rdata::AAAA;
use trust_dns_resolver::proto::rr::rdata::CNAME;
use trust_dns_resolver::proto::rr::RData;
use trust_dns_resolver::proto::rr::Record;
use trust_dns_resolver::proto::rr::RecordType;
use trust_dns_resolver::TokioAsyncResolver;

use crate::modules::bruteforce_subdomains::error::BruteforceSubdomainError;

pub mod error;

/// How many times [`resolve`] tries a specific domain
/// before aborting the entire operation with a [`BruteforceSubdomainError::RepeatedError`].
pub const RETRY: usize = 3;

/// The duration [`resolve`] waits between its attempts
pub const RETRY_INTERVAL: Duration = Duration::from_millis(200);

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
    tx: mpsc::Sender<BruteforceSubdomainResult>,
) -> Result<(), BruteforceSubdomainError> {
    info!("Started subdomain enumeration for {}", settings.domain);

    let mut opts = ResolverOpts::default();
    opts.ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    opts.preserve_intermediates = true;
    opts.shuffle_dns_servers = true;
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
        num_lines.div_ceil(settings.concurrent_limit as usize)
    };
    for chunk in &wordlist.lines().chunks(chunk_size) {
        let ctx = ctx.clone();
        let entries: Vec<_> = chunk.map(ToString::to_string).collect();

        tasks.spawn({
            async move {
                let (resolver, domain, tx, wildcard) = &*ctx;
                for entry in entries {
                    resolve(
                        resolver,
                        &format!("{entry}.{domain}."),
                        wildcard.as_ref(),
                        tx,
                    )
                    .await?;
                }
                Ok(())
            }
        });
    }
    debug!("Started {} tasks", tasks.len());
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

/// Tries to resolve a single domain.
///
/// This function calls [`TokioAsyncResolver::lookup_ip`] up to [`RETRY`]-times with `search`.
/// If it succeeds, it will send the results over the `tx`.
///
/// ## Errors
/// - If the channel `tx` is closed, [`BruteforceSubdomainError::ChannelClosed`] will be returned.
/// - If [`TokioAsyncResolver::lookup_ip`] fails [`RETRY`]-times, [`BruteforceSubdomainError::RepeatedError`] will be returned.
pub async fn resolve(
    resolver: &TokioAsyncResolver,
    search: &str,
    wildcards: Option<&Wildcards>,
    tx: &mpsc::Sender<BruteforceSubdomainResult>,
) -> Result<(), BruteforceSubdomainError> {
    for _ in 0..RETRY {
        match resolver.lookup_ip(search).await {
            Err(err) => {
                match err.kind() {
                    ResolveErrorKind::NoRecordsFound { .. } => {
                        trace!("No record found: {search}");
                        return Ok(());
                    }
                    ResolveErrorKind::Proto(err) => {
                        if err
                            .to_string()
                            .contains("Label contains invalid characters")
                        {
                            debug!("Skipping search: {search} as invalid characters were found");
                            return Ok(());
                        }
                    }
                    _ => {}
                }

                warn!(
                    "Failed to resolve {search:?}: {:?} {err}. Retrying in {}ms",
                    err.kind(),
                    RETRY_INTERVAL.as_millis()
                );
                sleep(RETRY_INTERVAL).await;
                continue;
            }
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
                            if wildcards.map(|wc| wc.matches(record)).unwrap_or(false) {
                                continue;
                            }

                            if let Some(RData::CNAME(target)) = record.data() {
                                let mut target = target.to_string();
                                if target.ends_with('.') {
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
                            if wildcards.map(|wc| wc.matches(record)).unwrap_or(false) {
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
                            if wildcards.map(|wc| wc.matches(record)).unwrap_or(false) {
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
                return Ok(());
            }
        }
    }
    Err(BruteforceSubdomainError::RepeatedError)
}

/// Struct storing the records which apply to any domain i.e. the wildcards
#[derive(Debug)]
pub struct Wildcards {
    domain: String,
    a: HashSet<A>,
    aaaa: HashSet<AAAA>,
    cname: HashSet<CNAME>,
}
impl Wildcards {
    /// Query a random domain and store its records
    ///
    /// Unlike [`resolve`] this method won't handle [`ResolveError`] and won't retry.
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

    /// Check whether a given `record` appeared in the wildcards
    pub fn matches(&self, record: &Record) -> bool {
        match record.data() {
            Some(RData::A(a)) => self.a.contains(a),
            Some(RData::AAAA(aaaa)) => self.aaaa.contains(aaaa),
            Some(RData::CNAME(cname)) => self.cname.contains(cname),
            _ => false,
        }
    }

    /// Iterate over the wildcard records as results
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
