import React, { useState } from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import { FullAggregationSource, ListFindings, TagType } from "../../../api/generated";
import { FullHttpService } from "../../../api/generated/models/FullHttpService";
import { HttpServiceRelations } from "../../../api/generated/models/HttpServiceRelations";
import SelectableText from "../../../components/selectable-text";
import Textarea from "../../../components/textarea";
import { handleApiError } from "../../../utils/helper";
import CertaintyIcon from "../components/certainty-icon";
import EditableTags from "../components/editable-tags";
import { HttpServiceRelationsList } from "../components/relations-list";
import SeverityIcon from "../components/severity-icon";
import { WORKSPACE_CONTEXT } from "../workspace";
import WorkspaceDataDetailsFindings from "./workspace-data-details-findings";
import WorkspaceDataDetailsResults from "./workspace-data-details-results";

export type WorkspaceDataHttpServiceDetailsProps = {
    httpService: string;
    updateHttpService?: (uuid: string, update: Partial<FullHttpService>) => void;
    tab: "general" | "results" | "relations" | "findings";
};

export function WorkspaceDataHttpServiceDetails(props: WorkspaceDataHttpServiceDetailsProps) {
    const { httpService: uuid, updateHttpService: signalUpdate, tab: tab } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [attacks, setAttacks] = useState({} as FullAggregationSource);
    const [httpService, setHttpService] = React.useState<FullHttpService | null>(null);
    const [relations, setRelations] = React.useState<HttpServiceRelations | null>(null);
    const [findings, setFindings] = React.useState<ListFindings | null>(null);
    React.useEffect(() => {
        Api.workspaces.httpServices.get(workspace, uuid).then(handleApiError(setHttpService));
        Api.workspaces.httpServices.relations(workspace, uuid).then(handleApiError(setRelations));
        Api.workspaces.httpServices.findings(workspace, uuid).then(handleApiError(setFindings));
        Api.workspaces.httpServices.sources(workspace, uuid).then(handleApiError(setAttacks));
    }, [workspace, uuid]);

    /** Send an update to the server and parent component */
    function update(uuid: string, update: Partial<FullHttpService>, msg?: string) {
        const { tags, comment } = update;
        Api.workspaces.httpServices
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

    if (httpService === null) return null;
    switch (tab) {
        case "general":
            return (
                <>
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>HTTP Service</h3>
                        {`${httpService.name} running on ${httpService.host.ipAddr}:${httpService.port.port}`}
                        {!httpService.domain ? "" : ` on ${httpService.domain.domain}`}
                    </div>
                    <div className="workspace-data-details-pane">
                        <h3 className="sub-heading">Certainty</h3>
                        <div className="workspace-data-certainty-list">
                            <CertaintyIcon certainty={"UnknownService"} />
                        </div>
                    </div>
                    {httpService.severity && (
                        <div className="workspace-data-details-pane">
                            <h3 className="sub-heading">Severity</h3>
                            <div className="workspace-data-certainty-list">
                                <SeverityIcon
                                    tooltip={false}
                                    className={"icon workspace-data-certainty-icon"}
                                    severity={httpService.severity}
                                />
                                {httpService.severity}
                            </div>
                        </div>
                    )}
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>Comment</h3>
                        <Textarea
                            value={httpService.comment}
                            onChange={(comment) => setHttpService({ ...httpService, comment })}
                        />
                        <button
                            className={"button"}
                            onClick={() =>
                                httpService &&
                                update(httpService.uuid, { comment: httpService.comment }, "Updated comment")
                            }
                        >
                            Update
                        </button>
                    </div>
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>Tags</h3>
                        <EditableTags
                            workspace={workspace}
                            tags={httpService.tags}
                            onChange={(tags) => {
                                setHttpService((service) => service && { ...service, tags });
                                update(httpService.uuid, { tags });
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
                    <HttpServiceRelationsList relations={relations} />
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
