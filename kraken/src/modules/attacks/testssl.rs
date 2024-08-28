use std::str::FromStr;

use futures::TryStreamExt;
use ipnetwork::IpNetwork;
use kraken_proto::shared::Address;
use kraken_proto::test_ssl_scans;
use kraken_proto::test_ssl_service;
use kraken_proto::BasicAuth;
use kraken_proto::StartTlsProtocol;
use kraken_proto::TestSslRequest;
use kraken_proto::TestSslResponse;
use kraken_proto::TestSslScanResult;
use kraken_proto::TestSslScans;
use kraken_proto::TestSslSeverity;
use log::error;
use rorm::and;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::handler::attacks::schema::StartTLSProtocol;
use crate::api::handler::services::schema::ServiceProtocols;
use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::models::AggregationSource;
use crate::models::AggregationTable;
use crate::models::DomainCertainty;
use crate::models::HostCertainty;
use crate::models::PortCertainty;
use crate::models::PortProtocol;
use crate::models::Service;
use crate::models::SourceType;
use crate::models::TestSSLResultFinding;
use crate::models::TestSSLResultFindingInsert;
use crate::models::TestSSLResultHeader;
use crate::models::TestSSLResultHeaderInsert;
use crate::models::TestSSLSection;
use crate::models::TestSSLSeverity;
use crate::modules::attacks::AttackContext;
use crate::modules::attacks::AttackError;
use crate::modules::attacks::HandleAttackResponse;
use crate::modules::attacks::TestSSLParams;

impl AttackContext {
    /// Executes the "testssl" attack
    pub async fn testssl(
        &self,
        mut leech: LeechClient,
        params: TestSSLParams,
    ) -> Result<(), AttackError> {
        let request = TestSslRequest {
            attack_uuid: self.attack_uuid.to_string(),
            uri: params.uri,
            ip: Some(Address::from(params.ip)),
            port: params.port as u32,
            connect_timeout: params.connect_timeout,
            openssl_timeout: params.openssl_timeout,
            v6: Some(true),
            basic_auth: params
                .basic_auth
                .map(|[username, password]| BasicAuth { username, password }),
            starttls: params.starttls.map(|p| {
                match p {
                    StartTLSProtocol::FTP => StartTlsProtocol::Ftp,
                    StartTLSProtocol::SMTP => StartTlsProtocol::Smtp,
                    StartTLSProtocol::POP3 => StartTlsProtocol::Pop3,
                    StartTLSProtocol::IMAP => StartTlsProtocol::Imap,
                    StartTLSProtocol::XMPP => StartTlsProtocol::Xmpp,
                    StartTLSProtocol::LMTP => StartTlsProtocol::Lmtp,
                    StartTLSProtocol::NNTP => StartTlsProtocol::Nntp,
                    StartTLSProtocol::Postgres => StartTlsProtocol::Postgres,
                    StartTLSProtocol::MySQL => StartTlsProtocol::MySql,
                }
                .into()
            }),
            scans: Some(TestSslScans {
                testssl_scans: Some(test_ssl_scans::TestsslScans::All(true)),
            }),
        };
        self.handle_response(leech.test_ssl(request).await?.into_inner())
            .await?;

        Ok(())
    }
}

impl HandleAttackResponse<TestSslResponse> for AttackContext {
    async fn handle_response(&self, response: TestSslResponse) -> Result<(), AttackError> {
        for service in response.services {
            if let Some(test_ssl_service::TestsslService::Result(result)) = service.testssl_service
            {
                let mut tx = GLOBAL.db.start_transaction().await?;

                let TestSslScanResult {
                    target_host,
                    ip,
                    port,
                    rdns,
                    service,
                    pretest,
                    protocols,
                    grease,
                    ciphers,
                    pfs,
                    server_preferences,
                    server_defaults,
                    header_response,
                    vulnerabilities,
                    cipher_tests,
                    browser_simulations,
                } = result;

                let ip = match IpNetwork::from_str(&ip) {
                    Ok(ip) => ip,
                    Err(err) => {
                        error!("Testssl didn't return a valid ip: {err}");
                        return Ok(());
                    }
                };

                let port = match u16::from_str(&port) {
                    Ok(port) if port > 0 => port,
                    Ok(port) => {
                        error!("Testssl didn't return a valid port: {port}");
                        return Ok(());
                    }
                    Err(err) => {
                        error!("Testssl didn't return a valid port: {err}");
                        return Ok(());
                    }
                };

                let mut domain = rdns.clone();
                if domain.ends_with('.') {
                    domain.pop();
                }

                let findings = [
                    (pretest, TestSSLSection::Pretest),
                    (protocols, TestSSLSection::Protocols),
                    (grease, TestSSLSection::Grease),
                    (ciphers, TestSSLSection::Ciphers),
                    (pfs, TestSSLSection::Pfs),
                    (server_preferences, TestSSLSection::ServerPreferences),
                    (server_defaults, TestSSLSection::ServerDefaults),
                    (header_response, TestSSLSection::HeaderResponse),
                    (vulnerabilities, TestSSLSection::Vulnerabilities),
                    (cipher_tests, TestSSLSection::CipherTests),
                    (browser_simulations, TestSSLSection::BrowserSimulations),
                ]
                .into_iter()
                .flat_map(|(findings, section)| {
                    findings
                        .into_iter()
                        .filter(|finding| finding.id != "cert")
                        .map(move |finding| {
                            Ok(TestSSLResultFindingInsert {
                                uuid: Uuid::new_v4(),
                                attack: ForeignModelByField::Key(self.attack_uuid),
                                section,
                                key: finding.id,
                                value: finding.finding,
                                testssl_severity: match TestSslSeverity::try_from(finding.severity)
                                    .map_err(|e| AttackError::Custom(Box::new(e)))?
                                {
                                    TestSslSeverity::Debug => TestSSLSeverity::Debug,
                                    TestSslSeverity::Info => TestSSLSeverity::Info,
                                    TestSslSeverity::Warn => TestSSLSeverity::Warn,
                                    TestSslSeverity::Fatal => TestSSLSeverity::Fatal,
                                    TestSslSeverity::Ok => TestSSLSeverity::Ok,
                                    TestSslSeverity::Low => TestSSLSeverity::Low,
                                    TestSslSeverity::Medium => TestSSLSeverity::Medium,
                                    TestSslSeverity::High => TestSSLSeverity::High,
                                    TestSslSeverity::Critical => TestSSLSeverity::Critical,
                                },
                                cve: finding.cve,
                                cwe: finding.cwe,
                            })
                        })
                })
                .collect::<Result<Vec<_>, AttackError>>()?;

                let domain_uuid = GLOBAL
                    .aggregator
                    .aggregate_domain(
                        self.workspace.uuid,
                        &domain,
                        DomainCertainty::Unverified,
                        self.user.uuid,
                    )
                    .await?;

                let host_uuid = GLOBAL
                    .aggregator
                    .aggregate_host(self.workspace.uuid, ip, HostCertainty::Verified)
                    .await?;

                let port_uuid = GLOBAL
                    .aggregator
                    .aggregate_port(
                        self.workspace.uuid,
                        host_uuid,
                        port,
                        PortProtocol::Tcp,
                        PortCertainty::Verified,
                    )
                    .await?;

                // testssl didn't gather the information to aggregate a service, but it should attach to an existing one
                let service_uuids = query!(&GLOBAL.db, (Service::F.uuid, Service::F.protocols))
                    .condition(and![
                        Service::F.workspace.equals(self.workspace.uuid),
                        Service::F.host.equals(host_uuid),
                        Service::F.port.equals(port_uuid),
                    ])
                    .stream()
                    .try_filter_map(|(uuid, protocols)| async move {
                        Ok(matches!(
                            PortProtocol::Tcp.decode_service(protocols),
                            ServiceProtocols::Tcp { tls: true, .. }
                        )
                        .then_some(uuid))
                    })
                    .try_collect::<Vec<Uuid>>()
                    .await?;

                let source_uuid = insert!(&mut tx, TestSSLResultHeader)
                    .return_primary_key()
                    .single(&TestSSLResultHeaderInsert {
                        uuid: Uuid::new_v4(),
                        attack: ForeignModelByField::Key(self.attack_uuid),
                        domain: target_host,
                        ip,
                        port: port as i32,
                        rdns,
                        service,
                    })
                    .await?;

                insert!(&mut tx, TestSSLResultFinding)
                    .return_nothing()
                    .bulk(&findings)
                    .await?;

                insert!(&mut tx, AggregationSource)
                    .return_nothing()
                    .bulk(
                        [
                            AggregationSource {
                                uuid: Uuid::new_v4(),
                                workspace: ForeignModelByField::Key(self.workspace.uuid),
                                source_type: SourceType::TestSSL,
                                source_uuid,
                                aggregated_table: AggregationTable::Domain,
                                aggregated_uuid: domain_uuid,
                            },
                            AggregationSource {
                                uuid: Uuid::new_v4(),
                                workspace: ForeignModelByField::Key(self.workspace.uuid),
                                source_type: SourceType::TestSSL,
                                source_uuid,
                                aggregated_table: AggregationTable::Host,
                                aggregated_uuid: host_uuid,
                            },
                            AggregationSource {
                                uuid: Uuid::new_v4(),
                                workspace: ForeignModelByField::Key(self.workspace.uuid),
                                source_type: SourceType::TestSSL,
                                source_uuid,
                                aggregated_table: AggregationTable::Port,
                                aggregated_uuid: port_uuid,
                            },
                        ]
                        .into_iter()
                        .chain(service_uuids.into_iter().map(|aggregated_uuid| {
                            AggregationSource {
                                uuid: Uuid::new_v4(),
                                workspace: ForeignModelByField::Key(self.workspace.uuid),
                                source_type: SourceType::TestSSL,
                                source_uuid,
                                aggregated_table: AggregationTable::Service,
                                aggregated_uuid,
                            }
                        })),
                    )
                    .await?;

                tx.commit().await?;
            }
        }
        Ok(())
    }
}
