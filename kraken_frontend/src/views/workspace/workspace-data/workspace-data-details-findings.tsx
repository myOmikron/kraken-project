import React from "react";
import { ListFindings } from "../../../api/generated";
import { ROUTES } from "../../../routes";
import SeverityIcon from "../components/severity-icon";
import { WORKSPACE_CONTEXT } from "../workspace";

export default function WorkspaceDataDetailsFindings({ findings }: { findings: ListFindings | null }) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
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
