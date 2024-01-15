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
use kraken_proto::shared::{Aaaa, Address, CertEntry, DnsRecord, GenericRecord, A};
use kraken_proto::{
    any_attack_response, shared, BruteforceSubdomainRequest, BruteforceSubdomainResponse,
    CertificateTransparencyRequest, CertificateTransparencyResponse, DnsResolutionRequest,
    DnsResolutionResponse, HostsAliveRequest, HostsAliveResponse, ServiceDetectionRequest,
    ServiceDetectionResponse, ServiceDetectionResponseType, TcpPortScanRequest,
    TcpPortScanResponse,
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
use crate::modules::dns::{dns_resolution, DnsRecordResult, DnsResolutionSettings};
use crate::modules::host_alive::icmp_scan::{start_icmp_scan, IcmpScanSettings};
use crate::modules::port_scanner::tcp_con::{start_tcp_con_port_scan, TcpPortScannerSettings};
use crate::modules::service_detection::{detect_service, DetectServiceSettings, Service};

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
                response_type: ServiceDetectionResponseType::Unknown as _,
                services: Vec::new(),
                address: request.address,
                port: request.port,
            },
            Service::Maybe(services) => ServiceDetectionResponse {
                response_type: ServiceDetectionResponseType::Maybe as _,
                services: services.iter().map(|s| s.to_string()).collect(),
                address: request.address,
                port: request.port,
            },
            Service::Definitely(service) => ServiceDetectionResponse {
                response_type: ServiceDetectionResponseType::Definitely as _,
                services: vec![service.to_string()],
                address: request.address,
                port: request.port,
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
