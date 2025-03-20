//! In this module is the definition of the gRPC services

use std::pin::Pin;

use futures::Stream;
use kraken_proto::req_attack_service_server::ReqAttackService;
use kraken_proto::BruteforceSubdomainRequest;
use kraken_proto::CertificateTransparencyRequest;
use kraken_proto::CertificateTransparencyResponse;
use kraken_proto::DnsResolutionRequest;
use kraken_proto::DnsTxtScanRequest;
use kraken_proto::HostsAliveRequest;
use kraken_proto::OsDetectionRequest;
use kraken_proto::ServiceDetectionRequest;
use kraken_proto::TestSslRequest;
use kraken_proto::TestSslResponse;
use kraken_proto::UdpServiceDetectionRequest;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::Response;
use tonic::Status;
use uuid::Uuid;

use crate::backlog::Backlog;
use crate::modules::bruteforce_subdomains::BruteforceSubdomain;
use crate::modules::certificate_transparency::CertificateTransparency;
use crate::modules::dns::txt::DnsTxtScan;
use crate::modules::dns::DnsResolution;
use crate::modules::host_alive::icmp_scan::IcmpScan;
use crate::modules::os_detection::OsDetection;
use crate::modules::service_detection::tcp::TcpServiceDetection;
use crate::modules::service_detection::udp::UdpServiceDetection;
use crate::modules::testssl::TestSSL;
use crate::modules::Attack;
use crate::modules::StreamedAttack;

/// The Attack service
pub struct Attacks {
    pub(crate) backlog: Backlog,
}

#[tonic::async_trait]
impl ReqAttackService for Attacks {
    type BruteforceSubdomainsStream = AttackStream<BruteforceSubdomain>;
    async fn bruteforce_subdomains(
        &self,
        request: Request<BruteforceSubdomainRequest>,
    ) -> Result<Response<Self::BruteforceSubdomainsStream>, Status> {
        self.streamed_attack::<BruteforceSubdomain>(request).await
    }

    async fn query_certificate_transparency(
        &self,
        request: Request<CertificateTransparencyRequest>,
    ) -> Result<Response<CertificateTransparencyResponse>, Status> {
        self.normal_attack::<CertificateTransparency>(request).await
    }

    type ServiceDetectionStream = AttackStream<TcpServiceDetection>;
    async fn service_detection(
        &self,
        request: Request<ServiceDetectionRequest>,
    ) -> Result<Response<Self::ServiceDetectionStream>, Status> {
        self.streamed_attack::<TcpServiceDetection>(request).await
    }

    type UdpServiceDetectionStream = AttackStream<UdpServiceDetection>;
    async fn udp_service_detection(
        &self,
        request: Request<UdpServiceDetectionRequest>,
    ) -> Result<Response<Self::UdpServiceDetectionStream>, Status> {
        self.streamed_attack::<UdpServiceDetection>(request).await
    }

    type HostsAliveCheckStream = AttackStream<IcmpScan>;
    async fn hosts_alive_check(
        &self,
        request: Request<HostsAliveRequest>,
    ) -> Result<Response<Self::HostsAliveCheckStream>, Status> {
        self.streamed_attack::<IcmpScan>(request).await
    }

    type DnsResolutionStream = AttackStream<DnsResolution>;
    async fn dns_resolution(
        &self,
        request: Request<DnsResolutionRequest>,
    ) -> Result<Response<Self::DnsResolutionStream>, Status> {
        self.streamed_attack::<DnsResolution>(request).await
    }

    type DnsTxtScanStream = AttackStream<DnsTxtScan>;
    async fn dns_txt_scan(
        &self,
        request: Request<DnsTxtScanRequest>,
    ) -> Result<Response<Self::DnsTxtScanStream>, Status> {
        self.streamed_attack::<DnsTxtScan>(request).await
    }

    type OsDetectionStream = AttackStream<OsDetection>;
    async fn os_detection(
        &self,
        request: Request<OsDetectionRequest>,
    ) -> Result<Response<Self::OsDetectionStream>, Status> {
        self.streamed_attack::<OsDetection>(request).await
    }

    async fn test_ssl(
        &self,
        request: Request<TestSslRequest>,
    ) -> Result<Response<TestSslResponse>, Status> {
        self.normal_attack::<TestSSL>(request).await
    }
}

type AttackStream<A> =
    Pin<Box<dyn Stream<Item = Result<<A as StreamedAttack>::Response, Status>> + Send>>;
impl Attacks {
    /// Perform an attack which returns its result
    async fn normal_attack<A: Attack + 'static>(
        &self,
        request: Request<A::Request>,
    ) -> Result<Response<A::Response>, Status> {
        let request = request.into_inner();
        let settings = A::decode_settings(request)?;
        match A::execute(settings).await {
            Ok(output) => Ok(Response::new(A::encode_output(output))),
            Err(error) => Err(Status::unknown(error.to_string())),
        }
    }

    /// Perform an attack which streams its results
    async fn streamed_attack<A: StreamedAttack + 'static>(
        &self,
        request: Request<A::Request>,
    ) -> Result<Response<AttackStream<A>>, Status> {
        let request = request.into_inner();
        let attack_uuid = Uuid::parse_str(A::get_attack_uuid(&request))
            .map_err(|_| Status::invalid_argument("attack_uuid has to be an Uuid"))?;
        let settings = A::decode_settings(request)?;

        let (from_attack, mut to_middleware) = mpsc::channel::<A::Output>(16);
        let (from_middleware, to_stream) = mpsc::channel::<Result<A::Response, Status>>(1);

        // Spawn attack
        let attack = A::execute(settings, from_attack);
        let error_from_attack = from_middleware.clone();
        tokio::spawn(async move {
            if let Err(err) = attack.await {
                let _ = error_from_attack
                    .send(Err(Status::unknown(err.to_string())))
                    .await;
            }
        });

        let backlog = self.backlog.clone();

        // Spawn middleware
        tokio::spawn({
            async move {
                while let Some(output) = to_middleware.recv().await {
                    let response = A::encode_output(output);

                    // Try sending the item over the rpc stream
                    let result = from_middleware.send(Ok(response)).await;

                    // Failure means the receiver i.e. outgoing stream has been closed and dropped
                    if let Err(error) = result {
                        let Ok(response) = error.0 else {
                            unreachable!("We tried to send an `Ok(_)` above");
                        };

                        // Save this item to the backlog
                        backlog
                            .store(attack_uuid, (A::BACKLOG_WRAPPER)(response))
                            .await;

                        // Drain all remaining items into the backlog, because the stream is gone
                        while let Some(output) = to_middleware.recv().await {
                            let response = A::encode_output(output);
                            backlog
                                .store(attack_uuid, (A::BACKLOG_WRAPPER)(response))
                                .await;
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
