//! In this module is the definition of the gRPC services

use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;

use chrono::{Datelike, Timelike};
use futures::Stream;
use log::error;
use prost_types::Timestamp;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::backlog::Backlog;
use crate::modules::bruteforce_subdomains::{
    bruteforce_subdomains, BruteforceSubdomainResult, BruteforceSubdomainsSettings,
};
use crate::modules::certificate_transparency::{query_ct_api, CertificateTransparencySettings};
use crate::modules::dns::{dns_resolution, DnsRecordResult, DnsResolutionSettings};
use crate::modules::host_alive::icmp_scan::{start_icmp_scan, IcmpScanSettings};
use crate::modules::port_scanner::tcp_con::{start_tcp_con_port_scan, TcpPortScannerSettings};
use crate::modules::service_detection::{detect_service, DetectServiceSettings, Service};
use crate::modules::testssl::{self, run_testssl};
use crate::rpc::rpc_attacks::req_attack_service_server::ReqAttackService;
use crate::rpc::rpc_attacks::shared::CertEntry;
use crate::rpc::rpc_attacks::{
    test_ssl_scans, test_ssl_service, BruteforceSubdomainRequest, BruteforceSubdomainResponse,
    CertificateTransparencyRequest, CertificateTransparencyResponse, DnsResolutionRequest,
    DnsResolutionResponse, HostsAliveRequest, HostsAliveResponse, ServiceDetectionRequest,
    ServiceDetectionResponse, ServiceDetectionResponseType, StartTlsProtocol, TcpPortScanRequest,
    TcpPortScanResponse, TestSslFinding, TestSslRequest, TestSslResponse, TestSslScanResult,
    TestSslService, TestSslSeverity,
};
use crate::rpc::utils::stream_attack;

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

        stream_attack(
            {
                |tx| async move {
                    bruteforce_subdomains(settings, tx)
                        .await
                        .map_err(|err| Status::unknown(err.to_string()))
                }
            },
            {
                let backlog = self.backlog.clone();
                move |item: BruteforceSubdomainResult| {
                    let backlog = backlog.clone();
                    async move {
                        backlog
                            .store_bruteforce_subdomains(attack_uuid, item.into())
                            .await;
                    }
                }
            },
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
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        if ports.is_empty() {
            ports.push(1..=u16::MAX);
        }

        let settings = TcpPortScannerSettings {
            addresses: req.targets.into_iter().map(|addr| addr.into()).collect(),
            ports,
            timeout: Duration::from_millis(req.timeout),
            max_retries: req.max_retries,
            retry_interval: Duration::from_millis(req.retry_interval),
            concurrent_limit: req.concurrent_limit,
            skip_icmp_check: req.skip_icmp_check,
        };

        stream_attack(
            {
                |tx| async move {
                    start_tcp_con_port_scan(settings, tx)
                        .await
                        .map_err(|err| Status::unknown(err.to_string()))
                }
            },
            {
                let backlog = self.backlog.clone();
                move |item| {
                    let backlog = backlog.clone();
                    async move {
                        backlog.store_tcp_port_scans(attack_uuid, item).await;
                    }
                }
            },
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
                request
                    .address
                    .ok_or(Status::invalid_argument("Missing address"))?
                    .into(),
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
                response_type: ServiceDetectionResponseType::Unknown as _,
                services: Vec::new(),
            },
            Service::Maybe(services) => ServiceDetectionResponse {
                response_type: ServiceDetectionResponseType::Maybe as _,
                services: services.iter().map(|s| s.to_string()).collect(),
            },
            Service::Definitely(service) => ServiceDetectionResponse {
                response_type: ServiceDetectionResponseType::Definitely as _,
                services: vec![service.to_string()],
            },
        }))
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
            addresses: req.targets.into_iter().map(|el| el.into()).collect(),
        };

        stream_attack(
            |tx| async move {
                start_icmp_scan(settings, tx)
                    .await
                    .map_err(|err| Status::unknown(err.to_string()))
            },
            {
                let backlog = self.backlog.clone();
                move |item| {
                    let backlog = backlog.clone();
                    async move { backlog.store_hosts_alive_check(attack_uuid, item).await }
                }
            },
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

        stream_attack(
            |tx| async move {
                dns_resolution(settings, tx)
                    .await
                    .map_err(|err| Status::unknown(err.to_string()))
            },
            {
                let backlog = self.backlog.clone();
                move |item: DnsRecordResult| {
                    let backlog = backlog.clone();
                    async move {
                        backlog.store_dns_resolution(attack_uuid, item.into()).await;
                    }
                }
            },
        )
    }

    async fn test_ssl(
        &self,
        request: Request<TestSslRequest>,
    ) -> Result<Response<TestSslResponse>, Status> {
        let TestSslRequest {
            attack_uuid,
            uri,
            connect_timeout,
            openssl_timeout,
            v6,
            basic_auth,
            starttls,
            scans,
        } = request.into_inner();
        let settings = testssl::TestSSLSettings {
            uri,
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
