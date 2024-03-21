import React, { useState } from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import { DomainRelations, FullAggregationSource, FullDomain, ListFindings, TagType } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import Textarea from "../../../components/textarea";
import { handleApiError } from "../../../utils/helper";
import EditableTags from "../components/editable-tags";
import { DomainRelationsList } from "../components/relations-list";
import { WORKSPACE_CONTEXT } from "../workspace";
import WorkspaceDataDetailsFindings from "./workspace-data-details-findings";
import WorkspaceDataDetailsResults from "./workspace-data-details-results";
import SeverityIcon from "../../../svg/severity";
import CertaintyIcon from "../components/certainty-icon";

export type WorkspaceDataDomainDetailsProps = {
    domain: string;
    updateDomain?: (uuid: string, update: Partial<FullDomain>) => void;
    tab: "general" | "results" | "relations" | "findings";
};

export function WorkspaceDataDomainDetails(props: WorkspaceDataDomainDetailsProps) {
    const { domain: uuid, updateDomain: signalUpdate, tab: tab } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [attacks, setAttacks] = useState({} as FullAggregationSource);
    const [domain, setDomain] = React.useState<FullDomain | null>(null);
    const [relations, setRelations] = React.useState<DomainRelations | null>(null);
    const [findings, setFindings] = React.useState<ListFindings | null>(null);
    React.useEffect(() => {
        Api.workspaces.domains.get(workspace, uuid).then(handleApiError(setDomain));
        Api.workspaces.domains.relations(workspace, uuid).then(handleApiError(setRelations));
        Api.workspaces.domains.findings(workspace, uuid).then(handleApiError(setFindings));
        Api.workspaces.domains.sources(workspace, uuid).then(handleApiError(setAttacks));
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
    switch (tab) {
        case "general":
            return (
                <>
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>Domain</h3>
                        {domain.domain}
                    </div>
                    <div className="workspace-data-details-pane">
                        <h3 className="sub-heading">Certainty</h3>
                        <div className="workspace-data-certainty-list">
                            <CertaintyIcon certainty={domain.certainty} />
                        </div>
                    </div>
                    {domain.severity && (
                        <div className="workspace-data-details-pane">
                            <h3 className="sub-heading">Severity</h3>
                            <div className="workspace-data-certainty-list">
                                <SeverityIcon
                                    className={"icon workspace-data-certainty-icon"}
                                    severity={domain.severity}
                                />
                                {domain.severity}
                            </div>
                        </div>
                    )}
                    <div className={"workspace-data-details-pane"}>
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
                    <div className={"workspace-data-details-pane"}>
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
                    <SelectableText className="uuid">{uuid}</SelectableText>
                </>
            );
        case "results":
            return (
                <div className="workspace-data-details-flex">
                    <WorkspaceDataDetailsResults attacks={attacks.attacks} />
                </div>
            );
        case "relations":
            return (
                <div className="workspace-data-details-overflow">
                    <DomainRelationsList relations={relations} />
                </div>
            );
        case "findings":
            return (
                <div className="workspace-data-details-overflow">
                    <WorkspaceDataDetailsFindings findings={findings} />
                </div>
            );
    }
}
