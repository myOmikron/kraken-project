mod detection;
mod settings;

use std::ops::ControlFlow;

pub use self::settings::TcpServiceDetectionSettings;
use crate::modules::service_detection::tcp::detection::{
    find_all_protocols, find_exact_match, BreakReason,
};
use crate::modules::service_detection::{DynResult, Service};

/// Detect the service behind a socket by talking to it
pub async fn detect_tcp_service(settings: TcpServiceDetectionSettings) -> DynResult<Service> {
    match find_exact_match(&settings).await {
        ControlFlow::Continue(partial_matches) => {
            Ok(Service::Maybe(partial_matches.into_keys().collect()))
        }
        ControlFlow::Break(BreakReason::Found(service, protocol)) => {
            let protocols = find_all_protocols(&settings, service, protocol).await?; // TODO
            Ok(Service::Definitely(service))
        }
        ControlFlow::Break(BreakReason::Error(error)) => Err(error),
    }
}
