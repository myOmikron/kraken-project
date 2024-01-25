import { Api } from "../../../api/api";
import React, { useState } from "react";
import { FullAggregationSource, FullPort, PortRelations, TagType } from "../../../api/generated";
import { handleApiError } from "../../../utils/helper";
import Textarea from "../../../components/textarea";
import { toast } from "react-toastify";
import EditableTags from "../components/editable-tags";
import { WORKSPACE_CONTEXT } from "../workspace";
import WorkspaceDataDetailsResults from "./workspace-data-details-results";
import ArrowLeftIcon from "../../../svg/arrow-left";
import ArrowRightIcon from "../../../svg/arrow-right";
import RelationRightIcon from "../../../svg/relation-right";
import RelationLeftIcon from "../../../svg/relation-left";
import VerifiedIcon from "../../../svg/verified";
import HistoricalIcon from "../../../svg/historical";
import Popup from "reactjs-popup";
import { CertaintyIcon } from "../workspace-data";

export type WorkspaceDataPortDetailsProps = {
    port: string;
    updatePort?: (uuid: string, update: Partial<FullPort>) => void;
    tab: "general" | "results" | "relations";
};

export function WorkspaceDataPortDetails(props: WorkspaceDataPortDetailsProps) {
    const { port: uuid, updatePort: signalUpdate, tab: tab } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [attacks, setAttacks] = useState({} as FullAggregationSource);
    const [limit, setLimit] = useState(0);
    const [page, setPage] = useState(0);
    const [port, setPort] = React.useState<FullPort | null>(null);
    const [relations, setRelations] = React.useState<PortRelations | null>(null);
    React.useEffect(() => {
        Api.workspaces.ports.get(workspace, uuid).then(handleApiError(setPort));
        Api.workspaces.ports.relations(workspace, uuid).then(handleApiError(setRelations));
        Api.workspaces.ports.sources(workspace, uuid).then(
            handleApiError((x) => {
                setAttacks(x);
                setLimit(x.attacks.length - 1);
            })
        );
    }, [workspace, uuid]);
    React.useEffect(() => {
        setPage(0);
    }, [uuid]);

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
                })
            );
    }

    if (port === null) return null;
    return (
        <>
            {tab === "general" ? (
                <>
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>Port</h3>
                        {`${port.port} open on ${port.host.ipAddr}`}
                    </div>
                    <div className="workspace-data-details-pane">
                        <h3 className="sub-heading">Certainty</h3>
                        <div className="workspace-data-certainty-list">
                            {port.certainty === "Verified"
                                ? CertaintyIcon({ certaintyType: "Verified", nameVisible: true })
                                : port.certainty === "Historical"
                                ? CertaintyIcon({ certaintyType: "Historical", nameVisible: true })
                                : CertaintyIcon({ certaintyType: "SupposedTo", nameVisible: true })}
                        </div>
                    </div>
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
                </>
            ) : (
                <>
                    {tab === "results" ? (
                        <div className="workspace-data-details-flex">
                            <WorkspaceDataDetailsResults attack={attacks.attacks[page]} uuid={port.uuid} />
                            <div className="workspace-data-details-table-controls">
                                <div className="workspace-data-details-controls-container">
                                    <button
                                        className={"workspace-table-button"}
                                        disabled={page === 0}
                                        onClick={() => {
                                            setPage(page - 1);
                                        }}
                                    >
                                        <ArrowLeftIcon />
                                    </button>
                                    <div className="workspace-table-controls-page-container">
                                        <span>
                                            {page + 1} of {limit + 1}
                                        </span>
                                    </div>
                                    <button
                                        className={"workspace-table-button"}
                                        disabled={page === limit}
                                        onClick={() => {
                                            setPage(page + 1);
                                        }}
                                    >
                                        <ArrowRightIcon />
                                    </button>
                                </div>
                            </div>
                        </div>
                    ) : (
                        <div className="workspace-data-details-overflow">
                            <div className="workspace-data-details-relations-container">
                                <div className="workspace-data-details-relations-header">
                                    <div className="workspace-data-details-relations-heading">Connection</div>
                                    <div className="workspace-data-details-relations-heading">Type</div>
                                    <div className="workspace-data-details-relations-heading">To</div>
                                </div>
                                <div className="workspace-data-details-relations-body">
                                    {relations?.host !== null && relations?.host !== undefined ? (
                                        <>
                                            <div title={"Direct"}>
                                                <RelationLeftIcon />
                                            </div>
                                            <span>Host</span>
                                            <span>{relations.host.ipAddr} </span>
                                        </>
                                    ) : undefined}
                                    {relations?.services.map((s) => {
                                        return (
                                            <>
                                                <div title={"Direct"}>
                                                    <RelationRightIcon />
                                                </div>
                                                <span>Service</span>
                                                <span>{s.name} </span>
                                            </>
                                        );
                                    })}
                                </div>
                            </div>
                        </div>
                    )}
                </>
            )}
        </>
    );
}
