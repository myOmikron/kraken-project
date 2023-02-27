//! This module uses a wordlist to bruteforce subdomains of a target domain.
//!
//! It requests A and AAAA records of the constructed domain of a DNS server.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use futures::{stream, StreamExt};
use log::{debug, error};
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use trust_dns_resolver::config::{LookupIpStrategy, ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::ResolveErrorKind;
use trust_dns_resolver::proto::rr::RecordType;
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
}

/// Enumerates subdomains by brute forcing dns records with a wordlist.
///
/// **Parameter**:
/// - `settings`: [BruteforceSubdomainsSettings]
pub async fn bruteforce_subdomains(settings: BruteforceSubdomainsSettings) -> Result<(), String> {
    let mut opts = ResolverOpts::default();
    opts.timeout = Duration::from_secs(30);
    opts.ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    opts.preserve_intermediates = true;
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::cloudflare_https(), opts).unwrap();

    let wordlist = fs::read_to_string(&settings.wordlist_path).map_err(|e| e.to_string())?;

    let mut wildcard_v4 = None;
    let mut wildcard_v6 = None;
    let mut wildcard_cname = None;

    let r = Alphanumeric.sample_string(&mut thread_rng(), 32);
    println!("{}", settings.domain);
    let search = format!("{r}.{}.", settings.domain);
    match resolver.lookup_ip(&search).await {
        Ok(res) => {
            for record in res.as_lookup().records() {
                let target;
                let record = record.clone();
                match record.record_type() {
                    RecordType::CNAME => {
                        let r = record.into_data().unwrap().into_cname().unwrap();
                        wildcard_cname = Some(r.clone());
                        target = r.to_string().strip_suffix('.').unwrap().to_owned();
                    }
                    RecordType::A => {
                        let r = record.into_data().unwrap().into_a().unwrap();
                        wildcard_v4 = Some(r);
                        target = r.to_string();
                    }
                    RecordType::AAAA => {
                        let r = record.into_data().unwrap().into_aaaa().unwrap();
                        wildcard_v6 = Some(r);
                        target = r.to_string();
                    }
                    _ => {
                        error!("Got unexpected record type");
                        unreachable!("got unexpected record type");
                    }
                };

                println!("Found record for *.{}: {}", &settings.domain, target);
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

    let concurrent_limit = 30;
    stream::iter(wordlist.lines())
        .chunks((wordlist.len() as f32 / concurrent_limit as f32).ceil() as usize)
        .for_each_concurrent(concurrent_limit, move |chunk| {
            let c = chunk.clone();
            let resolver = resolver.clone();
            let domain = settings.domain.clone();
            let wildcard_cname = wildcard_cname.clone();

            async move {
                for subdomain in c {
                    let search = format!("{subdomain}.{}.", &domain);
                    match resolver.lookup_ip(search).await {
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
                                        println!("Found CNAME record for {}: {}", &domain, &target);
                                    }
                                    RecordType::A => {
                                        if let Some(wildcard) = wildcard_v4 {
                                            if wildcard.to_string() == target {
                                                continue;
                                            }
                                        }

                                        println!("Found A record for {}: {}", &domain, &target);
                                    }
                                    RecordType::AAAA => {
                                        if let Some(wildcard) = wildcard_v6 {
                                            if wildcard.to_string() == target {
                                                continue;
                                            }
                                        }

                                        println!("Found AAAA record for {}: {}", &domain, &target);
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
                                debug!("Wildcard test: no wildcard")
                            }
                            _ => error!("Unknown error"),
                        },
                    }
                }
            }
        })
        .await;

    println!("Finished subdomain enumeration");

    Ok(())
}
