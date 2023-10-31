import { Api } from "../../../api/api";
import React from "react";
import { FullHost, TagType } from "../../../api/generated";
import { handleApiError } from "../../../utils/helper";
import Textarea from "../../../components/textarea";
import { toast } from "react-toastify";
import EditableTags from "../components/editable-tags";

export type WorkspaceDataHostDetailsProps = {
    workspace: string;
    host: string;
    updateHost?: (uuid: string, update: Partial<FullHost>) => void;
};

export function WorkspaceDataHostDetails(props: WorkspaceDataHostDetailsProps) {
    const { workspace, host: uuid, updateHost: signalUpdate } = props;

    const [host, setHost] = React.useState<FullHost | null>(null);
    React.useEffect(() => {
        Api.workspaces.hosts.get(workspace, uuid).then(handleApiError(setHost));
    }, [workspace, uuid]);

    /** Send an update to the server and parent component */
    function update(uuid: string, update: Partial<FullHost>, msg?: string) {
        const { tags, comment } = update;
        Api.workspaces.hosts
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

    if (host === null) return null;
    return (
        <>
            <div className={"pane"}>{`Host: ${host.ipAddr}`}</div>
            <div className={"workspace-data-details-comment pane"}>
                Comment
                <Textarea value={host.comment} onChange={(comment) => setHost({ ...host, comment })} />
                <button
                    className={"button"}
                    onClick={() => host && update(host.uuid, { comment: host.comment }, "Updated comment")}
                >
                    Update
                </button>
            </div>
            <div className={"pane"}>
                Tags
                <EditableTags
                    workspace={workspace}
                    tags={host.tags}
                    onChange={(tags) => {
                        setHost((host) => host && { ...host, tags });
                        update(host.uuid, { tags });
                    }}
                />
            </div>
        </>
    );
}
