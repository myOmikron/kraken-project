import React, { useCallback } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { AggregationType, FindingSeverity, ListFindings } from "../../../api/generated";
import "../../../index.css";
import { handleApiError } from "../../../utils/helper";
import WorkspaceDataDetailsFindings from "../workspace-data/workspace-data-details-findings";

/**
 * Props for the <SeverityIcon> component
 */
type SeverityIconProps = {
    /** The severity to choose the icon based on. */
    severity: FindingSeverity | null | undefined;
    /** Optional CSS class name to use instead of "icon" */
    className?: string;
    /** When true, adds a popup on hover that shows the severity as human readable string */
    tooltip?: boolean;
};

/**
 * An icon showing 0-5 bars in various colors depending on the passed in severity.
 */
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

/**
 * Props for the <Severity> component.
 */
type SeverityProps = {
    /** The severity object to display */
    severity: FindingSeverity | null | undefined;
    /** The data type to fetch findings for on hover */
    dataType: AggregationType;
    /** The UUID of the object (of the dataType type) to fetch findings for on hover */
    uuid: string;
    /** the workspace to use for fetching findings on hover */
    workspace: string;
};

/**
 * Shows a severity icon that shows a popup on hover where the severity comes from,
 * e.g. shows all the attached findings.
 */
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
