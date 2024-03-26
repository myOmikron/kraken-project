import React, { useState } from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import { FullAggregationSource, FullHost, HostRelations, ListFindings, TagType } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import Textarea from "../../../components/textarea";
import "../../../styling/workspace-data-details.css";
import { handleApiError } from "../../../utils/helper";
import CertaintyIcon from "../components/certainty-icon";
import EditableTags from "../components/editable-tags";
import { HostRelationsList } from "../components/relations-list";
import SeverityIcon from "../components/severity-icon";
import { WORKSPACE_CONTEXT } from "../workspace";
import WorkspaceDataDetailsFindings from "./workspace-data-details-findings";
import WorkspaceDataDetailsResults from "./workspace-data-details-results";

export type WorkspaceDataHostDetailsProps = {
    host: string;
    updateHost?: (uuid: string, update: Partial<FullHost>) => void;
    tab: "general" | "results" | "relations" | "findings";
};

export function WorkspaceDataHostDetails(props: WorkspaceDataHostDetailsProps) {
    const { host: uuid, updateHost: signalUpdate, tab: tab } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [attacks, setAttacks] = useState({} as FullAggregationSource);
    const [host, setHost] = React.useState<FullHost | null>(null);
    const [relations, setRelations] = React.useState<HostRelations | null>(null);
    const [findings, setFindings] = React.useState<ListFindings | null>(null);
    React.useEffect(() => {
        Api.workspaces.hosts.get(workspace, uuid).then(handleApiError(setHost));
        Api.workspaces.hosts.relations(workspace, uuid).then(handleApiError(setRelations));
        Api.workspaces.hosts.findings(workspace, uuid).then(handleApiError(setFindings));
        Api.workspaces.hosts.sources(workspace, uuid).then(handleApiError(setAttacks));
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
    switch (tab) {
        case "general":
            return (
                <>
                    <div className="workspace-data-details-pane">
                        <h3 className={"sub-heading"}>Host</h3>
                        {host.ipAddr}
                    </div>
                    <div className="workspace-data-details-pane">
                        <h3 className="sub-heading">Certainty</h3>
                        <div className="workspace-data-certainty-list">
                            <CertaintyIcon certainty={host.certainty} />
                        </div>
                    </div>
                    {host.severity && (
                        <div className="workspace-data-details-pane">
                            <h3 className="sub-heading">Severity</h3>
                            <div className="workspace-data-certainty-list">
                                <SeverityIcon
                                    tooltip={false}
                                    className={"icon workspace-data-certainty-icon"}
                                    severity={host.severity}
                                />
                                {host.severity}
                            </div>
                        </div>
                    )}
                    <div className="workspace-data-details-pane">
                        <h3 className={"sub-heading"}>Comment</h3>
                        <Textarea value={host.comment} onChange={(comment) => setHost({ ...host, comment })} />
                        <button
                            className={"button"}
                            onClick={() => host && update(host.uuid, { comment: host.comment }, "Updated comment")}
                        >
                            Update
                        </button>
                    </div>
                    <div className="workspace-data-details-pane">
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
                    <HostRelationsList relations={relations} />
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
