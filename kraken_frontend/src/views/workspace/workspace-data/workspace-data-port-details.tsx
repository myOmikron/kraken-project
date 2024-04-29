import React, { useState } from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import { FullAggregationSource, FullPort, ListFindings, PortRelations, TagType } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import Textarea from "../../../components/textarea";
import { handleApiError } from "../../../utils/helper";
import CertaintyIcon from "../components/certainty-icon";
import EditableTags from "../components/editable-tags";
import { PortRelationsList } from "../components/relations-list";
import SeverityIcon from "../components/severity-icon";
import { WORKSPACE_CONTEXT } from "../workspace";
import WorkspaceDataDetailsFindings from "./workspace-data-details-findings";
import WorkspaceDataDetailsResults from "./workspace-data-details-results";

/** React props for [`<WorkspaceDataPortDetails />`]{@link WorkspaceDataPortDetails} */
export type WorkspaceDataPortDetailsProps = {
    /**
     * Port UUID
     */
    port: string;
    /**
     * Callback when Port data was edited
     */
    updatePort?: (uuid: string, update: Partial<FullPort>) => void;
    /**
     * The tab to render
     */
    tab: "general" | "results" | "relations" | "findings";
};

/**
 * Tall pane with background which shows detailed information for the Port
 */
export function WorkspaceDataPortDetails(props: WorkspaceDataPortDetailsProps) {
    const { port: uuid, updatePort: signalUpdate, tab: tab } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [attacks, setAttacks] = useState({} as FullAggregationSource);
    const [port, setPort] = React.useState<FullPort | null>(null);
    const [relations, setRelations] = React.useState<PortRelations | null>(null);
    const [findings, setFindings] = React.useState<ListFindings | null>(null);
    React.useEffect(() => {
        Api.workspaces.ports.get(workspace, uuid).then(handleApiError(setPort));
        Api.workspaces.ports.relations(workspace, uuid).then(handleApiError(setRelations));
        Api.workspaces.ports.findings(workspace, uuid).then(handleApiError(setFindings));
        Api.workspaces.ports.sources(workspace, uuid).then(handleApiError(setAttacks));
    }, [workspace, uuid]);

    /**
     * Send an update to the server and parent component
     *
     * @param uuid  Port UUID to update
     * @param update The properties to update on the Port
     * @param msg Message to show on success
     */
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
    switch (tab) {
        case "general":
            return (
                <>
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>Port</h3>
                        {`${port.port} open on ${port.host.ipAddr}`}
                    </div>
                    <div className="workspace-data-details-pane">
                        <h3 className="sub-heading">Certainty</h3>
                        <div className="workspace-data-certainty-list">
                            <CertaintyIcon certainty={port.certainty} />
                        </div>
                    </div>
                    {port.severity && (
                        <div className="workspace-data-details-pane">
                            <h3 className="sub-heading">Severity</h3>
                            <div className="workspace-data-certainty-list">
                                <SeverityIcon
                                    tooltip={false}
                                    className={"icon workspace-data-certainty-icon"}
                                    severity={port.severity}
                                />
                                {port.severity}
                            </div>
                        </div>
                    )}
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>Comment</h3>
                        <Textarea value={port.comment} onChange={(comment) => setPort({ ...port, comment })} />
                        <button
                            className={"button"}
                            onClick={() => port && update(port.uuid, { comment: port.comment }, "Updated comment")}
                        >
                            Update
                        </button>
                    </div>
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>Tags</h3>
                        <EditableTags
                            workspace={workspace}
                            tags={port.tags}
                            onChange={(tags) => {
                                setPort((port) => port && { ...port, tags });
                                update(port.uuid, { tags });
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
                    <PortRelationsList relations={relations} />
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
