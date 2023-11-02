import { Api } from "../../../api/api";
import React from "react";
import WorkspaceTable from "../components/workspace-table";
import { FullHost, FullService, SimpleService } from "../../../api/generated";

export type WorkspaceDataServicesProps = {
    workspace: string;
    onSelect: (uuid: string) => void;
    host: FullHost | null;
};

export function WorkspaceHostServices(props: WorkspaceDataServicesProps) {
    const { workspace, onSelect, host } = props;
    return (
        <WorkspaceTable<FullService>
            query={(limit, offset) => Api.workspaces.services.all(workspace, limit, offset, { host: host?.uuid })}
            queryDeps={[workspace, host?.uuid]}
            columnsTemplate={"1fr 5ch 1fr"}
        >
            <div className={"workspace-table-header"}>
                <span>Name</span>
                <span>Port</span>
                <span>Comment</span>
            </div>
            {(service) => (
                <div className={"workspace-table-row"} onClick={() => onSelect(service.uuid)}>
                    <span>{service.name}</span>
                    <span>{service.port?.port}</span>
                    <span>{service.comment}</span>
                </div>
            )}
        </WorkspaceTable>
    );
}
