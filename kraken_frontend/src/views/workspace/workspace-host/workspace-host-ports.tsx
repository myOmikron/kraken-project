import React from "react";
import { Api } from "../../../api/api";
import { FullHost, FullPort } from "../../../api/generated";
import SourcesList from "../components/sources-list";
import TagList from "../components/tag-list";
import WorkspaceTable from "../components/workspace-table";
import { WORKSPACE_CONTEXT } from "../workspace";

/** React props for [`<WorkspaceHostPorts />`]{@link WorkspaceHostPorts} */
type WorkspaceDataPortsProps = {
    /**
     * Host of which details are shown
     */
    host: FullHost | null;
};

/**
 * Shows table with all the found ports from current host
 */
export function WorkspaceHostPorts(props: WorkspaceDataPortsProps) {
    const { host } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    return (
        <WorkspaceTable<FullPort>
            workspace={workspace}
            query={(filter, limit, offset) =>
                Api.workspaces.ports.all(workspace, limit, offset, { host: host?.uuid, globalFilter: filter })
            }
            queryDeps={[workspace, host?.uuid]}
            columnsTemplate={"5ch 1fr 1fr 1fr"}
        >
            <div className={"workspace-table-header"}>
                <span>Port</span>
                <span>Tags</span>
                <span>Comment</span>
                <span>Attacks</span>
            </div>
            {(port) => (
                <div key={port.uuid} className={"workspace-table-row"}>
                    <span>{port.port}</span>
                    <TagList tags={port.tags} />
                    <span>{port.comment}</span>
                    <SourcesList sources={port.sources} />
                </div>
            )}
        </WorkspaceTable>
    );
}
