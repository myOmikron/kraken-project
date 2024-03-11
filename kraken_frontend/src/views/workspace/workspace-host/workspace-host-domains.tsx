import React from "react";
import { Api } from "../../../api/api";
import { FullDomain, FullHost } from "../../../api/generated";
import SourcesList from "../components/sources-list";
import TagList from "../components/tag-list";
import WorkspaceTable from "../components/workspace-table";
import { WORKSPACE_CONTEXT } from "../workspace";

export type WorkspaceDataDomainsProps = {
    onSelect: (uuid: string) => void;
    host: FullHost | null;
};

export function WorkspaceHostDomains(props: WorkspaceDataDomainsProps) {
    const { onSelect, host } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    return (
        <WorkspaceTable<FullDomain>
            workspace={workspace}
            query={(limit, offset) => Api.workspaces.domains.all(workspace, limit, offset, { host: host?.uuid })}
            queryDeps={[workspace, host?.uuid]}
            columnsTemplate={"1fr 1fr 1fr 1fr"}
        >
            <div className={"workspace-table-header"}>
                <span>Domain</span>
                <span>Tags</span>
                <span>Comment</span>
                <span>Attacks</span>
            </div>
            {(domain) => (
                <div className={"workspace-table-row"} onClick={() => onSelect(domain.uuid)}>
                    <span>{domain.domain}</span>
                    <TagList tags={domain.tags} />
                    <span>{domain.comment}</span>
                    <SourcesList sources={domain.sources} />
                </div>
            )}
        </WorkspaceTable>
    );
}
