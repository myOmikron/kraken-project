import React from "react";
import { Api } from "../../../api/api";
import { FullHost, FullService } from "../../../api/generated";
import SourcesList from "../components/sources-list";
import TagList from "../components/tag-list";
import WorkspaceTable from "../components/workspace-table";
import { WORKSPACE_CONTEXT } from "../workspace";

export type WorkspaceDataServicesProps = {
    onSelect: (uuid: string) => void;
    host: FullHost | null;
};

export function WorkspaceHostServices(props: WorkspaceDataServicesProps) {
    const { onSelect, host } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    return (
        <WorkspaceTable<FullService>
            workspace={workspace}
            query={(limit, offset) => Api.workspaces.services.all(workspace, limit, offset, { host: host?.uuid })}
            queryDeps={[workspace, host?.uuid]}
            columnsTemplate={"1fr 5ch 1fr 1fr 1fr"}
        >
            <div className={"workspace-table-header"}>
                <span>Service</span>
                <span>Port</span>
                <span>Tags</span>
                <span>Comment</span>
                <span>Attacks</span>
            </div>
            {(service) => (
                <div className={"workspace-table-row"} onClick={() => onSelect(service.uuid)}>
                    <span>{service.name}</span>
                    <span>{service.port?.port}</span>
                    <TagList tags={service.tags} />
                    <span>{service.comment}</span>
                    <SourcesList sources={service.sources} />
                </div>
            )}
        </WorkspaceTable>
    );
}
