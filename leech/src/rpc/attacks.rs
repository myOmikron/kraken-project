//! In this module is the definition of the gRPC services

use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;

use chrono::{Datelike, Timelike};
use futures::Stream;
use log::warn;
use prost_types::Timestamp;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status};

use crate::modules::bruteforce_subdomains::{
    bruteforce_subdomains, BruteforceSubdomainResult, BruteforceSubdomainsSettings,
};
use crate::modules::certificate_transparency::{query_ct_api, CertificateTransparencySettings};
use crate::modules::port_scanner::tcp_con::{start_tcp_con_port_scan, TcpPortScannerSettings};
use crate::rpc::rpc_attacks::req_attack_service_server::ReqAttackService;
use crate::rpc::rpc_attacks::{
    BruteforceSubdomainRequest, BruteforceSubdomainResponse, CertEntry,
    CertificateTransparencyRequest, CertificateTransparencyResponse, TcpPortScanRequest,
    TcpPortScanResponse,
};

/// The Attack service
#[derive(Debug)]
pub struct Attacks;

#[tonic::async_trait]
impl ReqAttackService for Attacks {
    type BruteforceSubdomainsStream =
        Pin<Box<dyn Stream<Item = Result<BruteforceSubdomainResponse, Status>> + Send>>;

    async fn bruteforce_subdomains(
        &self,
        request: Request<BruteforceSubdomainRequest>,
    ) -> Result<Response<Self::BruteforceSubdomainsStream>, Status> {
        let (rpc_tx, rpc_rx) = mpsc::channel(16);
        let (tx, mut rx) = mpsc::channel::<BruteforceSubdomainResult>(16);

        tokio::spawn(async move {
            while let Some(res) = rx.recv().await {
                let rpc_res = res.into();

                if let Err(err) = rpc_tx.send(Ok(rpc_res)).await {
                    warn!("Could not send to rpc_tx: {err}");
                    // TODO: Save to backlog and use push api
                }
            }
        });

        let req = request.into_inner();
        let settings = BruteforceSubdomainsSettings {
            domain: req.domain,
            wordlist_path: req.wordlist_path.parse().unwrap(),
            concurrent_limit: req.concurrent_limit,
        };
        if let Err(err) = bruteforce_subdomains(settings, tx).await {
            warn!("Attack {} returned error: {err}", req.attack_id);
            // TODO: Send error to grpc client
        }

        let output_stream = ReceiverStream::new(rpc_rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::BruteforceSubdomainsStream
        ))
    }

    type RunTcpPortScanStream =
        Pin<Box<dyn Stream<Item = Result<TcpPortScanResponse, Status>> + Send>>;

    async fn run_tcp_port_scan(
        &self,
        request: Request<TcpPortScanRequest>,
    ) -> Result<Response<Self::RunTcpPortScanStream>, Status> {
        let (rpc_tx, rpc_rx) = mpsc::channel(16);
        let (tx, mut rx) = mpsc::channel::<SocketAddr>(16);

        tokio::spawn(async move {
            while let Some(addr) = rx.recv().await {
                if let Err(err) = rpc_tx.send(Ok(addr.into())).await {
                    warn!("Could not send to rpc_tx: {err}");
                    // TODO: Save to backlog and use push api
                }
            }
        });

        let req = request.into_inner();
        let settings = TcpPortScannerSettings {
            addresses: req.targets.into_iter().map(|addr| addr.into()).collect(),
            port_range: req.ports.into_iter().map(|p| p as u16).collect(),
            timeout: Duration::from_millis(req.timeout),
            max_retries: req.max_retries,
            retry_interval: Duration::from_millis(req.retry_interval),
            concurrent_limit: req.concurrent_limit,
            skip_icmp_check: req.skip_icmp_check,
        };

        if let Err(err) = start_tcp_con_port_scan(settings, tx).await {
            warn!("Attack {} returned error: {err}", req.attack_id);
            // TODO: Send error to grpc client
        }

        let output_stream = ReceiverStream::new(rpc_rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::RunTcpPortScanStream
        ))
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
                .map_err(|err| Status::new(Code::Unknown, err.to_string()))?
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
}
