import { Api } from "../../../api/api";
import React from "react";
import WorkspaceTable from "../components/workspace-table";
import { FullHost, FullPort, SimplePort } from "../../../api/generated";
import SourcesList from "../components/sources-list";

type WorkspaceDataPortsProps = {
    workspace: string;
    onSelect: (uuid: string) => void;
    host: FullHost | null;
};

export function WorkspaceHostPorts(props: WorkspaceDataPortsProps) {
    const { workspace, onSelect, host } = props;
    return (
        <WorkspaceTable<FullPort>
            query={(limit, offset) => Api.workspaces.ports.all(workspace, limit, offset, { host: host?.uuid })}
            queryDeps={[workspace, host?.uuid]}
            columnsTemplate={"5ch 1fr 1fr"}
        >
            <div className={"workspace-table-header"}>
                <span>Port</span>
                <span>Comment</span>
                <span>Attacks</span>
            </div>
            {(port) => (
                <div className={"workspace-table-row"} onClick={() => onSelect(port.uuid)}>
                    <span>{port.port}</span>
                    <span>{port.comment}</span>
                    <SourcesList sources={port.sources} />
                </div>
            )}
        </WorkspaceTable>
    );
}
