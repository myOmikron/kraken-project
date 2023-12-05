import { Api } from "../../../api/api";
import React from "react";
import { FullDomain, FullHost, TagType } from "../../../api/generated";
import { handleApiError } from "../../../utils/helper";
import Textarea from "../../../components/textarea";
import { toast } from "react-toastify";
import EditableTags from "../components/editable-tags";
import { WORKSPACE_CONTEXT } from "../workspace";

export type WorkspaceDataDomainDetailsProps = {
    domain: string;
    updateDomain?: (uuid: string, update: Partial<FullDomain>) => void;
    tab: "general" | "results" | "relations";
};

export function WorkspaceDataDomainDetails(props: WorkspaceDataDomainDetailsProps) {
    const { domain: uuid, updateDomain: signalUpdate, tab: tab } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

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
                })
            );
    }

    if (domain === null) return null;
    return (
        <>
            {tab === "general" ? (
                <>
                    <div className={"pane"}>
                        <h3 className={"sub-heading"}>Domain</h3>
                        {domain.domain}
                    </div>
                    <div className={"pane"}>
                        <h3 className={"sub-heading"}>Comment</h3>
                        <Textarea value={domain.comment} onChange={(comment) => setDomain({ ...domain, comment })} />
                        <button
                            className={"button"}
                            onClick={() =>
                                domain && update(domain.uuid, { comment: domain.comment }, "Updated comment")
                            }
                        >
                            Update
                        </button>
                    </div>
                    <div className={"pane"}>
                        <h3 className={"sub-heading"}>Tags</h3>
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
            ) : (
                <>{tab === "results" ? <div> domain results</div> : <div> domain relations</div>}</>
            )}
        </>
    );
}
