//! All code for dns resolution resides in this module

pub mod errors;
pub mod spf;
pub mod txt;

use std::future::Future;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use futures::stream;
use futures::StreamExt;
use hickory_resolver::error::ResolveError;
use hickory_resolver::error::ResolveErrorKind;
use hickory_resolver::proto::rr::Record;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::TokioAsyncResolver;
use kraken_proto::any_attack_response;
use kraken_proto::push_attack_request;
use kraken_proto::shared;
use kraken_proto::shared::Aaaa;
use kraken_proto::shared::DnsRecord;
use kraken_proto::shared::GenericRecord;
use kraken_proto::shared::A;
use kraken_proto::DnsResolutionRequest;
use kraken_proto::DnsResolutionResponse;
use kraken_proto::RepeatedDnsResolutionResponse;
use log::debug;
use log::error;
use log::info;
use log::warn;
use serde::Serialize;
use tokio::sync::mpsc::Sender;
use tonic::Status;

use crate::modules::dns::errors::DnsResolutionError;
use crate::modules::StreamedAttack;

/// Attack resolving DNS
pub struct DnsResolution;
#[tonic::async_trait]
impl StreamedAttack for DnsResolution {
    type Settings = DnsResolutionSettings;
    type Output = DnsRecordResult;
    type Error = DnsResolutionError;
    async fn execute(
        settings: Self::Settings,
        sender: Sender<Self::Output>,
    ) -> Result<(), Self::Error> {
        dns_resolution(settings, sender).await
    }

    type Request = DnsResolutionRequest;
    fn get_attack_uuid(request: &Self::Request) -> &str {
        &request.attack_uuid
    }
    fn decode_settings(request: Self::Request) -> Result<Self::Settings, Status> {
        if request.targets.is_empty() {
            return Err(Status::invalid_argument("nothing to resolve"));
        }
        Ok(DnsResolutionSettings {
            domains: request.targets,
            concurrent_limit: request.concurrent_limit,
        })
    }

    type Response = DnsResolutionResponse;
    fn encode_output(output: Self::Output) -> Self::Response {
        DnsResolutionResponse {
            record: Some(match output {
                DnsRecordResult::A { source, target } => DnsRecord {
                    record: Some(shared::dns_record::Record::A(A {
                        source,
                        to: Some(shared::Ipv4::from(target)),
                    })),
                },
                DnsRecordResult::Aaaa { source, target } => DnsRecord {
                    record: Some(shared::dns_record::Record::Aaaa(Aaaa {
                        source,
                        to: Some(shared::Ipv6::from(target)),
                    })),
                },
                DnsRecordResult::CAA { source, target } => DnsRecord {
                    record: Some(shared::dns_record::Record::Caa(GenericRecord {
                        source,
                        to: target,
                    })),
                },
                DnsRecordResult::Cname { source, target } => DnsRecord {
                    record: Some(shared::dns_record::Record::Cname(GenericRecord {
                        source,
                        to: target,
                    })),
                },
                DnsRecordResult::Mx { source, target } => DnsRecord {
                    record: Some(shared::dns_record::Record::Mx(GenericRecord {
                        source,
                        to: target,
                    })),
                },
                DnsRecordResult::Tlsa { source, target } => DnsRecord {
                    record: Some(shared::dns_record::Record::Tlsa(GenericRecord {
                        source,
                        to: target,
                    })),
                },
                DnsRecordResult::Txt { source, target } => DnsRecord {
                    record: Some(shared::dns_record::Record::Txt(GenericRecord {
                        source,
                        to: target,
                    })),
                },
            }),
        }
    }

    fn print_output(output: &Self::Output) {
        match output {
            DnsRecordResult::A { source, target } => {
                info!("Found a record for {source}: {target}");
            }
            DnsRecordResult::Aaaa { source, target } => {
                info!("Found aaaa record for {source}: {target}");
            }
            DnsRecordResult::Cname { source, target } => {
                info!("Found cname record for {source}: {target}");
            }
            DnsRecordResult::CAA { source, target } => {
                info!("Found caa record for {source}: {target}");
            }
            DnsRecordResult::Mx { source, target } => {
                info!("Found mx record for {source}: {target}");
            }
            DnsRecordResult::Tlsa { source, target } => {
                info!("Found tlsa record for {source}: {target}");
            }
            DnsRecordResult::Txt { source, target } => {
                info!("Found txt record for {source}: {target}");
            }
        };
    }

    fn wrap_for_backlog(response: Self::Response) -> any_attack_response::Response {
        any_attack_response::Response::DnsResolution(response)
    }

    fn wrap_for_push(responses: Vec<Self::Response>) -> push_attack_request::Response {
        push_attack_request::Response::DnsResolution(RepeatedDnsResolutionResponse { responses })
    }
}

/// Result of a subdomain
#[derive(Debug, Serialize, Clone)]
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
#[derive(Debug)]
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
    let chunk_count = if settings.concurrent_limit == 0 {
        settings.domains.len()
    } else {
        (settings.domains.len() as f32 / settings.concurrent_limit as f32).ceil() as usize
    };

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
