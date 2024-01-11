use std::str::FromStr;

use ipnetwork::IpNetwork;
use kraken_proto::{
    test_ssl_scans, test_ssl_service, BasicAuth, StartTlsProtocol, TestSslRequest, TestSslResponse,
    TestSslScanResult, TestSslScans,
};
use log::error;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use uuid::Uuid;

use crate::api::handler::attacks::schema::StartTLSProtocol;
use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::models::{
    AggregationSource, AggregationTable, DomainCertainty, HostCertainty, PortCertainty,
    PortProtocol, ServiceCertainty, SourceType, TestSSLResult, TestSSLResultInsert,
};
use crate::modules::attacks::{AttackContext, AttackError, HandleAttackResponse, TestSSLParams};

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
                    Ok(port) => port,
                    Err(err) => {
                        error!("Testssl didn't return a valid port: {err}");
                        return Ok(());
                    }
                };

                let mut domain = rdns.clone();
                if domain.ends_with('.') {
                    domain.pop();
                }
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

                let service_uuid = GLOBAL
                    .aggregator
                    .aggregate_service(
                        self.workspace.uuid,
                        host_uuid,
                        Some(port_uuid),
                        &service,
                        ServiceCertainty::MaybeVerified, // TODO might be DefinitelyVerified?
                    )
                    .await?;

                let source_uuid = insert!(&mut tx, TestSSLResult)
                    .return_primary_key()
                    .single(&TestSSLResultInsert {
                        uuid: Uuid::new_v4(),
                        attack: ForeignModelByField::Key(self.attack_uuid),
                        target_host,
                        ip,
                        port: port as i32,
                        rdns,
                        service,
                    })
                    .await?;

                insert!(&mut tx, AggregationSource)
                    .return_nothing()
                    .bulk([
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
                        AggregationSource {
                            uuid: Uuid::new_v4(),
                            workspace: ForeignModelByField::Key(self.workspace.uuid),
                            source_type: SourceType::TestSSL,
                            source_uuid,
                            aggregated_table: AggregationTable::Service,
                            aggregated_uuid: service_uuid,
                        },
                    ])
                    .await?;

                tx.commit().await?;
            }
        }
        Ok(())
    }
}
