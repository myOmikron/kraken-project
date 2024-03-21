import Editor from "@monaco-editor/react";
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
    SimpleFindingDefinition,
    UpdateFindingRequest,
} from "../../../api/generated";
import WS from "../../../api/websocket";
import { GithubMarkdown } from "../../../components/github-markdown";
import useLiveEditor from "../../../components/live-editor";
import { SelectPrimitive } from "../../../components/select-menu";
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
import { ObjectFns, handleApiError } from "../../../utils/helper";
import { setupMonaco } from "../../knowledge-base";
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
import EditingTreeGraph from "./workspace-finding-editing-tree";
import WorkspaceFindingTable from "./workspace-finding-table";

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
    const [userDetails, setUserDetails] = React.useState("");
    const [toolDetails, setToolDetails] = React.useState("");

    const [affected, setAffected] = React.useState<Record<UUID, FullFindingAffected>>({});
    const [logFile, setLogFile] = React.useState("");
    const [screenshot, setScreenshot] = React.useState("");

    // Upload to API with changes
    const [pendingApiChanges, setPendingApiChanges] = React.useState<
        UpdateFindingRequest & { _onSuccess: Function[]; _onFailure: Function[] }
    >();
    const updateFinding = (changes: UpdateFindingRequest, rollback: Function, success?: Function) => {
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
                setUserDetails(fullFinding.userDetails || "");
                setToolDetails(fullFinding.toolDetails || "");
                setScreenshot(fullFinding.screenshot || "");
                setLogFile(fullFinding.logFile || "");

                Promise.all(
                    fullFinding.affected.map((simpleAffected) =>
                        Api.workspaces.findings
                            .getAffected(workspace, finding, simpleAffected.affectedUuid)
                            .then((fullAffected) => [simpleAffected.affectedUuid, fullAffected.unwrap()]),
                    ),
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
                    handleApiError((newAffected) =>
                        setAffected((affected) => ({
                            ...affected,
                            [affectedUuid]: newAffected,
                        })),
                    ),
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
            }),
        ];
        return () => {
            for (const handle of handles) {
                WS.removeEventListener(handle);
            }
        };
    }, [workspace, finding]);

    const [editorInstance, setEditorInstance] = React.useState<editor.IStandaloneCodeEditor | null>(null);
    const { cursors: editorCursors, onChange: editorOnChange } = useLiveEditor({
        target: { finding: { finding } },
        editorInstance,
        setValue: setUserDetails,
        receiveCursor: (target) => {
            if ("finding" in target && target.finding.finding === finding) {
                return true;
            }
        },
        receiveEdit: (target, editorInstance) => {
            if ("finding" in target && target.finding.finding === finding) {
                const model = editorInstance?.getModel();
                if (model) return { model, setValue: setUserDetails };
            }
        },
    });

    return (
        <div className="pane">
            <div className="workspace-findings-selection-info">
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
                            value={severity}
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
                            onSelect={(value) => {
                                setFindingDef(value);
                                Api.workspaces.findings
                                    .update(workspace, finding, { definition: value.uuid })
                                    .then(handleApiError);
                            }}
                            hovered={hoveredFindingDef?.uuid}
                            onHover={setHoveredFindingDef}
                        />
                    </div>

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
                                    .sort(([aUuid, a], [bUuid, b]) => {
                                        let aType = getAffectedType(a);
                                        let bType = getAffectedType(b);
                                        if (aType < bType) return -1;
                                        if (aType > bType) return 1;
                                        // TODO: type-based sorters
                                        if (aUuid < bUuid) return -1;
                                        if (aUuid > bUuid) return 1;
                                        return 0;
                                    })
                                    .map(([affectedUuid, fullAffected]) => (
                                        <div
                                            key={affectedUuid}
                                            className={`affected affected-${getAffectedType(fullAffected)}`}
                                        >
                                            <div className="name">
                                                <div
                                                    title={"Remove affected"}
                                                    className="remove"
                                                    onClick={() => {
                                                        Api.workspaces.findings
                                                            .removeAffected(workspace, finding, affectedUuid)
                                                            .then(
                                                                handleApiError(() =>
                                                                    setAffected(
                                                                        ({ [affectedUuid]: _, ...rest }) => rest,
                                                                    ),
                                                                ),
                                                            );
                                                    }}
                                                >
                                                    <CloseIcon />
                                                </div>
                                                <AffectedLabel affected={fullAffected.affected} pretty />
                                            </div>
                                            <MarkdownLiveEditorPopup
                                                label={<AffectedLabel affected={fullAffected.affected} pretty />}
                                                value={fullAffected.userDetails}
                                                setValue={(userDetails) => {
                                                    setAffected(({ [affectedUuid]: affected, ...rest }) => ({
                                                        [affectedUuid]: { ...affected, userDetails },
                                                        ...rest,
                                                    }));
                                                }}
                                                findingUuid={finding}
                                                affectedUuid={affectedUuid}
                                            />
                                            <TagList tags={fullAffected.affectedTags} />
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
                                const effectiveDef = hoveredFindingDef || findingDef;
                                return effectiveDef && <FindingDefinitionDetails {...effectiveDef} />;
                            case "description":
                                return (
                                    <>
                                        <Editor
                                            className={"knowledge-base-editor"}
                                            theme={"custom"}
                                            beforeMount={setupMonaco}
                                            value={userDetails}
                                            onChange={editorOnChange}
                                            onMount={setEditorInstance}
                                        />
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
                                                    handleApiError((fullAffected) =>
                                                        setAffected((affected) => ({
                                                            ...affected,
                                                            [uuid]: fullAffected,
                                                        })),
                                                    ),
                                                ),
                                            ),
                                        );
                                return (
                                    <div className="workspace-finding-data-table">
                                        <WorkspaceFindingTable
                                            onAddDomain={({ uuid }) => addAffected(uuid, AggregationType.Domain)}
                                            onAddHost={({ uuid }) => addAffected(uuid, AggregationType.Host)}
                                            onAddPort={({ uuid }) => addAffected(uuid, AggregationType.Port)}
                                            onAddService={({ uuid }) => addAffected(uuid, AggregationType.Service)}
                                        />
                                    </div>
                                );
                            case "network":
                                return (
                                    <EditingTreeGraph
                                        uuid={finding}
                                        definition={findingDef}
                                        severity={severity}
                                        affected={Object.values(affected)}
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

type MarkdownLiveEditorPopupProps = {
    label: React.ReactNode;
    findingUuid: string;
    affectedUuid: string;
    value: string;
    setValue: (newValue: string) => void;
};

export function MarkdownLiveEditorPopup(props: MarkdownLiveEditorPopupProps) {
    const { label, value, setValue, findingUuid, affectedUuid } = props;

    const [editorInstance, setEditorInstance] = React.useState<editor.IStandaloneCodeEditor | null>(null);
    const { onChange } = useLiveEditor({
        target: { findingAffected: { finding: findingUuid, affected: affectedUuid } },
        editorInstance,
        setValue,
        receiveEdit: () => undefined,
        receiveCursor: () => undefined,
    });

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
                    <Editor
                        className={"knowledge-base-editor"}
                        theme={"custom"}
                        beforeMount={setupMonaco}
                        language={"markdown"}
                        value={value}
                        onChange={onChange}
                        onMount={setEditorInstance}
                    />
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

export function getAffectedType({ affected }: FullFindingAffected): AggregationType {
    if (isAffectedDomain(affected)) return AggregationType.Domain;
    if (isAffectedHost(affected)) return AggregationType.Host;
    if (isAffectedPort(affected)) return AggregationType.Port;
    else return AggregationType.Service;
}

export function getAffectedData({ affected }: FullFindingAffected) {
    if (isAffectedDomain(affected)) return affected.domain;
    if (isAffectedHost(affected)) return affected.host;
    if (isAffectedPort(affected)) return affected.port;
    else return affected.service;
}
