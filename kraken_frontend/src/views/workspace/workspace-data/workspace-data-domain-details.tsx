import { Api } from "../../../api/api";
import React from "react";
import { FullDomain, FullHost, TagType } from "../../../api/generated";
import { handleApiError } from "../../../utils/helper";
import Textarea from "../../../components/textarea";
import { toast } from "react-toastify";
import EditableTags from "../components/editable-tags";

export type WorkspaceDataDomainDetailsProps = {
    workspace: string;
    domain: string;
    updateDomain?: (uuid: string, update: Partial<FullDomain>) => void;
};

export function WorkspaceDataDomainDetails(props: WorkspaceDataDomainDetailsProps) {
    const { workspace, domain: uuid, updateDomain: signalUpdate } = props;

    const [domain, setDomain] = React.useState<FullDomain | null>(null);
    React.useEffect(() => {
        Api.workspaces.domains.get(workspace, uuid).then(handleApiError(setDomain));
    }, [workspace, uuid]);

    function update(uuid: string, update: Partial<FullDomain>, msg?: string) {
        const { tags, comment } = update;
        Api.workspaces.domains
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

    if (domain === null) return null;
    return (
        <>
            <div className={"pane"}>{`Domain: ${domain.domain}`}</div>
            <div className={"workspace-data-details-comment pane"}>
                Comment
                <Textarea value={domain.comment} onChange={(comment) => setDomain({ ...domain, comment })} />
                <button
                    className={"button"}
                    onClick={() => domain && update(domain.uuid, { comment: domain.comment }, "Updated comment")}
                >
                    Update
                </button>
            </div>
            <div className={"pane"}>
                Tags
                <EditableTags
                    workspace={workspace}
                    tags={domain.tags}
                    onChange={(tags) => {
                        setDomain((host) => host && { ...host, tags });
                        update(domain.uuid, { tags });
                    }}
                />
            </div>
        </>
    );
}
