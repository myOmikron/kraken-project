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
            columns={3}
            type={"Host"}
        >
            <div className={"workspace-data-table-header"}>
                <span>Name</span>
                <span>Port</span>
                <span>Comment</span>
            </div>
            {(service) => (
                <div className={"workspace-data-table-row"} onClick={() => onSelect(service.uuid)}>
                    <span>{service.name}</span>
                    <span>{service.port}</span>
                    <span>{service.comment}</span>
                </div>
            )}
        </WorkspaceTable>
    );
}
