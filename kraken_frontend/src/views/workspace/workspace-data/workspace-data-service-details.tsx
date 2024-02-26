import React, { useState } from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import { FullAggregationSource, FullService, ServiceRelations, TagType } from "../../../api/generated";
import Textarea from "../../../components/textarea";
import ArrowLeftIcon from "../../../svg/arrow-left";
import ArrowRightIcon from "../../../svg/arrow-right";
import { handleApiError } from "../../../utils/helper";
import EditableTags from "../components/editable-tags";
import { ServiceRelationsList } from "../components/relations-list";
import { WORKSPACE_CONTEXT } from "../workspace";
import { CertaintyIcon } from "../workspace-data";
import WorkspaceDataDetailsResults from "./workspace-data-details-results";

export type WorkspaceDataServiceDetailsProps = {
    service: string;
    updateService?: (uuid: string, update: Partial<FullService>) => void;
    tab: "general" | "results" | "relations" | "findings";
};

export function WorkspaceDataServiceDetails(props: WorkspaceDataServiceDetailsProps) {
    const { service: uuid, updateService: signalUpdate, tab: tab } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [attacks, setAttacks] = useState({} as FullAggregationSource);
    const [limit, setLimit] = useState(0);
    const [page, setPage] = useState(0);
    const [service, setService] = React.useState<FullService | null>(null);
    const [relations, setRelations] = React.useState<ServiceRelations | null>(null);
    React.useEffect(() => {
        Api.workspaces.services.get(workspace, uuid).then(handleApiError(setService));
        Api.workspaces.services.relations(workspace, uuid).then(handleApiError(setRelations));
        Api.workspaces.services.sources(workspace, uuid).then(
            handleApiError((x) => {
                setAttacks(x);
                setLimit(x.attacks.length - 1);
            }),
        );
    }, [workspace, uuid]);
    React.useEffect(() => {
        setPage(0);
    }, [uuid]);

    /** Send an update to the server and parent component */
    function update(uuid: string, update: Partial<FullService>, msg?: string) {
        const { tags, comment } = update;
        Api.workspaces.services
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

    if (service === null) return null;
    return (
        <>
            {tab === "general" ? (
                <>
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>Service</h3>
                        {`${service.name} running on ${service.host.ipAddr}`}
                        {!service.port ? "" : ` (Port ${service.port.port}, ${service.port.protocol})`}
                    </div>
                    <div className="workspace-data-details-pane">
                        <h3 className="sub-heading">Certainty</h3>
                        <div className="workspace-data-certainty-list">
                            {service.certainty === "Historical"
                                ? CertaintyIcon({ certaintyType: "Historical", nameVisible: true })
                                : service.certainty === "SupposedTo"
                                  ? CertaintyIcon({ certaintyType: "SupposedTo", nameVisible: true })
                                  : service.certainty === "UnknownService"
                                    ? CertaintyIcon({ certaintyType: "UnknownService", nameVisible: true })
                                    : service.certainty === "MaybeVerified"
                                      ? CertaintyIcon({ certaintyType: "MaybeVerified", nameVisible: true })
                                      : CertaintyIcon({ certaintyType: "DefinitelyVerified", nameVisible: true })}
                        </div>
                    </div>
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>Comment</h3>
                        <Textarea value={service.comment} onChange={(comment) => setService({ ...service, comment })} />
                        <button
                            className={"button"}
                            onClick={() =>
                                service && update(service.uuid, { comment: service.comment }, "Updated comment")
                            }
                        >
                            Update
                        </button>
                    </div>
                    <div className={"workspace-data-details-pane"}>
                        <h3 className={"sub-heading"}>Tags</h3>
                        <EditableTags
                            workspace={workspace}
                            tags={service.tags}
                            onChange={(tags) => {
                                setService((service) => service && { ...service, tags });
                                update(service.uuid, { tags });
                            }}
                        />
                    </div>
                </>
            ) : (
                <>
                    {tab === "results" ? (
                        <div className="workspace-data-details-flex">
                            <WorkspaceDataDetailsResults attack={attacks.attacks[page]} uuid={service.uuid} />
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
                        <>
                            {tab === "relations" ? (
                                <div className="workspace-data-details-overflow">
                                    <ServiceRelationsList relations={relations} />
                                </div>
                            ) : (
                                <>
                                    {tab === "findings" ? (
                                        <div className="workspace-data-details-overflow">
                                            <div className="workspace-data-details-relations-container">
                                                <div className="workspace-data-details-relations-header workspace-data-details-findings">
                                                    <div className="workspace-data-details-relations-heading">
                                                        Severity
                                                    </div>
                                                    <div className="workspace-data-details-relations-heading">CVE</div>
                                                    <div className="workspace-data-details-relations-heading">Name</div>
                                                </div>
                                                <div className="workspace-data-details-relations-body"></div>
                                            </div>
                                        </div>
                                    ) : (
                                        <div>Unimplemented</div>
                                    )}
                                </>
                            )}
                        </>
                    )}
                </>
            )}
        </>
    );
}
