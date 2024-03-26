import React from "react";
import { FindingSeverity, ListFindings } from "../../../api/generated";
import { ROUTES } from "../../../routes";
import SeverityIcon from "../components/severity-icon";
import { WORKSPACE_CONTEXT } from "../workspace";

const SEVERITY_SORTING: { [k in FindingSeverity]: number } = {
    Okay: 0,
    Low: 1,
    Medium: 2,
    High: 3,
    Critical: 4,
};
export default function WorkspaceDataDetailsFindings({
    findings,
    ...props
}: { findings: ListFindings | null } & React.HTMLProps<HTMLDivElement>) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    return (
        <div className="workspace-data-details-relations-container" {...props}>
            <div className="workspace-data-details-relations-header workspace-data-details-findings">
                <div className="workspace-data-details-relations-heading">Severity</div>
                <div className="workspace-data-details-relations-heading">CVE</div>
                <div className="workspace-data-details-relations-heading">Name</div>
            </div>
            {findings ? (
                <div className="workspace-data-details-relations-body workspace-data-details-findings">
                    {findings.findings
                        .sort((a, b) => {
                            return SEVERITY_SORTING[b.severity] - SEVERITY_SORTING[a.severity];
                        })
                        .map((r) => (
                            <div
                                className="workspace-data-details-relations-entry"
                                {...ROUTES.WORKSPACE_FINDINGS_EDIT.clickHandler({ wUuid: workspace, fUuid: r.uuid })}
                            >
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
