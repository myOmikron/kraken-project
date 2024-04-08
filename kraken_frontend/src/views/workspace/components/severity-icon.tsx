import React, { useCallback } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FindingSeverity, ListFindings } from "../../../api/generated";
import "../../../index.css";
import { handleApiError } from "../../../utils/helper";
import WorkspaceDataDetailsFindings from "../workspace-data/workspace-data-details-findings";

type SeverityIconProps = {
    severity: FindingSeverity | null | undefined;
    className?: string;
    tooltip?: boolean;
};

export default function SeverityIcon(props: SeverityIconProps) {
    const { className, severity, tooltip } = props;
    if (!severity) return null;
    const index =
        severity === FindingSeverity.Okay
            ? 0
            : severity === FindingSeverity.Low
              ? 1
              : severity === FindingSeverity.Medium
                ? 2
                : severity === FindingSeverity.High
                  ? 3
                  : severity === FindingSeverity.Critical
                    ? 4
                    : 5;
    const severities = [
        "severity-icon-ok",
        "severity-icon-low",
        "severity-icon-medium",
        "severity-icon-high",
        "severity-icon-critical",
        "neon",
    ] as const;
    return (
        <Popup
            trigger={
                <div className={className ?? "icon"}>
                    <svg
                        width="800px"
                        height="800px"
                        className={"severity-icon " + severities[index]}
                        viewBox="0 0 24 24"
                        stroke="none"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        {[...Array(4)].map((_, i) => (
                            <rect
                                key={i}
                                x={4}
                                y={19 - i * 5}
                                rx={0.5}
                                width={16}
                                height={1}
                                strokeWidth={1}
                                className={`${index > i ? "filled" : ""}`}
                                strokeLinecap="round"
                            />
                        ))}
                    </svg>
                </div>
            }
            position={"bottom center"}
            on={tooltip === undefined || tooltip ? ["hover"] : []}
            arrow={true}
        >
            <div className="pane-thin">
                <span>
                    Severity: <b>{severity}</b>
                </span>
            </div>
        </Popup>
    );
}

type SeverityProps = {
    severity: FindingSeverity | null | undefined;
    dataType: "Domain" | "Host" | "Port" | "Service" | "HttpService";
    uuid: string;
    workspace: string;
};

export function Severity(props: SeverityProps) {
    const { severity, dataType, uuid, workspace } = props;
    const [findings, setFindings] = React.useState<ListFindings | null>(null);

    const ensureDataLoaded = useCallback(() => {
        if (findings !== null) return;

        (async function () {
            switch (dataType) {
                case "Domain":
                    return Api.workspaces.domains.findings(workspace, uuid).then(handleApiError(setFindings));
                case "Host":
                    return Api.workspaces.hosts.findings(workspace, uuid).then(handleApiError(setFindings));
                case "Port":
                    return Api.workspaces.ports.findings(workspace, uuid).then(handleApiError(setFindings));
                case "Service":
                    return Api.workspaces.services.findings(workspace, uuid).then(handleApiError(setFindings));
                case "HttpService":
                    return Api.workspaces.httpServices.findings(workspace, uuid).then(handleApiError(setFindings));
            }
        })();
    }, [workspace, uuid, findings, setFindings]);

    return (
        <Popup
            on={["hover", "focus"]}
            position={"right center"}
            arrow
            trigger={
                // eagerly load on mouse over, so popup potentially doesn't need to wait
                <div onMouseOver={ensureDataLoaded} className={"workspace-data-certainty-icon"}>
                    <SeverityIcon severity={severity} tooltip={false} />
                </div>
            }
            onOpen={ensureDataLoaded}
            keepTooltipInside
        >
            <WorkspaceDataDetailsFindings
                className="workspace-data-details-relations-container pane-thin zero-padding-popup"
                findings={findings}
            />
        </Popup>
    );
}
