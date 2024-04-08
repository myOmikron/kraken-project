import React from "react";
import { Api } from "../../../api/api";
import { FullHost, FullHttpService } from "../../../api/generated";
import SourcesList from "../components/sources-list";
import TagList from "../components/tag-list";
import WorkspaceTable from "../components/workspace-table";
import { WORKSPACE_CONTEXT } from "../workspace";

export type WorkspaceDataServicesProps = {
    host: FullHost | null;
};

export function WorkspaceHostHttpServices(props: WorkspaceDataServicesProps) {
    const { host } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    return (
        <WorkspaceTable<FullHttpService>
            workspace={workspace}
            query={(filter, limit, offset) =>
                Api.workspaces.httpServices.all(workspace, limit, offset, { host: host?.uuid, globalFilter: filter })
            }
            queryDeps={[workspace, host?.uuid]}
            columnsTemplate={"1fr 1fr 5ch 1fr 1fr 1fr 1fr"}
        >
            <div className={"workspace-table-header"}>
                <span>HTTP Service</span>
                <span>Host</span>
                <span>Port</span>
                <span>Domain</span>
                <span>Path</span>
                <span>Tags</span>
                <span>Comment</span>
                <span>Attacks</span>
            </div>
            {(httpService) => (
                <div key={httpService.uuid} className={"workspace-table-row"}>
                    <span>{httpService.name}</span>
                    <span>{httpService.port.port}</span>
                    <span>{httpService.domain?.domain ?? ""}</span>
                    <span>{httpService.basePath}</span>
                    <TagList tags={httpService.tags} />
                    <span>{httpService.comment}</span>
                    <SourcesList sources={httpService.sources} />
                </div>
            )}
        </WorkspaceTable>
    );
}
