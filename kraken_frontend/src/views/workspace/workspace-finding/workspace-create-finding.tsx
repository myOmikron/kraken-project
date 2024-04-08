import Editor from "@monaco-editor/react";
import React from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import {
    AggregationType,
    CreateFindingAffectedRequest,
    FindingSeverity,
    FullDomain,
    FullHost,
    FullPort,
    FullService,
    SimpleFindingCategory,
    SimpleFindingDefinition,
    SimpleTag,
} from "../../../api/generated";
import { FullHttpService } from "../../../api/generated/models/FullHttpService";
import FindingCategoryList from "../../../components/finding-category-list";
import { GithubMarkdown } from "../../../components/github-markdown";
import { SelectPrimitive } from "../../../components/select-menu";
import { ROUTES } from "../../../routes";
import ArrowLeftIcon from "../../../svg/arrow-left";
import BookIcon from "../../../svg/book";
import CloseIcon from "../../../svg/close";
import FileIcon from "../../../svg/file";
import GraphIcon from "../../../svg/graph";
import InformationIcon from "../../../svg/information";
import RelationLeftRightIcon from "../../../svg/relation-left-right";
import ScreenshotIcon from "../../../svg/screenshot";
import {
    aggregationTypeOrdering,
    compareDomain,
    compareHost,
    compareHttpService,
    comparePort,
    compareService,
} from "../../../utils/data-sorter";
import { handleApiError } from "../../../utils/helper";
import { configureMonaco } from "../../../utils/monaco";
import CollapsibleSection from "../components/collapsible-section";
import Domain from "../components/domain";
import EditableCategories from "../components/editable-categories";
import { FileInput } from "../components/file-input";
import IpAddr from "../components/host";
import HttpServiceName from "../components/http-service";
import MarkdownEditorPopup from "../components/markdown-editor-popup";
import PortNumber from "../components/port";
import SelectFindingDefinition from "../components/select-finding-definition";
import ServiceName from "../components/service";
import TagList, { TagClickCallback } from "../components/tag-list";
import { WORKSPACE_CONTEXT } from "../workspace";
import WorkspaceFindingDataTable, { WorkspaceFindingDataTableRef } from "./workspace-finding-data-table";
import EditingTreeGraph, { EditingTreeGraphRef } from "./workspace-finding-editing-tree";

export type CreateFindingObject =
    | { domain: FullDomain }
    | { host: FullHost }
    | { service: FullService }
    | { httpService: FullHttpService }
    | { port: FullPort };

export type CreateFindingProps = {
    initAffected?: CreateFindingObject[];
};

export type LocalAffected = CreateFindingAffectedRequest & {
    _localScreenshot?: File;
    _localLogFile?: File;
} & (
        | { type: "Domain"; _data: FullDomain }
        | { type: "Host"; _data: FullHost }
        | { type: "Service"; _data: FullService }
        | { type: "HttpService"; _data: FullHttpService }
        | { type: "Port"; _data: FullPort }
    );

type Section = "definition" | "description" | "affected" | "network";

export function WorkspaceCreateFinding(props: CreateFindingProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [section, setSection] = React.useState<Section>("definition");

    const [severity, setSeverity] = React.useState<FindingSeverity>("Medium");
    const [findingDef, setFindingDef] = React.useState<SimpleFindingDefinition>();
    const [hoveredFindingDef, setHoveredFindingDef] = React.useState<SimpleFindingDefinition>();
    const [details, setDetails] = React.useState<string>("");
    const [categories, setCategories] = React.useState<Array<SimpleFindingCategory>>([]);
    // TODO: set categories from hovering/updating finding definitions

    const [affected, setAffected] = React.useState<Array<LocalAffected>>(
        (props.initAffected ?? []).map((a) => {
            const type = getCreateAffectedType(a);
            const data = getCreateAffectedData(a);
            return {
                _data: data satisfies LocalAffected["_data"],
                type: type satisfies LocalAffected["type"],
                uuid: data.uuid,
                details: "",
            } as LocalAffected;
        }),
    );

    const [logFile, setLogFile] = React.useState<File>();
    const [screenshot, setScreenshot] = React.useState<File>();

    const dataTableRef = React.useRef<WorkspaceFindingDataTableRef>(null);
    const graphRef = React.useRef<EditingTreeGraphRef>(null);

    const onClickTag = (e: { ctrlKey: boolean; shiftKey: boolean; altKey: boolean }, tag: SimpleTag) => {
        dataTableRef.current?.addFilterColumn("tag", tag.name, e.altKey);
        graphRef.current?.addTag(tag, e.altKey);
    };

    const addAffected = (newAffected: LocalAffected) => {
        setAffected((affected) => {
            if (affected.some((a) => a.uuid == newAffected.uuid)) return affected;

            return [
                ...affected,
                {
                    _fileDataURL: undefined,
                    _screenshotDataURL: undefined,
                    ...newAffected,
                },
            ].sort((a, b) => {
                if (aggregationTypeOrdering[a.type] < aggregationTypeOrdering[b.type]) return -1;
                if (aggregationTypeOrdering[a.type] > aggregationTypeOrdering[b.type]) return 1;
                switch (a.type) {
                    case "Domain":
                        return compareDomain(a._data, b._data as FullDomain);
                    case "Host":
                        return compareHost(a._data, b._data as FullHost);
                    case "Port":
                        return comparePort(a._data, b._data as FullPort);
                    case "Service":
                        return compareService(a._data, b._data as FullService);
                    case "HttpService":
                        return compareHttpService(a._data, b._data as FullHttpService);
                    default:
                        return 0;
                }
            });
        });
    };

    const editor = () => {
        switch (section) {
            case "definition":
                return <FindingDefinitionDetails definition={hoveredFindingDef || findingDef} />;
            case "description":
                return (
                    <Editor
                        className={"knowledge-base-editor"}
                        theme={"kraken"}
                        beforeMount={configureMonaco}
                        language={"markdown"}
                        value={details}
                        onChange={(value) => {
                            if (value !== undefined) setDetails(value);
                        }}
                    />
                );
            case "affected":
                return (
                    <div className="workspace-finding-data-table">
                        <WorkspaceFindingDataTable
                            ref={dataTableRef}
                            hideUuids={affected.map((a) => a._data.uuid)}
                            onAddDomains={(ds) =>
                                ds.map((d) =>
                                    addAffected({
                                        type: "Domain",
                                        uuid: d.uuid,
                                        details: "",
                                        _data: d,
                                    }),
                                )
                            }
                            onAddHosts={(ds) =>
                                ds.map((d) =>
                                    addAffected({
                                        type: "Host",
                                        uuid: d.uuid,
                                        details: "",
                                        _data: d,
                                    }),
                                )
                            }
                            onAddPorts={(ds) =>
                                ds.map((d) =>
                                    addAffected({
                                        type: "Port",
                                        uuid: d.uuid,
                                        details: "",
                                        _data: d,
                                    }),
                                )
                            }
                            onAddServices={(ds) =>
                                ds.map((d) =>
                                    addAffected({
                                        type: "Service",
                                        uuid: d.uuid,
                                        details: "",
                                        _data: d,
                                    }),
                                )
                            }
                            onAddHttpServices={(ds) =>
                                ds.map((d) =>
                                    addAffected({
                                        type: "HttpService",
                                        uuid: d.uuid,
                                        details: "",
                                        _data: d,
                                    }),
                                )
                            }
                        />
                    </div>
                );
            case "network":
                return (
                    <EditingTreeGraph
                        ref={graphRef}
                        definition={findingDef}
                        severity={severity}
                        affected={affected}
                        workspace={workspace}
                        maximizable
                    />
                );
            default:
                return "Unimplemented";
        }
    };

    return (
        <>
            <div className="pane">
                <div className="workspace-findings-selection-info">
                    <ArrowLeftIcon
                        title={"Back"}
                        {...ROUTES.WORKSPACE_FINDINGS_LIST.clickHandler({ uuid: workspace })}
                    />
                    <h1 className="heading">Create new finding</h1>
                </div>
                <div className="create-finding-container">
                    <form
                        className="create-finding-form"
                        onSubmit={async (e) => {
                            e.preventDefault();
                            if (findingDef === undefined) {
                                return toast.error("Please select finding definition");
                            }

                            const affectedUploaded = await Promise.all(
                                affected.map(async (a) => {
                                    const { _localLogFile: logFile, _localScreenshot: screenshot, ...request } = a;
                                    if (screenshot !== undefined) {
                                        const r = await Api.workspaces.files.uploadImage(
                                            workspace,
                                            screenshot.name,
                                            screenshot,
                                        );
                                        request.screenshot = r.unwrap().uuid;
                                    }
                                    if (logFile !== undefined) {
                                        const r = await Api.workspaces.files.uploadFile(
                                            workspace,
                                            logFile.name,
                                            logFile,
                                        );
                                        request.logFile = r.unwrap().uuid;
                                    }
                                    return request;
                                }),
                            ).catch((e) => {
                                console.error(e);
                                return null;
                            });

                            if (affectedUploaded === null) {
                                return toast.error("Some files for affected data couldn't be uploaded");
                            }

                            let screenshotUuid = null;
                            if (screenshot !== undefined) {
                                await Api.workspaces.files.uploadImage(workspace, screenshot.name, screenshot).then(
                                    handleApiError(({ uuid }) => {
                                        screenshotUuid = uuid;
                                    }),
                                );
                                if (screenshotUuid === null) return toast.error("Fail to upload screenshot");
                            }

                            let logFileUuid = null;
                            if (logFile !== undefined) {
                                await Api.workspaces.files.uploadFile(workspace, logFile.name, logFile).then(
                                    handleApiError(({ uuid }) => {
                                        logFileUuid = uuid;
                                    }),
                                );
                                if (logFileUuid === null) return toast.error("Fail to upload logfile");
                            }

                            Api.workspaces.findings
                                .create(workspace, {
                                    severity: severity,
                                    definition: findingDef.uuid,
                                    details: details,
                                    logFile: logFileUuid,
                                    screenshot: screenshotUuid,
                                    categories: categories.map((c) => c.uuid),
                                })
                                .then(
                                    handleApiError(async ({ uuid }) => {
                                        await Promise.all(
                                            affectedUploaded.map((a) => {
                                                Api.workspaces.findings
                                                    .addAffected(workspace, uuid, a)
                                                    .then(handleApiError());
                                            }),
                                        );
                                        ROUTES.WORKSPACE_FINDINGS_LIST.visit({ uuid: workspace });
                                        toast.success("Created finding");
                                    }),
                                );
                        }}
                    >
                        <div className="create-finding-header">
                            <h2 className={"sub-heading"}>Severity</h2>
                            <h2 className={"sub-heading"}>
                                <InformationIcon /> Finding Definition
                            </h2>

                            <SelectPrimitive
                                value={hoveredFindingDef?.severity || severity}
                                options={Object.values(FindingSeverity)}
                                onChange={(value) => setSeverity(value || severity)}
                            />
                            <SelectFindingDefinition
                                selected={findingDef?.uuid}
                                onSelect={(newDef) => {
                                    setFindingDef(newDef);
                                    setSeverity(newDef.severity);
                                    setCategories(newDef.categories);
                                }}
                                onHover={setHoveredFindingDef}
                            />

                            <div className="categories">
                                <h2 className="sub-heading">Categories</h2>
                                <EditableCategories
                                    categories={hoveredFindingDef?.categories || categories}
                                    onChange={setCategories}
                                />
                            </div>
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
                                <FileInput image file={screenshot} onChange={setScreenshot} />
                                <FileInput file={logFile} onChange={setLogFile} />
                            </div>

                            <CollapsibleSection
                                summary={
                                    <>
                                        <BookIcon />
                                        User Details
                                    </>
                                }
                            >
                                <GithubMarkdown>{details}</GithubMarkdown>
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
                                    {affected.length > 0 ? (
                                        affected.map((a, index) => (
                                            <CreateFindingAffected
                                                affected={a}
                                                onRemove={() => {
                                                    const copy = [...affected];
                                                    copy.splice(index, 1);
                                                    setAffected(copy);
                                                }}
                                                onClickTag={onClickTag}
                                                onChangeDetails={(d) => {
                                                    setAffected((affected) =>
                                                        affected.map((orig) =>
                                                            orig.uuid == a.uuid
                                                                ? {
                                                                      ...orig,
                                                                      details: d,
                                                                  }
                                                                : orig,
                                                        ),
                                                    );
                                                }}
                                                onChangeScreenshot={(v) => {
                                                    setAffected((affected) =>
                                                        affected.map((orig) =>
                                                            orig.uuid == a.uuid
                                                                ? {
                                                                      ...orig,
                                                                      _localScreenshot: v,
                                                                  }
                                                                : orig,
                                                        ),
                                                    );
                                                }}
                                                onChangeLogFile={(f) => {
                                                    setAffected((affected) =>
                                                        affected.map((orig) =>
                                                            orig.uuid == a.uuid
                                                                ? {
                                                                      ...orig,
                                                                      _localLogFile: f,
                                                                  }
                                                                : orig,
                                                        ),
                                                    );
                                                }}
                                            />
                                        ))
                                    ) : (
                                        <p>No affected items yet</p>
                                    )}
                                </div>
                            </CollapsibleSection>
                        </div>

                        <button type={"submit"} className="button">
                            Create
                        </button>
                    </form>
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
                                title={"Details"}
                                className={`knowledge-base-editor-tab ${section === "description" ? "selected" : ""}`}
                                onClick={() => {
                                    setSection("description");
                                }}
                            >
                                <BookIcon />
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
                        {editor()}
                    </div>
                </div>
            </div>
        </>
    );
}

export function FindingDefinitionDetails(props: { definition: SimpleFindingDefinition | null | undefined }) {
    const { definition } = props;
    if (!definition) return <div className={"create-finding-pane"} />;
    else
        return (
            <div className={"create-finding-pane"}>
                <h1 className={"sub-heading"}>
                    {definition.name} <small>{definition.severity}</small>
                </h1>
                <FindingCategoryList categories={definition.categories} />
                <p>{definition.summary}</p>
            </div>
        );
}

function isAffectedDomain(obj: CreateFindingObject): obj is { domain: FullDomain } {
    return "domain" in obj && obj["domain"] !== undefined;
}

function isAffectedHost(obj: CreateFindingObject): obj is { host: FullHost } {
    return "host" in obj && obj["host"] !== undefined;
}

function isAffectedPort(obj: CreateFindingObject): obj is { port: FullPort } {
    return "port" in obj && obj["port"] !== undefined;
}

function isAffectedService(obj: CreateFindingObject): obj is { service: FullService } {
    return "service" in obj && obj["service"] !== undefined;
}

function isAffectedHttpService(obj: CreateFindingObject): obj is { httpService: FullHttpService } {
    return "httpService" in obj && obj["httpService"] !== undefined;
}

export function getCreateAffectedType(affected: CreateFindingObject): AggregationType {
    if (isAffectedDomain(affected)) return AggregationType.Domain;
    if (isAffectedHost(affected)) return AggregationType.Host;
    if (isAffectedPort(affected)) return AggregationType.Port;
    if (isAffectedService(affected)) return AggregationType.Service;
    if (isAffectedHttpService(affected)) return AggregationType.HttpService;
    const _exhaustiveCheck: never = affected;
    throw new Error("unknown affected type?!");
}

export function getCreateAffectedKey(
    affected: CreateFindingObject,
): "domain" | "host" | "service" | "httpService" | "port" {
    if (isAffectedDomain(affected)) return "domain";
    if (isAffectedHost(affected)) return "host";
    if (isAffectedPort(affected)) return "port";
    if (isAffectedService(affected)) return "service";
    if (isAffectedHttpService(affected)) return "httpService";
    const _exhaustiveCheck: never = affected;
    throw new Error("unknown affected type?!");
}

export function getCreateAffectedData(affected: CreateFindingObject) {
    if (isAffectedDomain(affected)) return affected.domain;
    if (isAffectedHost(affected)) return affected.host;
    if (isAffectedPort(affected)) return affected.port;
    if (isAffectedHttpService(affected)) return affected.httpService;
    else return affected.service;
}

export function CreateFindingAffected({
    affected: a,
    onRemove,
    onChangeDetails,
    onChangeScreenshot,
    onChangeLogFile,
    onClickTag,
}: {
    affected: LocalAffected;
    onRemove?: () => void;
    onChangeDetails?: (content: string) => void;
    onChangeScreenshot?: (newFile: File | undefined) => void;
    onChangeLogFile?: (newFile: File | undefined) => void;
    onClickTag?: TagClickCallback;
}) {
    const label =
        a.type == "Domain" ? (
            <Domain domain={a._data} pretty />
        ) : a.type == "Host" ? (
            <IpAddr host={a._data} pretty />
        ) : a.type == "Port" ? (
            <PortNumber port={a._data} pretty />
        ) : a.type == "Service" ? (
            <ServiceName service={a._data} pretty />
        ) : a.type == "HttpService" ? (
            <HttpServiceName httpService={a._data} pretty />
        ) : (
            "not implemented"
        );

    const noop = () => {};

    return (
        <div className={`create-finding-affected affected affected-${a.type}`}>
            <div className="name">
                {onRemove && (
                    <div title={"Remove affected"} className="remove" onClick={() => onRemove()}>
                        <CloseIcon />
                    </div>
                )}
                {label}
            </div>
            {(a.details || onChangeDetails) && (
                <MarkdownEditorPopup label={label} content={a.details || ""} onChange={onChangeDetails ?? noop} />
            )}
            <TagList tags={a._data.tags} onClickTag={onClickTag} />
            {(a._localScreenshot || onChangeScreenshot) && (
                <FileInput
                    image
                    shortText
                    className="screenshot"
                    file={a._localScreenshot}
                    onChange={onChangeScreenshot ?? noop}
                >
                    <ScreenshotIcon />
                </FileInput>
            )}
            {(a._localLogFile || onChangeLogFile) && (
                <FileInput shortText className="logfile" file={a._localLogFile} onChange={onChangeLogFile ?? noop}>
                    <FileIcon />
                </FileInput>
            )}
        </div>
    );
}
