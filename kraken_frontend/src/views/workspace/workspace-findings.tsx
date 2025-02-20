import React from "react";
import { Api } from "../../api/api";
import { ROUTES } from "../../routes";
import "../../styling/tabs.css";
import "../../styling/workspace-findings.css";
import GraphIcon from "../../svg/graph";
import TableIcon from "../../svg/table";
import { handleApiError } from "../../utils/helper";
import { WORKSPACE_CONTEXT } from "./workspace";
import { DynamicTreeGraph } from "./workspace-finding/workspace-finding-dynamic-tree";
import WorkspaceFindingTable from "./workspace-finding/workspace-finding-table";

export type WorkspaceFindingsProps = {
    view: "table" | "graph";
};

export default function WorkspaceFindings(props: WorkspaceFindingsProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [roots, setRoots] = React.useState<string[]>([]);

    React.useEffect(() => {
        Api.workspaces.findings.all(workspace).then(
            handleApiError(({ findings }): void => {
                setRoots(findings.map((f) => f.uuid));
            }),
        );
    }, [workspace]);

    const body = (() => {
        switch (props.view) {
            case "table":
                return (
                    <WorkspaceFindingTable
                        sortingWeights
                        onClickRow={(f) =>
                            ROUTES.WORKSPACE_FINDINGS_EDIT.visit({
                                wUuid: workspace,
                                fUuid: f.uuid,
                            })
                        }
                        onAuxClickRow={(f) =>
                            ROUTES.WORKSPACE_FINDINGS_EDIT.open({
                                wUuid: workspace,
                                fUuid: f.uuid,
                            })
                        }
                    />
                );
            case "graph":
                return <DynamicTreeGraph maximizable workspace={workspace} uuids={roots} />;
        }
    })();

    return (
        <div className="workspace-findings-layout">
            <div className="tabs-selector-container">
                <div
                    className={`icon-tabs ${props.view === "table" ? "selected-icon-tab" : ""}`}
                    {...ROUTES.WORKSPACE_FINDINGS_LIST.clickHandler({ uuid: workspace })}
                >
                    <TableIcon />
                </div>
                <div
                    className={`icon-tabs ${props.view === "graph" ? "selected-icon-tab" : ""}`}
                    {...ROUTES.WORKSPACE_FINDINGS_GRAPH.clickHandler({ uuid: workspace })}
                >
                    <GraphIcon />
                </div>
            </div>
            <div className="pane workspace-findings-body">{body}</div>
        </div>
    );
}
