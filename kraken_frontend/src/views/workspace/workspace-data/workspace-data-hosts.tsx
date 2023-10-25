import { Api } from "../../../api/api";
import React from "react";
import { FullHost, SimpleDomain, SimpleHost } from "../../../api/generated";
import WorkspaceTable from "../components/workspace-table";
import { handleApiError } from "../../../utils/helper";

export type WorkspaceDataHostsProps = {
    workspace: string;
    onSelect: (uuid: string) => void;
};

export function WorkspaceDataHosts(props: WorkspaceDataHostsProps) {
    const { workspace, onSelect } = props;
    return (
        <WorkspaceTable<FullHost>
            query={(limit, offset) => Api.workspaces.hosts.all(workspace, limit, offset)}
            queryDeps={[workspace]}
            columns={2}
        >
            <div className={"workspace-data-table-header"}>
                <span>IP</span>
                <span>Comment</span>
            </div>
            {(host) => (
                <div className={"workspace-data-table-row"} onClick={() => onSelect(host.uuid)}>
                    <span>{host.ipAddr}</span>
                    <span>{host.comment}</span>
                </div>
            )}
        </WorkspaceTable>
    );
}
