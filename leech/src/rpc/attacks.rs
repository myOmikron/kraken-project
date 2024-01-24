//! In this module is the definition of the gRPC services

use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::ops::RangeInclusive;
use std::pin::Pin;
use std::time::Duration;

use chrono::{Datelike, Timelike};
use futures::stream::BoxStream;
use futures::Stream;
use ipnetwork::IpNetwork;
use kraken_proto::req_attack_service_server::ReqAttackService;
use kraken_proto::shared::dns_record::Record;
use kraken_proto::shared::dns_txt_scan::Info;
use kraken_proto::shared::{
    spf_directive, spf_part, Aaaa, Aaaa, Address, Address, AttackTechnique, CertEntry, CertEntry,
    DnsRecord, DnsRecord, DnsTxtKnownEntry, DnsTxtScan, GenericRecord, GenericRecord, Net,
    SpfDirective, SpfExplanationModifier, SpfInfo, SpfMechanismA, SpfMechanismAll,
    SpfMechanismExists, SpfMechanismInclude, SpfMechanismIp, SpfMechanismMx, SpfMechanismPtr,
    SpfPart, SpfQualifier, SpfRedirectModifier, SpfUnknownModifier, A, A,
};
use kraken_proto::{
    any_attack_response, shared, shared, test_ssl_scans, test_ssl_service,
    BruteforceSubdomainRequest, BruteforceSubdomainRequest, BruteforceSubdomainResponse,
    BruteforceSubdomainResponse, CertificateTransparencyRequest, CertificateTransparencyRequest,
    CertificateTransparencyResponse, CertificateTransparencyResponse, DnsResolutionRequest,
    DnsResolutionRequest, DnsResolutionResponse, DnsResolutionResponse, DnsTxtScanRequest,
    DnsTxtScanResponse, HostsAliveRequest, HostsAliveRequest, HostsAliveResponse,
    HostsAliveResponse, ServiceCertainty, ServiceCertainty, ServiceDetectionRequest,
    ServiceDetectionRequest, ServiceDetectionResponse, ServiceDetectionResponse, StartTlsProtocol,
    TcpPortScanRequest, TcpPortScanRequest, TcpPortScanResponse, TcpPortScanResponse,
    TestSslFinding, TestSslRequest, TestSslResponse, TestSslScanResult, TestSslService,
    TestSslSeverity, UdpServiceDetectionRequest, UdpServiceDetectionRequest,
    UdpServiceDetectionResponse, UdpServiceDetectionResponse,
};
use log::error;
use prost_types::Timestamp;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::backlog::Backlog;
use crate::modules::bruteforce_subdomains::{
    bruteforce_subdomains, BruteforceSubdomainResult, BruteforceSubdomainsSettings,
};
use crate::modules::certificate_transparency::{query_ct_api, CertificateTransparencySettings};
use crate::modules::dns::spf::{SPFMechanism, SPFPart, SPFQualifier};
use crate::modules::dns::txt::{start_dns_txt_scan, DnsTxtScanSettings, TxtScanInfo};
use crate::modules::dns::{dns_resolution, DnsRecordResult, DnsResolutionSettings};
use crate::modules::host_alive::icmp_scan::{start_icmp_scan, IcmpScanSettings};
use crate::modules::port_scanner::tcp_con::{start_tcp_con_port_scan, TcpPortScannerSettings};
use crate::modules::service_detection::udp::{
    start_udp_service_detection, UdpServiceDetectionSettings,
};
use crate::modules::service_detection::{detect_service, DetectServiceSettings, Service};
use crate::modules::testssl::{self, run_testssl};

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

    type RunTcpPortScanStream =
        Pin<Box<dyn Stream<Item = Result<TcpPortScanResponse, Status>> + Send>>;

    async fn run_tcp_port_scan(
        &self,
        request: Request<TcpPortScanRequest>,
    ) -> Result<Response<Self::RunTcpPortScanStream>, Status> {
        let req = request.into_inner();

        let attack_uuid = Uuid::parse_str(&req.attack_uuid)
            .map_err(|_| Status::invalid_argument("attack_uuid has to be an Uuid"))?;

        let mut ports = req
            .ports
            .into_iter()
            .map(RangeInclusive::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        if ports.is_empty() {
            ports.push(1..=u16::MAX);
        }

        let settings = TcpPortScannerSettings {
            addresses: req
                .targets
                .into_iter()
                .map(IpNetwork::try_from)
                .collect::<Result<_, _>>()?,
            ports,
            timeout: Duration::from_millis(req.timeout),
            max_retries: req.max_retries,
            retry_interval: Duration::from_millis(req.retry_interval),
            concurrent_limit: req.concurrent_limit,
            skip_icmp_check: req.skip_icmp_check,
        };

        self.stream_attack(
            attack_uuid,
            {
                |tx| async move {
                    start_tcp_con_port_scan(settings, tx)
                        .await
                        .map_err(|err| Status::unknown(err.to_string()))
                }
            },
            |value| TcpPortScanResponse {
                address: Some(Address::from(value.ip())),
                port: value.port() as u32,
            },
            any_attack_response::Response::TcpPortScan,
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

    async fn service_detection(
        &self,
        request: Request<ServiceDetectionRequest>,
    ) -> Result<Response<ServiceDetectionResponse>, Status> {
        let request = request.into_inner();
        let settings = DetectServiceSettings {
            socket: SocketAddr::new(
                IpAddr::try_from(
                    request
                        .address
                        .clone()
                        .ok_or(Status::invalid_argument("Missing address"))?,
                )?,
                request
                    .port
                    .try_into()
                    .map_err(|_| Status::invalid_argument("Port is out of range"))?,
            ),
            timeout: Duration::from_millis(request.timeout),
            always_run_everything: false,
        };

        let service = detect_service(settings).await.map_err(|err| {
            error!("Service detection failed: {err:?}");
            Status::internal("Service detection failed. See logs")
        })?;

        Ok(Response::new(match service {
            Service::Unknown => ServiceDetectionResponse {
                response_type: ServiceCertainty::Unknown as _,
                services: Vec::new(),
                address: request.address,
                port: request.port,
            },
            Service::Maybe(services) => ServiceDetectionResponse {
                response_type: ServiceCertainty::Maybe as _,
                services: services.iter().map(|s| s.to_string()).collect(),
                address: request.address,
                port: request.port,
            },
            Service::Definitely(service) => ServiceDetectionResponse {
                response_type: ServiceCertainty::Definitely as _,
                services: vec![service.to_string()],
                address: request.address,
                port: request.port,
            },
        }))
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
            ip: IpAddr::try_from(
                request
                    .address
                    .clone()
                    .ok_or(Status::invalid_argument("Missing address"))?,
            )?,
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
                address: request.address.clone(),
                port: value.port as u32,
                certainty: match value.service {
                    Service::Unknown => ServiceCertainty::Unknown as _,
                    Service::Maybe(_) => ServiceCertainty::Maybe as _,
                    Service::Definitely(_) => ServiceCertainty::Definitely as _,
                },
                services: match value.service {
                    Service::Unknown => Vec::new(),
                    Service::Maybe(services) => services.iter().map(|s| s.to_string()).collect(),
                    Service::Definitely(service) => vec![service.to_string()],
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
            concurrent_limit: req.concurrent_limit,
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
                    rule: value.rule,
                    info: Some(match value.info {
                        TxtScanInfo::HasGoogleAccount => {
                            Info::WellKnown(DnsTxtKnownEntry::HasGoogleAccount as _)
                        }
                        TxtScanInfo::HasDocusignAccount => {
                            Info::WellKnown(DnsTxtKnownEntry::HasDocusignAccount as _)
                        }
                        TxtScanInfo::HasAppleAccount => {
                            Info::WellKnown(DnsTxtKnownEntry::HasAppleAccount as _)
                        }
                        TxtScanInfo::HasFacebookAccount => {
                            Info::WellKnown(DnsTxtKnownEntry::HasFacebookAccount as _)
                        }
                        TxtScanInfo::HasHubspotAccount => {
                            Info::WellKnown(DnsTxtKnownEntry::HasHubspotAccount as _)
                        }
                        TxtScanInfo::HasMsDynamics365 => {
                            Info::WellKnown(DnsTxtKnownEntry::HasMsDynamics365 as _)
                        }
                        TxtScanInfo::HasStripeAccount => {
                            Info::WellKnown(DnsTxtKnownEntry::HasStripeAccount as _)
                        }
                        TxtScanInfo::HasOneTrustSso => {
                            Info::WellKnown(DnsTxtKnownEntry::HasOneTrustSso as _)
                        }
                        TxtScanInfo::HasBrevoAccount => {
                            Info::WellKnown(DnsTxtKnownEntry::HasBrevoAccount as _)
                        }
                        TxtScanInfo::HasGlobalsignAccount => {
                            Info::WellKnown(DnsTxtKnownEntry::HasGlobalsignAccount as _)
                        }
                        TxtScanInfo::HasGlobalsignSMime => {
                            Info::WellKnown(DnsTxtKnownEntry::HasGlobalsignSMime as _)
                        }
                        TxtScanInfo::OwnsAtlassianAccounts => {
                            Info::WellKnown(DnsTxtKnownEntry::OwnsAtlassianAccounts as _)
                        }
                        TxtScanInfo::OwnsZoomAccounts => {
                            Info::WellKnown(DnsTxtKnownEntry::OwnsZoomAccounts as _)
                        }
                        TxtScanInfo::EmailProtonMail => {
                            Info::WellKnown(DnsTxtKnownEntry::EmailProtonMail as _)
                        }
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
                    }),
                }),
            },
            any_attack_response::Response::DnsTxtScan,
        )
    }

    async fn test_ssl(
        &self,
        request: Request<TestSslRequest>,
    ) -> Result<Response<TestSslResponse>, Status> {
        let TestSslRequest {
            attack_uuid: _,
            uri,
            ip,
            port,
            connect_timeout,
            openssl_timeout,
            v6,
            basic_auth,
            starttls,
            scans,
        } = request.into_inner();
        let settings = testssl::TestSSLSettings {
            uri,
            ip: IpAddr::try_from(ip.ok_or(Status::invalid_argument("Missing ip"))?)?,
            port: port as u16,
            connect_timeout,
            openssl_timeout,
            v6: v6.unwrap_or(false),
            basic_auth: basic_auth.map(|x| (x.username, x.password)),
            starttls: starttls
                .map(|x| {
                    StartTlsProtocol::try_from(x).map_err(|_| {
                        Status::invalid_argument(format!(
                            "Invalid enum value {x} for StartTlsProtocol"
                        ))
                    })
                })
                .transpose()?
                .map(|x| match x {
                    StartTlsProtocol::Ftp => testssl::StartTLSProtocol::FTP,
                    StartTlsProtocol::Smtp => testssl::StartTLSProtocol::SMTP,
                    StartTlsProtocol::Pop3 => testssl::StartTLSProtocol::POP3,
                    StartTlsProtocol::Imap => testssl::StartTLSProtocol::IMAP,
                    StartTlsProtocol::Xmpp => testssl::StartTLSProtocol::XMPP,
                    StartTlsProtocol::Lmtp => testssl::StartTLSProtocol::LMTP,
                    StartTlsProtocol::Nntp => testssl::StartTLSProtocol::NNTP,
                    StartTlsProtocol::Postgres => testssl::StartTLSProtocol::Postgres,
                    StartTlsProtocol::MySql => testssl::StartTLSProtocol::MySQL,
                }),
            scans: scans
                .and_then(|x| x.testssl_scans)
                .map(|x| match x {
                    test_ssl_scans::TestsslScans::All(true) => testssl::TestSSLScans::All,
                    test_ssl_scans::TestsslScans::All(false) => testssl::TestSSLScans::Default,
                    test_ssl_scans::TestsslScans::Manual(x) => testssl::TestSSLScans::Manual {
                        protocols: x.protocols,
                        grease: x.grease,
                        ciphers: x.ciphers,
                        pfs: x.pfs,
                        server_preferences: x.server_preferences,
                        server_defaults: x.server_defaults,
                        header_response: x.header_response,
                        vulnerabilities: x.vulnerabilities,
                        cipher_tests_all: x.cipher_tests_all,
                        cipher_tests_per_proto: x.cipher_tests_per_proto,
                        browser_simulations: x.browser_simulations,
                    },
                })
                .unwrap_or_default(),
        };

        let services = run_testssl(settings)
            .await
            .map_err(|err| {
                error!("testssl failed: {err:?}");
                Status::internal("testssl failed. See logs")
            })?
            .scan_result;

        fn conv_finding(finding: testssl::Finding) -> TestSslFinding {
            let mitre = testssl::categorize(&finding);
            TestSslFinding {
                id: finding.id,
                severity: match finding.severity {
                    testssl::Severity::Debug => TestSslSeverity::Debug,
                    testssl::Severity::Info => TestSslSeverity::Info,
                    testssl::Severity::Warn => TestSslSeverity::Warn,
                    testssl::Severity::Fatal => TestSslSeverity::Fatal,
                    testssl::Severity::Ok => TestSslSeverity::Ok,
                    testssl::Severity::Low => TestSslSeverity::Low,
                    testssl::Severity::Medium => TestSslSeverity::Medium,
                    testssl::Severity::High => TestSslSeverity::High,
                    testssl::Severity::Critical => TestSslSeverity::Critical,
                }
                .into(),
                finding: finding.finding,
                cve: finding.cve,
                cwe: finding.cwe,
                mitre: mitre.map(AttackTechnique::from),
            }
        }
        fn conv_findings(findings: Vec<testssl::Finding>) -> Vec<TestSslFinding> {
            findings.into_iter().map(conv_finding).collect()
        }

        Ok(Response::new(TestSslResponse {
            services: services
                .into_iter()
                .map(|service| TestSslService {
                    testssl_service: Some(match service {
                        testssl::Service::Result(service) => {
                            test_ssl_service::TestsslService::Result(TestSslScanResult {
                                target_host: service.target_host,
                                ip: service.ip,
                                port: service.port,
                                rdns: service.rdns,
                                service: service.service,
                                pretest: conv_findings(service.pretest),
                                protocols: conv_findings(service.protocols),
                                grease: conv_findings(service.grease),
                                ciphers: conv_findings(service.ciphers),
                                pfs: conv_findings(service.pfs),
                                server_preferences: conv_findings(service.server_preferences),
                                server_defaults: conv_findings(service.server_defaults),
                                header_response: conv_findings(service.header_response),
                                vulnerabilities: conv_findings(service.vulnerabilities),
                                cipher_tests: conv_findings(service.cipher_tests),
                                browser_simulations: conv_findings(service.browser_simulations),
                            })
                        }
                        testssl::Service::Error(finding) => {
                            test_ssl_service::TestsslService::Error(conv_finding(finding))
                        }
                    }),
                })
                .collect(),
        }))
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
