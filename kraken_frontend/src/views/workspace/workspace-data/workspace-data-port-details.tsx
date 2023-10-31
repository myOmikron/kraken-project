import { Api } from "../../../api/api";
import React from "react";
import { FullHost, FullPort, TagType } from "../../../api/generated";
import { handleApiError } from "../../../utils/helper";
import Textarea from "../../../components/textarea";
import { toast } from "react-toastify";
import EditableTags from "../components/editable-tags";

export type WorkspaceDataPortDetailsProps = {
    workspace: string;
    port: string;
    updatePort?: (uuid: string, update: Partial<FullPort>) => void;
};

export function WorkspaceDataPortDetails(props: WorkspaceDataPortDetailsProps) {
    const { workspace, port: uuid, updatePort: signalUpdate } = props;

    const [port, setPort] = React.useState<FullPort | null>(null);
    React.useEffect(() => {
        Api.workspaces.ports.get(workspace, uuid).then(handleApiError(setPort));
    }, [workspace, uuid]);

    /** Send an update to the server and parent component */
    function update(uuid: string, update: Partial<FullPort>, msg?: string) {
        const { tags, comment } = update;
        Api.workspaces.ports
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
                }),
            );
    }

    if (port === null) return null;
    return (
        <>
            <div className={"pane"}>{`Port ${port.port} on ${port.host.ipAddr}`}</div>
            <div className={"workspace-data-details-comment pane"}>
                Comment
                <Textarea value={port.comment} onChange={(comment) => setPort({ ...port, comment })} />
                <button
                    className={"button"}
                    onClick={() => port && update(port.uuid, { comment: port.comment }, "Updated comment")}
                >
                    Update
                </button>
            </div>
            <div className={"pane"}>
                Tags
                <EditableTags
                    workspace={workspace}
                    tags={port.tags}
                    onChange={(tags) => {
                        setPort((port) => port && { ...port, tags });
                        update(port.uuid, { tags });
                    }}
                />
            </div>
        </>
    );
}
