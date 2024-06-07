import React, { useEffect } from "react";
import { toast } from "react-toastify";
import { Api, UUID } from "../../api/api";
import { AggregationType, FullDomain, FullHost, FullPort, FullService, ListFindings } from "../../api/generated";
import { FullHttpService } from "../../api/generated/models/FullHttpService";
import { handleApiError } from "../../utils/helper";
import { WORKSPACE_CONTEXT } from "./workspace";
import { CreateFindingAffected, LocalAffected } from "./workspace-finding/workspace-create-finding";
import WorkspaceFindingTable from "./workspace-finding/workspace-finding-table";

export type WorkspaceFindingsQuickAttachProps = {
    type: AggregationType;
    onAttached?: (finding: UUID, wantMore: boolean) => void;
} & ({ uuid: UUID } | { data: FullDomain | FullHost | FullService | FullPort | FullHttpService });

export default function WorkspaceFindingsQuickAttach(props: WorkspaceFindingsQuickAttachProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [data, setData] = React.useState<FullDomain | FullHost | FullService | FullPort | FullHttpService>();
    const [details, setDetails] = React.useState<string>("");
    const [findings, setFindings] = React.useState<ListFindings>();

    const uuid = "data" in props ? props.data.uuid : props.uuid;

    useEffect(() => {
        const hasData = "data" in props;
        if (hasData && data != props.data) {
            setData(props.data);
        }
        switch (props.type) {
            case "Domain":
                if (!hasData) Api.workspaces.domains.get(workspace, uuid).then(handleApiError(setData));
                Api.workspaces.domains.findings(workspace, uuid).then(handleApiError(setFindings));
                break;
            case "Host":
                if (!hasData) Api.workspaces.hosts.get(workspace, uuid).then(handleApiError(setData));
                Api.workspaces.hosts.findings(workspace, uuid).then(handleApiError(setFindings));
                break;
            case "Port":
                if (!hasData) Api.workspaces.ports.get(workspace, uuid).then(handleApiError(setData));
                Api.workspaces.ports.findings(workspace, uuid).then(handleApiError(setFindings));
                break;
            case "Service":
                if (!hasData) Api.workspaces.services.get(workspace, uuid).then(handleApiError(setData));
                Api.workspaces.services.findings(workspace, uuid).then(handleApiError(setFindings));
                break;
            case "HttpService":
                if (!hasData) Api.workspaces.httpServices.get(workspace, uuid).then(handleApiError(setData));
                Api.workspaces.httpServices.findings(workspace, uuid).then(handleApiError(setFindings));
                break;
            default:
                toast.error("Invalid data type");
                return;
        }
    }, [workspace, props.type, "data" in props ? props.data : props.uuid]);

    return (
        <div className="workspace-findings-layout">
            {data ? (
                <div className="selected-affected">
                    <span>Attaching affected</span>
                    <CreateFindingAffected
                        affected={
                            {
                                uuid: uuid,
                                userDetails: details,
                                exportDetails: "",
                                type: props.type,
                                _data: data,
                            } as LocalAffected
                        }
                        onChangeDetails={setDetails}
                    />
                </div>
            ) : (
                <div>Attachment loading...</div>
            )}
            <div className="pane workspace-findings-body">
                <WorkspaceFindingTable
                    filter={(f) => (findings ? !findings.findings.map((a) => a.uuid).includes(f.uuid) : true)}
                    onClickRow={async (f, e) => {
                        toast.promise(
                            Api.workspaces.findings
                                .addAffected(workspace, f.uuid, {
                                    userDetails: details,
                                    exportDetails: "",
                                    type: props.type,
                                    uuid: uuid,
                                })
                                .then((v) => {
                                    v.unwrap();
                                    props.onAttached?.(f.uuid, e.shiftKey || e.ctrlKey);
                                    setFindings((attachedTo) => ({
                                        findings: [...(attachedTo?.findings ?? []), f],
                                    }));
                                }),
                            {
                                pending: "Attaching to finding",
                                success: "Attached to finding",
                                error: "Failed attaching to finding",
                            },
                        );
                    }}
                />
            </div>
        </div>
    );
}
