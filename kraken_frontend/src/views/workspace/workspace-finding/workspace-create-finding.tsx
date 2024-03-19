import Editor from "@monaco-editor/react";
import React from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import {
    CreateFindingAffectedRequest,
    FindingSeverity,
    FullDomain,
    FullHost,
    FullPort,
    FullService,
    SimpleFindingDefinition,
} from "../../../api/generated";
import { GithubMarkdown } from "../../../components/github-markdown";
import { SelectPrimitive } from "../../../components/select-menu";
import { ROUTES } from "../../../routes";
import ArrowDownIcon from "../../../svg/arrow-down";
import BookIcon from "../../../svg/book";
import CloseIcon from "../../../svg/close";
import FileIcon from "../../../svg/file";
import GraphIcon from "../../../svg/graph";
import InformationIcon from "../../../svg/information";
import RelationLeftRightIcon from "../../../svg/relation-left-right";
import ScreenshotIcon from "../../../svg/screenshot";
import { handleApiError } from "../../../utils/helper";
import { setupMonaco } from "../../knowledge-base";
import Domain from "../components/domain";
import { FileInput } from "../components/file-input";
import IpAddr from "../components/host";
import MarkdownEditorPopup from "../components/markdown-editor-popup";
import PortNumber from "../components/port";
import SelectFindingDefinition from "../components/select-finding-definition";
import ServiceName from "../components/service";
import TagList from "../components/tag-list";
import { WORKSPACE_CONTEXT } from "../workspace";
import EditingTreeGraph from "./workspace-finding-editing-tree";
import WorkspaceFindingTable from "./workspace-finding-table";

export type CreateFindingProps = {};

type LocalAffected = CreateFindingAffectedRequest & {
    _localScreenshot?: File;
    _localLogFile?: File;
} & (
        | { type: "Domain"; _data: FullDomain }
        | { type: "Host"; _data: FullHost }
        | { type: "Service"; _data: FullService }
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

    const [description, setDescription] = React.useState<boolean>(true);
    const [affectedVisible, setAffectedVisible] = React.useState<boolean>(true);
    const [affected, setAffected] = React.useState<Array<LocalAffected>>([]);

    const [logFile, setLogFile] = React.useState<File>();
    const [screenshot, setScreenshot] = React.useState<File>();

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
                if (a.type < b.type) return -1;
                if (a.type > b.type) return 1;
                // TODO: type-based sorters
                if (a.uuid < b.uuid) return -1;
                if (a.uuid > b.uuid) return 1;
                return 0;
            });
        });
    };

    const editor = () => {
        switch (section) {
            case "definition":
                const effectiveDef = hoveredFindingDef || findingDef;
                return effectiveDef && <FindingDefinitionDetails {...effectiveDef} />;
            case "description":
                return (
                    <Editor
                        className={"knowledge-base-editor"}
                        theme={"custom"}
                        beforeMount={setupMonaco}
                        language={"markdown"}
                        value={details}
                        onChange={(value, event) => {
                            if (value !== undefined) setDetails(value);
                        }}
                    />
                );
            case "affected":
                return (
                    <div className="workspace-finding-data-table">
                        <WorkspaceFindingTable
                            onAddDomain={(d) =>
                                addAffected({
                                    type: "Domain",
                                    uuid: d.uuid,
                                    details: "",
                                    _data: d,
                                })
                            }
                            onAddHost={(d) =>
                                addAffected({
                                    type: "Host",
                                    uuid: d.uuid,
                                    details: "",
                                    _data: d,
                                })
                            }
                            onAddPort={(d) =>
                                addAffected({
                                    type: "Port",
                                    uuid: d.uuid,
                                    details: "",
                                    _data: d,
                                })
                            }
                            onAddService={(d) =>
                                addAffected({
                                    type: "Service",
                                    uuid: d.uuid,
                                    details: "",
                                    _data: d,
                                })
                            }
                        />
                    </div>
                );
            case "network":
                return (
                    <EditingTreeGraph
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
                                    let { _localLogFile: logFile, _localScreenshot: screenshot, ...request } = a;
                                    if (screenshot !== undefined) {
                                        let r = await Api.workspaces.files.uploadImage(
                                            workspace,
                                            screenshot.name,
                                            screenshot,
                                        );
                                        request.screenshot = r.unwrap().uuid;
                                    }
                                    if (logFile !== undefined) {
                                        let r = await Api.workspaces.files.uploadFile(workspace, logFile.name, logFile);
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
                                })
                                .then(
                                    handleApiError(async ({ uuid }) => {
                                        await Promise.all(
                                            affectedUploaded.map((a) =>
                                                Api.workspaces.findings
                                                    .addAffected(workspace, uuid, a)
                                                    .then(handleApiError()),
                                            ),
                                        );
                                        ROUTES.WORKSPACE_FINDINGS_EDIT.visit({ wUuid: workspace, fUuid: uuid });
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
                                value={severity}
                                options={[
                                    FindingSeverity.Okay,
                                    FindingSeverity.Low,
                                    FindingSeverity.Medium,
                                    FindingSeverity.High,
                                    FindingSeverity.Critical,
                                ]}
                                onChange={(value) => setSeverity(value || severity)}
                            />
                            <SelectFindingDefinition
                                selected={findingDef?.uuid}
                                onSelect={setFindingDef}
                                hovered={hoveredFindingDef?.uuid}
                                onHover={setHoveredFindingDef}
                            />
                        </div>

                        <div>
                            <h2 className={"sub-heading"}>
                                <BookIcon />
                                Details
                                <div
                                    className="create-finding-section-toggle"
                                    onClick={() => setDescription(!description)}
                                >
                                    <ArrowDownIcon inverted={description} />
                                </div>
                            </h2>
                            {description ? <GithubMarkdown>{details}</GithubMarkdown> : <div />}
                        </div>
                        <div>
                            <h2 className={"sub-heading"}>
                                <RelationLeftRightIcon />
                                Affected
                                <div
                                    className="create-finding-section-toggle"
                                    onClick={() => setAffectedVisible(!affectedVisible)}
                                >
                                    <ArrowDownIcon inverted={affectedVisible} />
                                </div>
                            </h2>
                            {affectedVisible && (
                                <div className="affected-list">
                                    {affected.length > 0 ? (
                                        affected.map((a, index) => {
                                            const label =
                                                a.type == "Domain" ? (
                                                    <Domain domain={a._data} pretty />
                                                ) : a.type == "Host" ? (
                                                    <IpAddr host={a._data} pretty />
                                                ) : a.type == "Port" ? (
                                                    <PortNumber port={a._data} pretty />
                                                ) : a.type == "Service" ? (
                                                    <ServiceName service={a._data} pretty />
                                                ) : (
                                                    "not implemented"
                                                );

                                            return (
                                                <div className={`affected affected-${a.type}`}>
                                                    <div className="name">
                                                        <div
                                                            title={"Remove affected"}
                                                            className="remove"
                                                            onClick={() => {
                                                                let copy = [...affected];
                                                                copy.splice(index, 1);
                                                                setAffected(copy);
                                                            }}
                                                        >
                                                            <CloseIcon />
                                                        </div>
                                                        {label}
                                                    </div>
                                                    <MarkdownEditorPopup
                                                        label={label}
                                                        content={a.details || ""}
                                                        onChange={(d) => {
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
                                                    />
                                                    <TagList tags={a._data.tags} />
                                                    <FileInput
                                                        image
                                                        shortText
                                                        className="screenshot"
                                                        file={a._localScreenshot}
                                                        onChange={(v) => {
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
                                                    >
                                                        <ScreenshotIcon />
                                                    </FileInput>
                                                    <FileInput
                                                        shortText
                                                        className="logfile"
                                                        file={a._localLogFile}
                                                        onChange={(f) => {
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
                                                    >
                                                        <FileIcon />
                                                    </FileInput>
                                                </div>
                                            );
                                        })
                                    ) : (
                                        <p>No affected items yet</p>
                                    )}
                                </div>
                            )}
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
                            <FileInput image file={screenshot} onChange={setScreenshot} />
                            <FileInput file={logFile} onChange={setLogFile} />
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

export function FindingDefinitionDetails(props: SimpleFindingDefinition) {
    const { name, severity, summary } = props;
    return (
        <div className={"create-finding-pane"}>
            <h1 className={"sub-heading"}>
                {name} <small>{severity}</small>
            </h1>
            <p>{summary}</p>
        </div>
    );
}
