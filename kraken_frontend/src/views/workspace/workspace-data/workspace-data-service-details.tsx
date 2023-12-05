import { Api } from "../../../api/api";
import React from "react";
import { FullHost, FullService, TagType } from "../../../api/generated";
import { handleApiError } from "../../../utils/helper";
import Textarea from "../../../components/textarea";
import { toast } from "react-toastify";
import EditableTags from "../components/editable-tags";
import { WORKSPACE_CONTEXT } from "../workspace";

export type WorkspaceDataServiceDetailsProps = {
    service: string;
    updateService?: (uuid: string, update: Partial<FullService>) => void;
    tab: "general" | "results" | "relations";
};

export function WorkspaceDataServiceDetails(props: WorkspaceDataServiceDetailsProps) {
    const { service: uuid, updateService: signalUpdate, tab: tab } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [service, setService] = React.useState<FullService | null>(null);
    React.useEffect(() => {
        Api.workspaces.services.get(workspace, uuid).then(handleApiError(setService));
    }, [workspace, uuid]);

    /** Send an update to the server and parent component */
    function update(uuid: string, update: Partial<FullService>, msg?: string) {
        const { tags, comment } = update;
        Api.workspaces.services
            .update(workspace, uuid, {
                comment,
                workspaceTags:
                    tags && tags.filter(({ tagType }) => tagType === TagType.Workspace).map(({ uuid }) => uuid),
                globalTags: tags && tags.filter(({ tagType }) => tagType === TagType.Global).map(({ uuid }) => uuid),
            })
            .then(
                handleApiError(() => {
                    if (msg !== undefined) toast.success(msg);
                    if (signalUpdate !== undefined) signalUpdate(uuid, update);
                })
            );
    }

    if (service === null) return null;
    return (
        <>
            {tab === "general" ? (
                <>
                    <div className={"pane"}>
                        <h3 className={"sub-heading"}>Service</h3>
                        {`${service.name} running on ${service.host.ipAddr}`}
                        {!service.port ? "" : ` (Port ${service.port.port})`}
                    </div>
                    <div className={"pane"}>
                        <h3 className={"sub-heading"}>Comment</h3>
                        <Textarea value={service.comment} onChange={(comment) => setService({ ...service, comment })} />
                        <button
                            className={"button"}
                            onClick={() =>
                                service && update(service.uuid, { comment: service.comment }, "Updated comment")
                            }
                        >
                            Update
                        </button>
                    </div>
                    <div className={"pane"}>
                        <h3 className={"sub-heading"}>Tags</h3>
                        <EditableTags
                            workspace={workspace}
                            tags={service.tags}
                            onChange={(tags) => {
                                setService((service) => service && { ...service, tags });
                                update(service.uuid, { tags });
                            }}
                        />
                    </div>
                </>
            ) : (
                <>{tab === "results" ? <div> service results</div> : <div> service relations</div>}</>
            )}
        </>
    );
}
