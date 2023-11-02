import { Api } from "../../../api/api";
import React from "react";
import WorkspaceTable from "../components/workspace-table";
import { FullHost, SimpleDomain } from "../../../api/generated";

export type WorkspaceDataDomainsProps = {
    workspace: string;
    onSelect: (uuid: string) => void;
    host: FullHost | null;
};

export function WorkspaceHostDomains(props: WorkspaceDataDomainsProps) {
    const { workspace, onSelect, host } = props;
    return (
        <WorkspaceTable<SimpleDomain>
            query={(limit, offset) => Api.workspaces.domains.all(workspace, limit, offset, { host: host?.uuid })}
            queryDeps={[workspace, host?.uuid]}
            columnsTemplate={"1fr 1fr"}
            type={"Host"}
        >
            <div className={"workspace-table-header"}>
                <span>Name</span>
                <span>Comment</span>
            </div>
            {(domain) => (
                <div className={"workspace-table-row"} onClick={() => onSelect(domain.uuid)}>
                    <span>{domain.domain}</span>
                    <span>{domain.comment}</span>
                </div>
            )}
        </WorkspaceTable>
    );
}
