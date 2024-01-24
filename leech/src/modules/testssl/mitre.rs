use kraken_proto::mitre::Tactic;

use crate::modules::testssl::finding_id::FindingId;
use crate::modules::testssl::{Finding, Severity};

/// Categorize a finding into a tactic and technique from Mitre ATT&CK
pub fn categorize(finding: &Finding) -> Option<Tactic> {
    // Only categorize actual problems
    if matches!(
        &finding.severity,
        Severity::Debug | Severity::Info | Severity::Warn | Severity::Fatal | Severity::Ok
    ) {
        return None;
    }

    let _id = FindingId::from(finding.id.as_str());
    None // TODO
}
