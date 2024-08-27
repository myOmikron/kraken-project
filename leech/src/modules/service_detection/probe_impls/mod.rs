//! Rust implementations of probe files

pub mod postgres;

/// This module's content can be copied as a template for probe file implementations
#[allow(dead_code)]
pub mod template {
    use tokio::net::UdpSocket;

    use crate::modules::service_detection::generated::Match;
    use crate::modules::service_detection::tcp::OneShotTcpSettings;
    use crate::modules::service_detection::DynResult;

    pub async fn probe_tcp(_settings: &OneShotTcpSettings) -> DynResult<Match> {
        Ok(Match::No)
    }

    pub async fn probe_tls(
        _settings: &OneShotTcpSettings,
        _alpn: Option<&str>,
    ) -> DynResult<Match> {
        Ok(Match::No)
    }

    pub async fn probe_udp(_socket: &mut UdpSocket) -> DynResult<Match> {
        Ok(Match::No)
    }
}
