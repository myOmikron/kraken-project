use crate::api::handler::attacks::schema::StartTLSProtocol;
use crate::chan::global::GLOBAL;
use crate::chan::leech_manager::LeechClient;
use crate::modules::attack_results::store_testssl_result;
use crate::modules::attacks::{AttackContext, AttackError, TestSSLParams};
use crate::rpc::rpc_definitions::{
    test_ssl_scans, BasicAuth, StartTlsProtocol, TestSslRequest, TestSslScans,
};

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
        let response = leech.test_ssl(request).await?.into_inner();
        store_testssl_result(&GLOBAL.db, self.attack_uuid, self.workspace.uuid, response).await?;

        Ok(())
    }
}
