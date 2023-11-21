import { Api } from "../../../api/api";
import React from "react";
import { FullHost, TagType } from "../../../api/generated";
import { handleApiError } from "../../../utils/helper";
import Textarea from "../../../components/textarea";
import { toast } from "react-toastify";
import EditableTags from "../components/editable-tags";
import { WORKSPACE_CONTEXT } from "../workspace";

export type WorkspaceDataHostDetailsProps = {
    host: string;
    updateHost?: (uuid: string, update: Partial<FullHost>) => void;
};

export function WorkspaceDataHostDetails(props: WorkspaceDataHostDetailsProps) {
    const { host: uuid, updateHost: signalUpdate } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

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
            <div className={"pane"}>
                <h3 className={"sub-heading"}>Host</h3>
                {host.ipAddr}
            </div>
            <div className={"pane"}>
                <h3 className={"sub-heading"}>Comment</h3>
                <Textarea value={host.comment} onChange={(comment) => setHost({ ...host, comment })} />
                <button
                    className={"button"}
                    onClick={() => host && update(host.uuid, { comment: host.comment }, "Updated comment")}
                >
                    Update
                </button>
            </div>
            <div className={"pane"}>
                <h3 className={"sub-heading"}>Tags</h3>
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
