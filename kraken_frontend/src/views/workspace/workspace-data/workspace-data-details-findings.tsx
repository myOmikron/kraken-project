import { ListFindings } from "../../../api/generated";
import SeverityIcon from "../../../svg/severity";

export default function WorkspaceDataDetailsFindings({ findings }: { findings: ListFindings | null }) {
    return (
        <div className="workspace-data-details-relations-container">
            <div className="workspace-data-details-relations-header workspace-data-details-findings">
                <div className="workspace-data-details-relations-heading">Severity</div>
                <div className="workspace-data-details-relations-heading">CVE</div>
                <div className="workspace-data-details-relations-heading">Name</div>
            </div>
            {findings ? (
                <div className="workspace-data-details-relations-body workspace-data-details-findings">
                    {findings.findings.map((r) => (
                        <div className="workspace-data-details-relations-entry">
                            <SeverityIcon severity={r.severity} />
                            <span>{r.cve}</span>
                            <span>{r.name}</span>
                        </div>
                    ))}
                </div>
            ) : (
                <p>Loading...</p>
            )}
        </div>
    );
}
