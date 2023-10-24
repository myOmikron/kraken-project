import { Api } from "../../../api/api";
import React from "react";
import WorkspaceDataTable from "./workspace-data-table";
import { SimpleDomain } from "../../../api/generated";

export type WorkspaceDataDomainsProps = {
    workspace: string;
    onSelect: (uuid: string) => void;
};

export function WorkspaceDataDomains(props: WorkspaceDataDomainsProps) {
    const { workspace, onSelect } = props;
    return (
        <WorkspaceDataTable<SimpleDomain>
            query={(limit, offset) => Api.workspaces.domains.all(workspace, limit, offset)}
            queryDeps={[workspace]}
        >
            <div className={"workspace-data-table-header pane"}>
                <span>Name</span>
                <span>Comment</span>
            </div>
            {(domain) => (
                <div className={"workspace-data-table-row pane"} onClick={() => onSelect(domain.uuid)}>
                    <span>{domain.domain}</span>
                    <span>{domain.comment}</span>
                </div>
            )}
        </WorkspaceDataTable>
    );
}
