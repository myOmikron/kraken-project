//! In this module is the definition of the gRPC services

use std::future::Future;
use std::num::NonZeroU32;
use std::num::NonZeroUsize;
use std::ops::RangeInclusive;
use std::pin::Pin;
use std::time::Duration;

use chrono::Datelike;
use chrono::Timelike;
use futures::stream::BoxStream;
use futures::Stream;
use ipnetwork::IpNetwork;
use itertools::Itertools;
use kraken_proto::any_attack_response;
use kraken_proto::req_attack_service_server::ReqAttackService;
use kraken_proto::shared;
use kraken_proto::shared::dns_record::Record;
use kraken_proto::shared::dns_txt_scan::Info;
use kraken_proto::shared::spf_directive;
use kraken_proto::shared::spf_part;
use kraken_proto::shared::Aaaa;
use kraken_proto::shared::Address;
use kraken_proto::shared::CertEntry;
use kraken_proto::shared::DnsRecord;
use kraken_proto::shared::DnsTxtKnownService;
use kraken_proto::shared::DnsTxtKnownServiceList;
use kraken_proto::shared::DnsTxtScan;
use kraken_proto::shared::DnsTxtServiceHint;
use kraken_proto::shared::GenericRecord;
use kraken_proto::shared::Net;
use kraken_proto::shared::OperatingSystem;
use kraken_proto::shared::SpfDirective;
use kraken_proto::shared::SpfExplanationModifier;
use kraken_proto::shared::SpfInfo;
use kraken_proto::shared::SpfMechanismA;
use kraken_proto::shared::SpfMechanismAll;
use kraken_proto::shared::SpfMechanismExists;
use kraken_proto::shared::SpfMechanismInclude;
use kraken_proto::shared::SpfMechanismIp;
use kraken_proto::shared::SpfMechanismMx;
use kraken_proto::shared::SpfMechanismPtr;
use kraken_proto::shared::SpfPart;
use kraken_proto::shared::SpfQualifier;
use kraken_proto::shared::SpfRedirectModifier;
use kraken_proto::shared::SpfUnknownModifier;
use kraken_proto::shared::A;
use kraken_proto::BruteforceSubdomainRequest;
use kraken_proto::BruteforceSubdomainResponse;
use kraken_proto::CertificateTransparencyRequest;
use kraken_proto::CertificateTransparencyResponse;
use kraken_proto::DnsResolutionRequest;
use kraken_proto::DnsResolutionResponse;
use kraken_proto::DnsTxtScanRequest;
use kraken_proto::DnsTxtScanResponse;
use kraken_proto::HostsAliveRequest;
use kraken_proto::HostsAliveResponse;
use kraken_proto::OsDetectionRequest;
use kraken_proto::OsDetectionResponse;
use kraken_proto::ServiceCertainty;
use kraken_proto::ServiceDetectionRequest;
use kraken_proto::ServiceDetectionResponse;
use kraken_proto::UdpServiceDetectionRequest;
use kraken_proto::UdpServiceDetectionResponse;
use log::error;
use prost_types::Timestamp;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::Response;
use tonic::Status;
use uuid::Uuid;

use crate::backlog::Backlog;
use crate::modules::bruteforce_subdomains::bruteforce_subdomains;
use crate::modules::bruteforce_subdomains::BruteforceSubdomainResult;
use crate::modules::bruteforce_subdomains::BruteforceSubdomainsSettings;
use crate::modules::certificate_transparency::query_ct_api;
use crate::modules::certificate_transparency::CertificateTransparencySettings;
use crate::modules::dns::dns_resolution;
use crate::modules::dns::spf::SPFMechanism;
use crate::modules::dns::spf::SPFPart;
use crate::modules::dns::spf::SPFQualifier;
use crate::modules::dns::txt::start_dns_txt_scan;
use crate::modules::dns::txt::DnsTxtScanSettings;
use crate::modules::dns::txt::TxtScanInfo;
use crate::modules::dns::txt::TxtServiceHint;
use crate::modules::dns::DnsRecordResult;
use crate::modules::dns::DnsResolutionSettings;
use crate::modules::host_alive::icmp_scan::start_icmp_scan;
use crate::modules::host_alive::icmp_scan::IcmpScanSettings;
use crate::modules::os_detection::os_detection;
use crate::modules::os_detection::OperatingSystemInfo;
use crate::modules::os_detection::OsDetectionSettings;
use crate::modules::service_detection::tcp::start_tcp_service_detection;
use crate::modules::service_detection::tcp::TcpServiceDetectionResult;
use crate::modules::service_detection::tcp::TcpServiceDetectionSettings;
use crate::modules::service_detection::udp::start_udp_service_detection;
use crate::modules::service_detection::udp::UdpServiceDetectionSettings;
use crate::modules::service_detection::ProtocolSet;
use crate::modules::service_detection::Service;
use crate::utils::IteratorExt;

/// The Attack service
pub struct Attacks {
    pub(crate) backlog: Backlog,
}

#[tonic::async_trait]
impl ReqAttackService for Attacks {
    type BruteforceSubdomainsStream =
        Pin<Box<dyn Stream<Item = Result<BruteforceSubdomainResponse, Status>> + Send>>;

    async fn bruteforce_subdomains(
        &self,
        request: Request<BruteforceSubdomainRequest>,
    ) -> Result<Response<Self::BruteforceSubdomainsStream>, Status> {
        let req = request.into_inner();

        let attack_uuid = Uuid::parse_str(&req.attack_uuid)
            .map_err(|_| Status::invalid_argument("attack_uuid has to be an Uuid"))?;

        let settings = BruteforceSubdomainsSettings {
            domain: req.domain,
            wordlist_path: req.wordlist_path.parse().unwrap(),
            concurrent_limit: req.concurrent_limit,
        };

        self.stream_attack(
            attack_uuid,
            {
                |tx| async move {
                    bruteforce_subdomains(settings, tx)
                        .await
                        .map_err(|err| Status::unknown(err.to_string()))
                }
            },
            |value| BruteforceSubdomainResponse {
                record: Some(match value {
                    BruteforceSubdomainResult::A { source, target } => DnsRecord {
                        record: Some(Record::A(A {
                            source,
                            to: Some(shared::Ipv4::from(target)),
                        })),
                    },
                    BruteforceSubdomainResult::Aaaa { source, target } => DnsRecord {
                        record: Some(Record::Aaaa(Aaaa {
                            source,
                            to: Some(shared::Ipv6::from(target)),
                        })),
                    },
                    BruteforceSubdomainResult::Cname { source, target } => DnsRecord {
                        record: Some(Record::Cname(GenericRecord { source, to: target })),
                    },
                }),
            },
            any_attack_response::Response::BruteforceSubdomain,
        )
    }

    async fn query_certificate_transparency(
        &self,
        request: Request<CertificateTransparencyRequest>,
    ) -> Result<Response<CertificateTransparencyResponse>, Status> {
        let req = request.into_inner();

        let settings = CertificateTransparencySettings {
            target: req.target,
            include_expired: req.include_expired,
            max_retries: req.max_retries,
            retry_interval: Duration::from_millis(req.retry_interval),
        };

        let ct_res = CertificateTransparencyResponse {
            entries: query_ct_api(settings)
                .await
                .map_err(|err| Status::unknown(err.to_string()))?
                .into_iter()
                .map(|cert_entry| CertEntry {
                    issuer_name: cert_entry.issuer_name,
                    common_name: cert_entry.common_name,
                    value_names: cert_entry.name_value,
                    not_before: cert_entry.not_before.map(|nb| {
                        Timestamp::date_time_nanos(
                            nb.year() as i64,
                            nb.month() as u8,
                            nb.day() as u8,
                            nb.hour() as u8,
                            nb.minute() as u8,
                            nb.second() as u8,
                            nb.nanosecond(),
                        )
                        .unwrap()
                    }),
                    not_after: cert_entry.not_after.map(|na| {
                        Timestamp::date_time_nanos(
                            na.year() as i64,
                            na.month() as u8,
                            na.day() as u8,
                            na.hour() as u8,
                            na.minute() as u8,
                            na.second() as u8,
                            na.nanosecond(),
                        )
                        .unwrap()
                    }),
                    serial_number: cert_entry.serial_number,
                })
                .collect(),
        };

        Ok(Response::new(ct_res))
    }

    type ServiceDetectionStream =
        Pin<Box<dyn Stream<Item = Result<ServiceDetectionResponse, Status>> + Send>>;

    async fn service_detection(
        &self,
        request: Request<ServiceDetectionRequest>,
    ) -> Result<Response<Self::ServiceDetectionStream>, Status> {
        let request = request.into_inner();

        let attack_uuid = Uuid::parse_str(&request.attack_uuid)
            .map_err(|_| Status::invalid_argument("attack_uuid has to be an Uuid"))?;

        let mut ports = request
            .ports
            .into_iter()
            .map(RangeInclusive::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        if ports.is_empty() {
            ports.push(1..=u16::MAX);
        }

        let concurrent_limit = NonZeroU32::new(request.concurrent_limit)
            .ok_or_else(|| Status::invalid_argument("concurrent_limit can't be zero"))?;

        let settings = TcpServiceDetectionSettings {
            addresses: request
                .targets
                .into_iter()
                .map(IpNetwork::try_from)
                .collect::<Result<_, _>>()?,
            ports,
            connect_timeout: Duration::from_millis(request.connect_timeout),
            receive_timeout: Duration::from_millis(request.receive_timeout),
            max_retries: request.max_retries,
            retry_interval: Duration::from_millis(request.retry_interval),
            concurrent_limit,
            skip_icmp_check: request.skip_icmp_check,
            just_scan: false,
        };

        self.stream_attack(
            attack_uuid,
            {
                |tx| async move {
                    start_tcp_service_detection(settings, tx)
                        .await
                        .map_err(|err| {
                            error!("Service detection failed: {err:?}");
                            Status::internal("Service detection failed. See logs")
                        })
                }
            },
            |TcpServiceDetectionResult { service, addr }| match service {
                Service::Unknown => ServiceDetectionResponse {
                    response_type: ServiceCertainty::Unknown as _,
                    services: Vec::new(),
                    address: Some(shared::Address::from(addr.ip())),
                    port: addr.port() as u32,
                },
                Service::Maybe(services) => ServiceDetectionResponse {
                    response_type: ServiceCertainty::Maybe as _,
                    services: services.into_iter().map(new_rpc_service).collect(),
                    address: Some(shared::Address::from(addr.ip())),
                    port: addr.port() as u32,
                },
                Service::Definitely { service, protocols } => ServiceDetectionResponse {
                    response_type: ServiceCertainty::Definitely as _,
                    services: vec![new_rpc_service((service, protocols))],
                    address: Some(shared::Address::from(addr.ip())),
                    port: addr.port() as u32,
                },
            },
            any_attack_response::Response::ServiceDetection,
        )
    }

    type UdpServiceDetectionStream =
        Pin<Box<dyn Stream<Item = Result<UdpServiceDetectionResponse, Status>> + Send>>;

    async fn udp_service_detection(
        &self,
        request: Request<UdpServiceDetectionRequest>,
    ) -> Result<Response<Self::UdpServiceDetectionStream>, Status> {
        let request = request.into_inner();

        let attack_uuid = Uuid::parse_str(&request.attack_uuid)
            .map_err(|_| Status::invalid_argument("attack_uuid has to be an Uuid"))?;

        let mut ports = request
            .ports
            .into_iter()
            .map(RangeInclusive::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        if ports.is_empty() {
            ports.push(1..=u16::MAX);
        }

        let settings = UdpServiceDetectionSettings {
            addresses: request
                .targets
                .into_iter()
                .map(IpNetwork::try_from)
                .collect::<Result<_, _>>()?,
            ports,
            concurrent_limit: request.concurrent_limit,
            max_retries: request.max_retries,
            retry_interval: Duration::from_millis(request.retry_interval),
            timeout: Duration::from_millis(request.timeout),
        };

        self.stream_attack(
            attack_uuid,
            {
                |tx| async move {
                    start_udp_service_detection(&settings, tx)
                        .await
                        .map_err(|err| Status::unknown(err.to_string()))
                }
            },
            move |value| UdpServiceDetectionResponse {
                address: Some(shared::Address::from(value.address)),
                port: value.port as u32,
                certainty: match value.service {
                    Service::Unknown => ServiceCertainty::Unknown as _,
                    Service::Maybe(_) => ServiceCertainty::Maybe as _,
                    Service::Definitely { .. } => ServiceCertainty::Definitely as _,
                },
                services: match value.service {
                    Service::Unknown => Vec::new(),
                    Service::Maybe(services) => services.into_iter().map(new_rpc_service).collect(),
                    Service::Definitely { service, protocols } => {
                        vec![new_rpc_service((service, protocols))]
                    }
                },
            },
            any_attack_response::Response::UdpServiceDetection,
        )
    }

    type HostsAliveCheckStream =
        Pin<Box<dyn Stream<Item = Result<HostsAliveResponse, Status>> + Send>>;

    async fn hosts_alive_check(
        &self,
        request: Request<HostsAliveRequest>,
    ) -> Result<Response<Self::HostsAliveCheckStream>, Status> {
        let req = request.into_inner();

        if req.targets.is_empty() {
            return Err(Status::invalid_argument("no hosts to check"));
        }

        let attack_uuid = Uuid::parse_str(&req.attack_uuid)
            .map_err(|_| Status::invalid_argument("attack_uuid has to be an Uuid"))?;

        let settings = IcmpScanSettings {
            concurrent_limit: req.concurrent_limit,
            timeout: Duration::from_millis(req.timeout),
            addresses: req
                .targets
                .into_iter()
                .map(IpNetwork::try_from)
                .collect::<Result<_, _>>()?,
        };

        self.stream_attack(
            attack_uuid,
            |tx| async move {
                start_icmp_scan(settings, tx)
                    .await
                    .map_err(|err| Status::unknown(err.to_string()))
            },
            |value| HostsAliveResponse {
                host: Some(Address::from(value)),
            },
            any_attack_response::Response::HostsAlive,
        )
    }

    type DnsResolutionStream =
        Pin<Box<dyn Stream<Item = Result<DnsResolutionResponse, Status>> + Send>>;

    async fn dns_resolution(
        &self,
        request: Request<DnsResolutionRequest>,
    ) -> Result<Response<Self::DnsResolutionStream>, Status> {
        let req = request.into_inner();

        if req.targets.is_empty() {
            return Err(Status::invalid_argument("nothing to resolve"));
        }

        let attack_uuid = Uuid::parse_str(&req.attack_uuid)
            .map_err(|_| Status::invalid_argument("attack_uuid has to be an Uuid"))?;

        let settings = DnsResolutionSettings {
            domains: req.targets,
            // TODO: concurrent limit currently has no effect
        };

        self.stream_attack(
            attack_uuid,
            |tx| async move {
                dns_resolution(settings, tx)
                    .await
                    .map_err(|err| Status::unknown(err.to_string()))
            },
            |value| DnsResolutionResponse {
                record: Some(match value {
                    DnsRecordResult::A { source, target } => DnsRecord {
                        record: Some(Record::A(A {
                            source,
                            to: Some(shared::Ipv4::from(target)),
                        })),
                    },
                    DnsRecordResult::Aaaa { source, target } => DnsRecord {
                        record: Some(Record::Aaaa(Aaaa {
                            source,
                            to: Some(shared::Ipv6::from(target)),
                        })),
                    },
                    DnsRecordResult::CAA { source, target } => DnsRecord {
                        record: Some(Record::Caa(GenericRecord { source, to: target })),
                    },
                    DnsRecordResult::Cname { source, target } => DnsRecord {
                        record: Some(Record::Cname(GenericRecord { source, to: target })),
                    },
                    DnsRecordResult::Mx { source, target } => DnsRecord {
                        record: Some(Record::Mx(GenericRecord { source, to: target })),
                    },
                    DnsRecordResult::Tlsa { source, target } => DnsRecord {
                        record: Some(Record::Tlsa(GenericRecord { source, to: target })),
                    },
                    DnsRecordResult::Txt { source, target } => DnsRecord {
                        record: Some(Record::Txt(GenericRecord { source, to: target })),
                    },
                }),
            },
            any_attack_response::Response::DnsResolution,
        )
    }

    type DnsTxtScanStream = Pin<Box<dyn Stream<Item = Result<DnsTxtScanResponse, Status>> + Send>>;

    async fn dns_txt_scan(
        &self,
        request: Request<DnsTxtScanRequest>,
    ) -> Result<Response<Self::DnsTxtScanStream>, Status> {
        let req = request.into_inner();

        if req.targets.is_empty() {
            return Err(Status::invalid_argument("nothing to resolve"));
        }

        let attack_uuid = Uuid::parse_str(&req.attack_uuid)
            .map_err(|_| Status::invalid_argument("attack_uuid has to be an Uuid"))?;

        let settings = DnsTxtScanSettings {
            domains: req.targets,
        };

        self.stream_attack(
            attack_uuid,
            |tx| async move {
                start_dns_txt_scan(settings, tx)
                    .await
                    .map_err(|err| Status::unknown(err.to_string()))
            },
            |value| DnsTxtScanResponse {
                record: Some(DnsTxtScan {
                    domain: value.domain,
                    info: Some(match value.info {
                        TxtScanInfo::SPF { parts } => Info::Spf(SpfInfo {
                            parts: parts
                                .iter()
                                .map(|part| SpfPart {
                                    rule: part.encode_spf(),
                                    part: Some(match part {
                                        SPFPart::Directive {
                                            qualifier,
                                            mechanism,
                                        } => spf_part::Part::Directive(SpfDirective {
                                            mechanism: Some(match mechanism {
                                                SPFMechanism::All => spf_directive::Mechanism::All(
                                                    SpfMechanismAll {},
                                                ),
                                                SPFMechanism::Include { domain } => {
                                                    spf_directive::Mechanism::Include(
                                                        SpfMechanismInclude {
                                                            domain: domain.clone(),
                                                        },
                                                    )
                                                }
                                                SPFMechanism::A {
                                                    domain,
                                                    ipv4_cidr,
                                                    ipv6_cidr,
                                                } => spf_directive::Mechanism::A(SpfMechanismA {
                                                    domain: domain.clone(),
                                                    ipv4_cidr: ipv4_cidr.map(|a| a as _),
                                                    ipv6_cidr: ipv6_cidr.map(|a| a as _),
                                                }),
                                                SPFMechanism::MX {
                                                    domain,
                                                    ipv4_cidr,
                                                    ipv6_cidr,
                                                } => spf_directive::Mechanism::Mx(SpfMechanismMx {
                                                    domain: domain.clone(),
                                                    ipv4_cidr: ipv4_cidr.map(|a| a as _),
                                                    ipv6_cidr: ipv6_cidr.map(|a| a as _),
                                                }),
                                                SPFMechanism::PTR { domain } => {
                                                    spf_directive::Mechanism::Ptr(SpfMechanismPtr {
                                                        domain: domain.clone(),
                                                    })
                                                }
                                                SPFMechanism::IP { ipnet } => {
                                                    spf_directive::Mechanism::Ip(SpfMechanismIp {
                                                        ip: Some(Net::from(*ipnet)),
                                                    })
                                                }
                                                SPFMechanism::Exists { domain } => {
                                                    spf_directive::Mechanism::Exists(
                                                        SpfMechanismExists {
                                                            domain: domain.clone(),
                                                        },
                                                    )
                                                }
                                            }),
                                            qualifier: match qualifier {
                                                SPFQualifier::Pass => SpfQualifier::Pass as _,
                                                SPFQualifier::Fail => SpfQualifier::Fail as _,
                                                SPFQualifier::SoftFail => {
                                                    SpfQualifier::SoftFail as _
                                                }
                                                SPFQualifier::Neutral => SpfQualifier::Neutral as _,
                                            },
                                        }),
                                        SPFPart::RedirectModifier { domain } => {
                                            spf_part::Part::Redirect(SpfRedirectModifier {
                                                domain: domain.clone(),
                                            })
                                        }
                                        SPFPart::ExplanationModifier { domain } => {
                                            spf_part::Part::Explanation(SpfExplanationModifier {
                                                domain: domain.clone(),
                                            })
                                        }
                                        SPFPart::UnknownModifier { name, value } => {
                                            spf_part::Part::UnknownModifier(SpfUnknownModifier {
                                                name: name.clone(),
                                                value: value.clone(),
                                            })
                                        }
                                    }),
                                })
                                .collect(),
                        }),
                        TxtScanInfo::ServiceHints { hints } => {
                            Info::WellKnown(DnsTxtKnownServiceList {
                                hints: hints
                                    .into_iter()
                                    .map(|hint| DnsTxtKnownService {
                                        rule: hint.0,
                                        service: match hint.1 {
                                            TxtServiceHint::HasGoogleAccount => {
                                                DnsTxtServiceHint::HasGoogleAccount as _
                                            }
                                            TxtServiceHint::HasDocusignAccount => {
                                                DnsTxtServiceHint::HasDocusignAccount as _
                                            }
                                            TxtServiceHint::HasAppleAccount => {
                                                DnsTxtServiceHint::HasAppleAccount as _
                                            }
                                            TxtServiceHint::HasFacebookAccount => {
                                                DnsTxtServiceHint::HasFacebookAccount as _
                                            }
                                            TxtServiceHint::HasHubspotAccount => {
                                                DnsTxtServiceHint::HasHubspotAccount as _
                                            }
                                            TxtServiceHint::HasMsDynamics365 => {
                                                DnsTxtServiceHint::HasMsDynamics365 as _
                                            }
                                            TxtServiceHint::HasStripeAccount => {
                                                DnsTxtServiceHint::HasStripeAccount as _
                                            }
                                            TxtServiceHint::HasOneTrustSso => {
                                                DnsTxtServiceHint::HasOneTrustSso as _
                                            }
                                            TxtServiceHint::HasBrevoAccount => {
                                                DnsTxtServiceHint::HasBrevoAccount as _
                                            }
                                            TxtServiceHint::HasGlobalsignAccount => {
                                                DnsTxtServiceHint::HasGlobalsignAccount as _
                                            }
                                            TxtServiceHint::HasGlobalsignSMime => {
                                                DnsTxtServiceHint::HasGlobalsignSMime as _
                                            }
                                            TxtServiceHint::OwnsAtlassianAccounts => {
                                                DnsTxtServiceHint::OwnsAtlassianAccounts as _
                                            }
                                            TxtServiceHint::OwnsZoomAccounts => {
                                                DnsTxtServiceHint::OwnsZoomAccounts as _
                                            }
                                            TxtServiceHint::EmailProtonMail => {
                                                DnsTxtServiceHint::EmailProtonMail as _
                                            }
                                        },
                                    })
                                    .collect(),
                            })
                        }
                    }),
                }),
            },
            any_attack_response::Response::DnsTxtScan,
        )
    }

    type OsDetectionStream =
        Pin<Box<dyn Stream<Item = Result<OsDetectionResponse, Status>> + Send>>;

    async fn os_detection(
        &self,
        request: Request<OsDetectionRequest>,
    ) -> Result<Response<Self::OsDetectionStream>, Status> {
        let req = request.into_inner();

        if req.targets.is_empty() {
            return Err(Status::invalid_argument("no targets specified"));
        }

        let addresses: Vec<_> = req
            .targets
            .into_iter()
            .map(IpNetwork::try_from)
            .collect::<Result<_, _>>()?;

        let attack_uuid = Uuid::parse_str(&req.attack_uuid)
            .map_err(|_| Status::invalid_argument("attack_uuid has to be an Uuid"))?;

        let fingerprint_port = match req.fingerprint_port {
            None => None,
            Some(p) => Some(
                u16::try_from(p)
                    .map_err(|_| Status::invalid_argument("`fingerprint_port` out of range"))?,
            ),
        };

        let ssh_port = match req.ssh_port {
            None => None,
            Some(p) => Some(
                u16::try_from(p)
                    .map_err(|_| Status::invalid_argument("`ssh_port` out of range"))?,
            ),
        };

        let concurrent_limit = NonZeroUsize::new(req.concurrent_limit as usize);

        self.stream_attack(
            attack_uuid,
            |tx| async move {
                addresses
                    .iter()
                    .cloned()
                    .flat_map(|network| network.into_iter())
                    .try_for_each_concurrent(concurrent_limit, |address| async move {
                        let os = os_detection(OsDetectionSettings {
                            ip_addr: address,
                            fingerprint_port,
                            fingerprint_timeout: Duration::from_millis(req.fingerprint_timeout),
                            ssh_port,
                            ssh_connect_timeout: Duration::from_millis(req.ssh_connect_timeout),
                            ssh_timeout: Duration::from_millis(req.ssh_timeout),
                            port_ack_timeout: Duration::from_millis(req.port_ack_timeout),
                            port_parallel_syns: req.port_parallel_syns as usize,
                        })
                        .await
                        .map_err(|err| {
                            error!("OS detection failed: {err:?}");
                            Status::internal("OS detection failed. See logs")
                        })?;

                        let address = Address::from(address);

                        tx.send(match os {
                            OperatingSystemInfo::Unknown { hint } => OsDetectionResponse {
                                host: Some(address),
                                os: OperatingSystem::Unknown as _,
                                hints: hint.iter().cloned().collect(),
                                versions: Vec::new(),
                            },
                            OperatingSystemInfo::Linux {
                                distro,
                                kernel_version,
                                hint,
                            } => OsDetectionResponse {
                                host: Some(address),
                                os: OperatingSystem::Linux as _,
                                hints: if kernel_version.is_empty() {
                                    hint.iter().cloned().collect()
                                } else {
                                    hint.iter()
                                        .cloned()
                                        .chain(vec![format!(
                                            "Kernel {}",
                                            kernel_version.iter().join(" OR ")
                                        )])
                                        .collect()
                                },
                                versions: distro
                                    .iter()
                                    .map(|(distro, v)| match v {
                                        None => format!("{distro:?}"),
                                        Some(v) => format!("{distro:?} {v}"),
                                    })
                                    .collect(),
                            },
                            OperatingSystemInfo::BSD { version, hint } => OsDetectionResponse {
                                host: Some(address),
                                os: OperatingSystem::Bsd as _,
                                hints: hint.iter().cloned().collect(),
                                versions: version.iter().cloned().collect(),
                            },
                            OperatingSystemInfo::Android { version, hint } => OsDetectionResponse {
                                host: Some(address),
                                os: OperatingSystem::Android as _,
                                hints: hint.iter().cloned().collect(),
                                versions: version.iter().cloned().collect(),
                            },
                            OperatingSystemInfo::OSX { version, hint } => OsDetectionResponse {
                                host: Some(address),
                                os: OperatingSystem::Osx as _,
                                hints: hint.iter().cloned().collect(),
                                versions: version.iter().cloned().collect(),
                            },
                            OperatingSystemInfo::IOS { version, hint } => OsDetectionResponse {
                                host: Some(address),
                                os: OperatingSystem::Ios as _,
                                hints: hint.iter().cloned().collect(),
                                versions: version.iter().cloned().collect(),
                            },
                            OperatingSystemInfo::Windows { version, hint } => OsDetectionResponse {
                                host: Some(address),
                                os: OperatingSystem::Windows as _,
                                hints: hint.iter().cloned().collect(),
                                versions: version
                                    .iter()
                                    .map(|(ver, v)| match v {
                                        None => format!("{ver}"),
                                        Some(v) => format!("{ver} {v}"),
                                    })
                                    .collect(),
                            },
                        })
                        .await
                        .map_err(|_| Status::internal("failed to send"))
                    })
                    .await
            },
            |value| value,
            any_attack_response::Response::OsDetection,
        )
    }
}

impl Attacks {
    /// Perform an attack which streams its results
    ///
    /// It manages the communication between the attacking task, the grpc output stream and the backlog.
    ///
    /// The `perform_attack` argument is an async closure (called once) which performs the actual attack.
    /// It receives a [`mpsc::Sender<Item>`] to stream its results
    /// and is expected to produce a [`Result<(), Status>`](Status).
    fn stream_attack<Item, GrpcItem, AttackFut>(
        &self,
        attack_uuid: Uuid,
        perform_attack: impl FnOnce(mpsc::Sender<Item>) -> AttackFut,
        convert_result: impl Fn(Item) -> GrpcItem + Send + 'static,
        backlog_wrapper: impl Fn(GrpcItem) -> any_attack_response::Response + Send + 'static,
    ) -> Result<Response<BoxStream<'static, Result<GrpcItem, Status>>>, Status>
    where
        Item: Send + 'static,
        GrpcItem: Send + 'static,
        AttackFut: Future<Output = Result<(), Status>> + Send + 'static,
        AttackFut::Output: Send + 'static,
    {
        let (from_attack, mut to_middleware) = mpsc::channel::<Item>(16);
        let (from_middleware, to_stream) = mpsc::channel::<Result<GrpcItem, Status>>(1);

        // Spawn attack
        let attack = perform_attack(from_attack);
        let error_from_attack = from_middleware.clone();
        tokio::spawn(async move {
            if let Err(err) = attack.await {
                let _ = error_from_attack.send(Err(err)).await;
            }
        });

        let backlog = self.backlog.clone();

        // Spawn middleware
        tokio::spawn({
            async move {
                while let Some(item) = to_middleware.recv().await {
                    let grpc_item: GrpcItem = convert_result(item);

                    // Try sending the item over the rpc stream
                    let result = from_middleware.send(Ok(grpc_item)).await;

                    // Failure means the receiver i.e. outgoing stream has been closed and dropped
                    if let Err(error) = result {
                        let Ok(grpc_item) = error.0 else {
                            unreachable!("We tried to send an `Ok(_)` above");
                        };

                        // Save this item to the backlog
                        backlog.store(attack_uuid, backlog_wrapper(grpc_item)).await;

                        // Drain all remaining items into the backlog, because the stream is gone
                        while let Some(item) = to_middleware.recv().await {
                            let grpc_item: GrpcItem = convert_result(item);
                            backlog.store(attack_uuid, backlog_wrapper(grpc_item)).await;
                        }
                        return;
                    }
                }
            }
        });

        // Return stream
        Ok(Response::new(Box::pin(ReceiverStream::new(to_stream))))
    }
}

fn new_rpc_service((service, protocols): (&'static str, ProtocolSet)) -> kraken_proto::Service {
    let ProtocolSet { tcp, tls, udp } = protocols;
    kraken_proto::Service {
        name: service.to_string(),
        tcp,
        tls,
        udp,
    }
}
