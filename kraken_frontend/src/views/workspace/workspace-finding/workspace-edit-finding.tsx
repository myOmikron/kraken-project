import { editor } from "monaco-editor";
import React, { useEffect } from "react";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api, UUID } from "../../../api/api";
import {
    AggregationType,
    FindingAffectedObject,
    FindingAffectedObjectOneOf,
    FindingAffectedObjectOneOf1,
    FindingAffectedObjectOneOf2,
    FindingAffectedObjectOneOf3,
    FindingSeverity,
    FullFindingAffected,
    SimpleDomain,
    SimpleFindingDefinition,
    SimpleHost,
    SimplePort,
    SimpleService,
    SimpleTag,
    UpdateFindingRequest,
} from "../../../api/generated";
import WS from "../../../api/websocket";
import { GithubMarkdown } from "../../../components/github-markdown";
import ModelEditor from "../../../components/model-editor";
import { SelectPrimitive } from "../../../components/select-menu";
import { ROUTES } from "../../../routes";
import ArrowLeftIcon from "../../../svg/arrow-left";
import BookIcon from "../../../svg/book";
import CloseIcon from "../../../svg/close";
import EditIcon from "../../../svg/edit";
import FileIcon from "../../../svg/file";
import GraphIcon from "../../../svg/graph";
import InformationIcon from "../../../svg/information";
import PersonCircleIcon from "../../../svg/person-circle";
import PlusIcon from "../../../svg/plus";
import RelationLeftRightIcon from "../../../svg/relation-left-right";
import ScreenshotIcon from "../../../svg/screenshot";
import { compareDomain, compareHost, comparePort, compareService } from "../../../utils/data-sorter";
import { ObjectFns, handleApiError } from "../../../utils/helper";
import { useModel, useModelStore } from "../../../utils/model-controller";
import { useSyncedCursors } from "../../../utils/monaco-cursor";
import CollapsibleSection from "../components/collapsible-section";
import Domain from "../components/domain";
import { UploadingFileInput } from "../components/file-input";
import IpAddr from "../components/host";
import PortNumber from "../components/port";
import SelectFindingDefinition from "../components/select-finding-definition";
import ServiceName from "../components/service";
import TagList from "../components/tag-list";
import { WORKSPACE_CONTEXT } from "../workspace";
import { FindingDefinitionDetails } from "./workspace-create-finding";
import WorkspaceFindingDataTable, { WorkspaceFindingDataTableRef } from "./workspace-finding-data-table";
import EditingTreeGraph, { EditingTreeGraphRef } from "./workspace-finding-editing-tree";
import ITextModel = editor.ITextModel;

export type WorkspaceEditFindingProps = {
    /** The finding's uuid */
    uuid: string;
};

type Section = "definition" | "description" | "affected" | "network";

export default function WorkspaceEditFinding(props: WorkspaceEditFindingProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const { uuid: finding } = props;

    const [section, setSection] = React.useState<Section>("definition");

    const [severity, setSeverity] = React.useState<FindingSeverity>("Medium");
    const [findingDef, setFindingDef] = React.useState<SimpleFindingDefinition>();
    const [hoveredFindingDef, setHoveredFindingDef] = React.useState<SimpleFindingDefinition>();
    const [userDetails, setUserDetails, userDetailsModel] = useModel({ language: "markdown" });
    const [toolDetails, setToolDetails] = React.useState("");
    const [logFile, setLogFile] = React.useState("");
    const [screenshot, setScreenshot] = React.useState("");

    const [affected, setAffected] = React.useState<Record<UUID, Omit<FullFindingAffected, "userDetails">>>({});
    const affectedModels = useModelStore();

    const dataTableRef = React.useRef<WorkspaceFindingDataTableRef>(null);
    const graphRef = React.useRef<EditingTreeGraphRef>(null);

    const onClickTag = (e: { ctrlKey: boolean; shiftKey: boolean; altKey: boolean }, tag: SimpleTag) => {
        dataTableRef.current?.addFilterColumn("tag", tag.name, e.altKey);
        graphRef.current?.addTag(tag, e.altKey);
    };

    // Upload to API with changes
    const [pendingApiChanges, setPendingApiChanges] = React.useState<
        UpdateFindingRequest & { _onSuccess: Array<() => void>; _onFailure: Array<() => void> }
    >();
    const updateFinding = (changes: UpdateFindingRequest, rollback: () => void, success?: () => void) => {
        setPendingApiChanges((c) => ({
            ...c,
            ...changes,
            _onSuccess: success ? [...(c?._onSuccess ?? []), success] : c?._onSuccess ?? [],
            _onFailure: [...(c?._onFailure ?? []), rollback],
        }));
    };
    useEffect(() => {
        // TODO: debounce
        if (pendingApiChanges !== undefined) {
            const c = pendingApiChanges;
            Api.workspaces.findings.update(workspace, finding, c).then((v) => {
                v.match(
                    () => c._onSuccess.forEach((v) => v()),
                    () => {
                        toast.error("Failed to update finding");
                        c._onFailure.forEach((v) => v());
                    },
                );
            });
            setPendingApiChanges(undefined);
        }
    }, [pendingApiChanges]);

    React.useEffect(() => {
        // Get initial state
        Api.workspaces.findings.get(workspace, finding).then(
            handleApiError((fullFinding) => {
                setFindingDef(fullFinding.definition);
                setSeverity(fullFinding.severity);
                setUserDetails(fullFinding.userDetails || "", { finding: { finding: finding } });
                setToolDetails(fullFinding.toolDetails || "");
                setScreenshot(fullFinding.screenshot || "");
                setLogFile(fullFinding.logFile || "");

                Promise.all(
                    fullFinding.affected.map((simpleAffected) =>
                        Api.workspaces.findings
                            .getAffected(workspace, finding, simpleAffected.affectedUuid)
                            .then((fullAffected) => [simpleAffected.affectedUuid, fullAffected.unwrap()] as const),
                    ),
                )
                    .then((affected) =>
                        affected.map(([uuid, { userDetails, ...fullAffected }]) => {
                            affectedModels.addModel(uuid, {
                                value: userDetails,
                                language: "markdown",
                                syncTarget: { findingAffected: { finding: finding, affected: uuid } },
                            });
                            return [uuid, fullAffected];
                        }),
                    )
                    .then(Object.fromEntries)
                    .then(setAffected)
                    .catch(() => toast.error("Failed to read affected data"));
            }),
        );

        // Listen on state updates
        const handles = [
            WS.addEventListener("message.UpdatedFinding", ({ workspace: w, finding: f, update }) => {
                if (w !== workspace || f !== finding) return;
                const { severity, definition, screenshot, logFile } = update;
                if (severity) {
                    setSeverity(severity);
                }
                if (definition) {
                    Api.knowledgeBase.findingDefinitions.get(definition).then(handleApiError(setFindingDef));
                }
                if (screenshot) {
                    setScreenshot(screenshot);
                }
                if (logFile) {
                    setLogFile(logFile);
                }
            }),
            WS.addEventListener("message.AddedFindingAffected", ({ workspace: w, finding: f, affectedUuid }) => {
                if (w !== workspace || f !== finding) return;
                Api.workspaces.findings.getAffected(workspace, finding, affectedUuid).then(
                    handleApiError(({ userDetails, ...newAffected }) => {
                        setAffected((affected) => ({
                            ...affected,
                            [affectedUuid]: newAffected,
                        }));
                        affectedModels.addModel(affectedUuid, {
                            value: userDetails,
                            language: "markdown",
                            syncTarget: { findingAffected: { finding, affected: affectedUuid } },
                        });
                    }),
                );
            }),
            WS.addEventListener(
                "message.UpdatedFindingAffected",
                ({ workspace: w, finding: f, affectedUuid, update }) => {
                    if (w !== workspace || f !== finding) return;
                    setAffected(({ [affectedUuid]: affected, ...rest }) => ({
                        [affectedUuid]: { ...affected, ...update },
                        ...rest,
                    }));
                },
            ),
            WS.addEventListener("message.RemovedFindingAffected", ({ workspace: w, finding: f, affectedUuid }) => {
                if (w !== workspace || f !== finding) return;
                setAffected(({ [affectedUuid]: _, ...rest }) => rest);
                affectedModels.removeModel(affectedUuid);
            }),
        ];
        return () => {
            affectedModels.removeAll();
            for (const handle of handles) {
                WS.removeEventListener(handle);
            }
        };
    }, [workspace, finding]);

    const { cursors: editorCursors, setEditor } = useSyncedCursors({
        target: { finding: { finding } },
        receiveCursor: (target) => {
            if ("finding" in target && target.finding.finding === finding) {
                return true;
            }
        },
    });

    return (
        <div className="pane">
            <div className="workspace-findings-selection-info">
                <ArrowLeftIcon title={"Back"} {...ROUTES.WORKSPACE_FINDINGS_LIST.clickHandler({ uuid: workspace })} />
                <h1 className="heading">Edit finding</h1>
            </div>
            <div className="create-finding-container">
                <div className="create-finding-form">
                    <div className="create-finding-header">
                        <h2 className={"sub-heading"}>Severity</h2>
                        <h2 className={"sub-heading"}>
                            <InformationIcon /> Finding Definition
                        </h2>

                        <SelectPrimitive
                            value={hoveredFindingDef?.severity || severity}
                            options={Object.values(FindingSeverity)}
                            onChange={(value) => {
                                if (value) {
                                    setSeverity(value);
                                    Api.workspaces.findings
                                        .update(workspace, finding, { severity: value })
                                        .then(handleApiError);
                                }
                            }}
                        />
                        <SelectFindingDefinition
                            selected={findingDef?.uuid}
                            onSelect={(newDef) => {
                                setFindingDef(newDef);
                                setSeverity(newDef.severity);
                                Api.workspaces.findings
                                    .update(workspace, finding, { definition: newDef.uuid })
                                    .then(handleApiError);
                            }}
                            onHover={setHoveredFindingDef}
                        />
                    </div>

                    <div className="scrollable">
                        <div className="create-finding-files">
                            <h2 className={"sub-heading"}>
                                <ScreenshotIcon />
                                Screenshot
                            </h2>
                            <h2 className={"sub-heading"}>
                                <FileIcon />
                                Log File
                            </h2>
                            <UploadingFileInput
                                image
                                file={screenshot}
                                onChange={(newImage) => {
                                    setScreenshot((oldScreenshot) => {
                                        updateFinding({ screenshot: newImage }, () => {
                                            setScreenshot(oldScreenshot);
                                        });
                                        return newImage ?? "";
                                    });
                                }}
                            />
                            <UploadingFileInput
                                file={logFile}
                                onChange={(newFile) => {
                                    setLogFile((oldFile) => {
                                        updateFinding({ logFile: newFile }, () => {
                                            setLogFile(oldFile);
                                        });
                                        return newFile ?? "";
                                    });
                                }}
                            />
                        </div>

                        <CollapsibleSection
                            summary={
                                <>
                                    <BookIcon />
                                    User Details
                                </>
                            }
                        >
                            <GithubMarkdown>{userDetails}</GithubMarkdown>
                        </CollapsibleSection>

                        <CollapsibleSection
                            summary={
                                <>
                                    <BookIcon />
                                    Tool Details
                                </>
                            }
                        >
                            <GithubMarkdown>{toolDetails}</GithubMarkdown>
                        </CollapsibleSection>

                        <CollapsibleSection
                            summary={
                                <>
                                    <RelationLeftRightIcon />
                                    Affected
                                </>
                            }
                        >
                            <div className="affected-list">
                                {ObjectFns.isEmpty(affected) ? (
                                    <p>No affected items yet</p>
                                ) : (
                                    Object.entries(affected)
                                        .sort(([_1, a], [_2, b]) => {
                                            const aType = getAffectedType(a);
                                            const bType = getAffectedType(b);
                                            if (aType < bType) return -1;
                                            if (aType > bType) return 1;
                                            const aObj = getAffectedData(a);
                                            const bObj = getAffectedData(b);
                                            switch (aType) {
                                                case "Domain":
                                                    return compareDomain(aObj as SimpleDomain, bObj as SimpleDomain);
                                                case "Host":
                                                    return compareHost(aObj as SimpleHost, bObj as SimpleHost);
                                                case "Port":
                                                    // Sorting here doesn't compare host / relational data yet.
                                                    // To fix this we would need FullPort/FullService, which would
                                                    // need tons of more requests right now. Not going to do this until
                                                    // it really becomes a problem - it should not be a major problem.
                                                    return comparePort(aObj as SimplePort, bObj as SimplePort);
                                                case "Service":
                                                    return compareService(aObj as SimpleService, bObj as SimpleService);
                                                default:
                                                    return 0;
                                            }
                                        })
                                        .map(([affectedUuid, fullAffected]) => (
                                            <div
                                                key={affectedUuid}
                                                className={`create-finding-affected affected affected-${getAffectedType(fullAffected)}`}
                                            >
                                                <div className="name">
                                                    <div
                                                        title={"Remove affected"}
                                                        className="remove"
                                                        onClick={() => {
                                                            Api.workspaces.findings
                                                                .removeAffected(workspace, finding, affectedUuid)
                                                                .then(
                                                                    handleApiError(() => {
                                                                        setAffected(
                                                                            ({ [affectedUuid]: _, ...rest }) => rest,
                                                                        );
                                                                        affectedModels.removeModel(affectedUuid);
                                                                    }),
                                                                );
                                                        }}
                                                    >
                                                        <CloseIcon />
                                                    </div>
                                                    <AffectedLabel affected={fullAffected.affected} pretty />
                                                </div>
                                                <MarkdownLiveEditorPopup
                                                    label={<AffectedLabel affected={fullAffected.affected} pretty />}
                                                    value={affectedModels.models[affectedUuid].value}
                                                    model={affectedModels.models[affectedUuid].model}
                                                />
                                                <TagList tags={fullAffected.affectedTags} onClickTag={onClickTag} />
                                                <UploadingFileInput
                                                    image
                                                    shortText
                                                    className="screenshot"
                                                    file={fullAffected.screenshot ?? undefined}
                                                    onChange={(newImage) => {
                                                        Api.workspaces.findings
                                                            .updateAffected(workspace, finding, affectedUuid, {
                                                                screenshot: newImage,
                                                            })
                                                            .then(
                                                                handleApiError(() =>
                                                                    setAffected(
                                                                        ({ [affectedUuid]: affected, ...rest }) => ({
                                                                            [affectedUuid]: {
                                                                                ...affected,
                                                                                screenshot: newImage,
                                                                            },
                                                                            ...rest,
                                                                        }),
                                                                    ),
                                                                ),
                                                            );
                                                    }}
                                                >
                                                    <ScreenshotIcon />
                                                </UploadingFileInput>
                                                <UploadingFileInput
                                                    shortText
                                                    className="logfile"
                                                    file={fullAffected.logFile ?? undefined}
                                                    onChange={(newFile) => {
                                                        Api.workspaces.findings
                                                            .updateAffected(workspace, finding, affectedUuid, {
                                                                logFile: newFile,
                                                            })
                                                            .then(
                                                                handleApiError(() =>
                                                                    setAffected(
                                                                        ({ [affectedUuid]: affected, ...rest }) => ({
                                                                            [affectedUuid]: {
                                                                                ...affected,
                                                                                logFile: newFile,
                                                                            },
                                                                            ...rest,
                                                                        }),
                                                                    ),
                                                                ),
                                                            );
                                                    }}
                                                >
                                                    <FileIcon />
                                                </UploadingFileInput>
                                            </div>
                                        ))
                                )}
                            </div>
                        </CollapsibleSection>
                        <DeleteButton workspace={workspace} finding={finding} severity={severity} />
                    </div>
                </div>
                <div className="create-finding-editor-container">
                    <div className="knowledge-base-editor-tabs">
                        <button
                            title={"Finding Definition"}
                            className={`knowledge-base-editor-tab ${section === "definition" ? "selected" : ""}`}
                            onClick={() => {
                                setSection("definition");
                            }}
                        >
                            <InformationIcon />
                        </button>
                        <button
                            title={"Description"}
                            className={`knowledge-base-editor-tab ${section === "description" ? "selected" : ""}`}
                            onClick={() => {
                                setSection("description");
                            }}
                        >
                            <BookIcon />
                            {editorCursors.length > 0 ? <PersonCircleIcon /> : null}
                        </button>
                        <button
                            title={"Affected"}
                            className={`knowledge-base-editor-tab ${section === "affected" ? "selected" : ""}`}
                            onClick={() => {
                                setSection("affected");
                            }}
                        >
                            <RelationLeftRightIcon />
                        </button>
                        <button
                            title={"Network"}
                            className={`knowledge-base-editor-tab ${section === "network" ? "selected" : ""}`}
                            onClick={() => {
                                setSection("network");
                            }}
                        >
                            <GraphIcon />
                        </button>
                    </div>
                    {(() => {
                        switch (section) {
                            case "definition":
                                return <FindingDefinitionDetails definition={hoveredFindingDef || findingDef} />;
                            case "description":
                                return (
                                    <>
                                        <ModelEditor model={userDetailsModel} setEditor={setEditor} />
                                        {editorCursors.map(({ data: { displayName }, cursor }) =>
                                            cursor.render(<div className={"cursor-label"}>{displayName}</div>),
                                        )}
                                    </>
                                );
                            case "affected":
                                const addAffected = (uuid: UUID, type: AggregationType) =>
                                    Api.workspaces.findings
                                        .addAffected(workspace, finding, {
                                            type,
                                            uuid,
                                            details: "",
                                        })
                                        .then(
                                            handleApiError(() =>
                                                Api.workspaces.findings.getAffected(workspace, finding, uuid).then(
                                                    handleApiError(({ userDetails, ...fullAffected }) => {
                                                        setAffected((affected) => ({
                                                            ...affected,
                                                            [uuid]: fullAffected,
                                                        }));
                                                        affectedModels.addModel(uuid, {
                                                            value: userDetails || "",
                                                            language: "markdown",
                                                            syncTarget: {
                                                                findingAffected: {
                                                                    finding,
                                                                    affected: uuid,
                                                                },
                                                            },
                                                        });
                                                    }),
                                                ),
                                            ),
                                        );
                                return (
                                    <div className="workspace-finding-data-table">
                                        <WorkspaceFindingDataTable
                                            ref={dataTableRef}
                                            hideUuids={Object.keys(affected)}
                                            onAddDomains={(v) =>
                                                v.map(({ uuid }) => addAffected(uuid, AggregationType.Domain))
                                            }
                                            onAddHosts={(v) =>
                                                v.map(({ uuid }) => addAffected(uuid, AggregationType.Host))
                                            }
                                            onAddPorts={(v) =>
                                                v.map(({ uuid }) => addAffected(uuid, AggregationType.Port))
                                            }
                                            onAddServices={(v) =>
                                                v.map(({ uuid }) => addAffected(uuid, AggregationType.Service))
                                            }
                                        />
                                    </div>
                                );
                            case "network":
                                return (
                                    <EditingTreeGraph
                                        ref={graphRef}
                                        uuid={finding}
                                        definition={findingDef}
                                        severity={severity}
                                        affected={Object.values(affected) as Array<FullFindingAffected>}
                                        workspace={workspace}
                                        maximizable
                                    />
                                );
                            default:
                                return "Unimplemented";
                        }
                    })()}
                </div>
            </div>
        </div>
    );
}

function DeleteButton({ workspace, finding, severity }: { workspace: UUID; finding: UUID; severity: FindingSeverity }) {
    const [open, setOpen] = React.useState(false);

    return (
        <Popup
            modal
            nested
            open={open}
            onClose={() => setOpen(false)}
            trigger={
                <div className="workspace-data-danger-pane">
                    <h2 className={"sub-heading"}>Danger Zone</h2>
                    <button
                        type="button"
                        onClick={() => setOpen(true)}
                        className="workspace-settings-red-button button"
                    >
                        Delete finding
                    </button>
                </div>
            }
        >
            <div className="popup-content pane danger " style={{ width: "50ch", backgroundColor: "rgba(30,0,0,0.25)" }}>
                <div className="workspace-setting-popup">
                    <h2 className="heading neon">Are you sure you want to delete this {severity} Severity finding?</h2>
                    <button
                        className="workspace-settings-red-button button"
                        type="reset"
                        onClick={() => setOpen(false)}
                    >
                        Cancel
                    </button>
                    <button
                        className="workspace-settings-red-button button"
                        type="button"
                        onClick={() => {
                            toast.promise(
                                Api.workspaces.findings
                                    .delete(workspace, finding)
                                    .then(() => ROUTES.WORKSPACE_FINDINGS_LIST.visit({ uuid: workspace })),
                                {
                                    pending: "Deleting finding...",
                                    error: "Failed to delete finding!",
                                    success: "Successfully deleted",
                                },
                            );
                        }}
                    >
                        Delete
                    </button>
                </div>
            </div>
        </Popup>
    );
}

type MarkdownLiveEditorPopupProps = {
    label: React.ReactNode;
    value: string;
    model: ITextModel | null;
};

export function MarkdownLiveEditorPopup(props: MarkdownLiveEditorPopupProps) {
    const { label, value, model } = props;
    return (
        <Popup
            className="markdown-editor-popup"
            trigger={
                <div className="details">
                    {value.length > 0 ? ["Edit User Details", <EditIcon />] : ["Add User Details", <PlusIcon />]}
                </div>
            }
            nested
            modal
            on={"click"}
        >
            <div className="pane">
                <div className="label">
                    <h1 className="sub-heading">Details</h1>
                    <h3 className="sub-heading">{label}</h3>
                </div>
                <div className="grid">
                    <GithubMarkdown>{value}</GithubMarkdown>
                    <ModelEditor model={model} />
                </div>
            </div>
        </Popup>
    );
}

export type AffectedLabelProps = {
    pretty?: boolean;
    affected: FindingAffectedObject;
};

export function AffectedLabel({ pretty, affected }: AffectedLabelProps) {
    if (isAffectedDomain(affected)) return <Domain domain={affected.domain} pretty={pretty} />;
    if (isAffectedHost(affected)) return <IpAddr host={affected.host} pretty={pretty} />;
    if (isAffectedPort(affected)) return <PortNumber port={affected.port} pretty={pretty} />;
    else return <ServiceName service={affected.service} pretty={pretty} />;
}

function isAffectedDomain(obj: FindingAffectedObject): obj is FindingAffectedObjectOneOf {
    return "domain" in obj && obj["domain"] !== undefined;
}

function isAffectedHost(obj: FindingAffectedObject): obj is FindingAffectedObjectOneOf1 {
    return "host" in obj && obj["host"] !== undefined;
}

function isAffectedPort(obj: FindingAffectedObject): obj is FindingAffectedObjectOneOf2 {
    return "port" in obj && obj["port"] !== undefined;
}

function isAffectedService(obj: FindingAffectedObject): obj is FindingAffectedObjectOneOf3 {
    return "service" in obj && obj["service"] !== undefined;
}

export function getAffectedType({ affected }: { affected: FindingAffectedObject }): AggregationType {
    if (isAffectedDomain(affected)) return AggregationType.Domain;
    if (isAffectedHost(affected)) return AggregationType.Host;
    if (isAffectedPort(affected)) return AggregationType.Port;
    if (isAffectedService(affected)) return AggregationType.Service;
    const _exhaustiveCheck: never = affected;
    throw new Error("unknown affected type?!");
}

export function getAffectedData({ affected }: { affected: FindingAffectedObject }) {
    if (isAffectedDomain(affected)) return affected.domain;
    if (isAffectedHost(affected)) return affected.host;
    if (isAffectedPort(affected)) return affected.port;
    if (isAffectedService(affected)) return affected.service;
    const _exhaustiveCheck: never = affected;
    throw new Error("unknown affected type?!");
}
