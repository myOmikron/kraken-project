import React from "react";
import { ROUTES } from "../../../routes";
import ArrowLeftIcon from "../../../svg/arrow-left";
import { WORKSPACE_CONTEXT } from "../workspace";
import { DynamicTreeGraph } from "./workspace-finding-dynamic-tree";
import WorkspaceFindingTable from "./workspace-finding-table";

type WorkspaceEditFindingProps = {
    /** The finding's uuid */
    uuid: string;
};

export default function WorkspaceEditFinding(props: WorkspaceEditFindingProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    return (
        <div className="workspace-findings-selection-container">
            <div className="workspace-findings-selection-info pane">
                <ArrowLeftIcon title={"Back"} {...ROUTES.WORKSPACE_FINDINGS.clickHandler({ uuid: workspace })} />
            </div>
            <DynamicTreeGraph workspace={workspace} uuid={props.uuid} initialZoom={-0.25} maximizable />
            <WorkspaceFindingTable />
        </div>
    );
}
