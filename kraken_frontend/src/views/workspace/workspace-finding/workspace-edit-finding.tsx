import Editor from "@monaco-editor/react";
import React, { ChangeEvent } from "react";
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
import { SelectPrimitive, selectStyles } from "../../../components/select-menu";
import BookIcon from "../../../svg/book";
import FileIcon from "../../../svg/file";
import InformationIcon from "../../../svg/information";
import RelationLeftRightIcon from "../../../svg/relation-left-right";
import ScreenshotIcon from "../../../svg/screenshot";
import { handleApiError } from "../../../utils/helper";
import { setupMonaco } from "../../knowledge-base";
import { WORKSPACE_CONTEXT } from "../workspace";

import ArrowDownIcon from "../../../svg/arrow-down";
import PlusIcon from "../../../svg/plus";
import Domain from "../components/domain";
import IpAddr from "../components/host";
import PortNumber from "../components/port";
import { ScreenshotInput } from "../components/screenshot-input";
import ServiceName from "../components/service";
import TagList from "../components/tag-list";
import WorkspaceFindingTable from "./workspace-finding-table";
import { FindingDefinitionDetails } from "./workspace-create-finding";
import SelectFindingDefinition from "../components/select-finding-definition";
import WS from "../../../api/websocket";

export type WorkspaceEditFindingProps = {
    /** The finding's uuid */
    uuid: string;
};

type LocalAffected = CreateFindingAffectedRequest & {
    _localScreenshot?: File;
    _fileDataURL?: string;
} & (
        | { type: "Domain"; _data: FullDomain }
        | { type: "Host"; _data: FullHost }
        | { type: "Service"; _data: FullService }
        | { type: "Port"; _data: FullPort }
    );

const SECTION = { definition: "Definition", description: "Description", affected: "Affected", graph: "Graph" };

export default function WorkspaceEditFinding(props: WorkspaceEditFindingProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const { uuid: finding } = props;

    const [section, setSection] = React.useState<keyof typeof SECTION>("definition");
    const [hoveredFindingDef, setHoveredFindingDef] = React.useState<SimpleFindingDefinition>();

    const [severity, setSeverity] = React.useState<FindingSeverity>("Medium");
    const [findingDef, setFindingDef] = React.useState<SimpleFindingDefinition>();
    const [details, setDetails] = React.useState("");

    const [file, setFile] = React.useState<File>();
    const [fileDataURL, setFileDataURL] = React.useState<string>("");
    const [description, setDescription] = React.useState<boolean>(true);
    const [affectedVisible, setAffectedVisible] = React.useState<boolean>(true);
    const [affected, setAffected] = React.useState<Array<LocalAffected>>([]);
    const [screenshot, setScreenshot] = React.useState("");

    React.useEffect(() => {
        // Get initial state
        Api.workspaces.findings.get(workspace, finding).then(
            handleApiError((x) => {
                setFindingDef(x.definition);
                setSeverity(x.severity);
                setDetails(x.userDetails || "");
                setScreenshot(x.screenshot || "");
            }),
        );

        // Listen on state updates
        const handles = [
            WS.addEventListener("message.UpdatedFinding", ({ workspace: w, finding: f, update }) => {
                if (w !== workspace || f !== finding) return;
                const { severity, definition, screenshot } = update;
                if (severity) {
                    setSeverity(severity);
                }
                if (definition) {
                    Api.knowledgeBase.findingDefinitions.get(definition).then(handleApiError(setFindingDef));
                }
                if (screenshot) {
                    setScreenshot(screenshot);
                }
            }),
            WS.addEventListener("message.AddedFindingAffected", ({ workspace: w, finding: f }) => {
                if (w !== workspace || f !== finding) return;
            }),
            WS.addEventListener("message.UpdatedFindingAffected", ({ workspace: w, finding: f }) => {
                if (w !== workspace || f !== finding) return;
            }),
            WS.addEventListener("message.RemovedFindingAffected", ({ workspace: w, finding: f, affectedUuid }) => {
                if (w !== workspace || f !== finding) return;
                setAffected((affected) => affected.filter(({ uuid }) => uuid !== affectedUuid));
            }),
        ];
        return () => {
            for (const handle of handles) {
                WS.removeEventListener(handle);
            }
        };
    }, [workspace, finding]);

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
                        value={details}
                        onChange={(value) => setDetails(value || details)}
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
                                    _data: d,
                                })
                            }
                            onAddHost={(d) =>
                                addAffected({
                                    type: "Host",
                                    uuid: d.uuid,
                                    _data: d,
                                })
                            }
                            onAddPort={(d) =>
                                addAffected({
                                    type: "Port",
                                    uuid: d.uuid,
                                    _data: d,
                                })
                            }
                            onAddService={(d) =>
                                addAffected({
                                    type: "Service",
                                    uuid: d.uuid,
                                    _data: d,
                                })
                            }
                        />
                    </div>
                );
            default:
                return "Unimplemented";
        }
    };

    return (
        <div className="pane">
            <div className="workspace-findings-selection-info">
                <h1 className="heading">Edit finding</h1>
            </div>
            <div className="create-finding-container">
                <form className="create-finding-form" onSubmit={async () => {}}>
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

                    <div>
                        <h2 className={"sub-heading"}>
                            <BookIcon />
                            Description
                            <div className="create-finding-section-toggle" onClick={() => setDescription(!description)}>
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
                                    affected.map((a) => (
                                        <div className={`affected affected-${a.type}`}>
                                            <div>
                                                {a.type == "Domain" ? (
                                                    <Domain domain={a._data} pretty />
                                                ) : a.type == "Host" ? (
                                                    <IpAddr host={a._data} pretty />
                                                ) : a.type == "Port" ? (
                                                    <PortNumber port={a._data} pretty />
                                                ) : a.type == "Service" ? (
                                                    <ServiceName service={a._data} pretty />
                                                ) : (
                                                    "not implemented"
                                                )}
                                            </div>
                                            <TagList tags={a._data.tags} />
                                            <ScreenshotInput
                                                shortText
                                                className="screenshot"
                                                screenshot={a._localScreenshot}
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
                                            </ScreenshotInput>
                                            <div className="logfile">
                                                <FileIcon />
                                                Upload Attachment
                                            </div>
                                        </div>
                                    ))
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
                        <ScreenshotInput
                            screenshot={screenshot}
                            onChange={(newScreenshot) => {
                                if (newScreenshot === undefined) {
                                    setScreenshot("");
                                } else {
                                    Api.workspaces.files
                                        .uploadImage(workspace, newScreenshot.name, newScreenshot)
                                        .then(handleApiError(({ uuid }) => setScreenshot(uuid)));
                                }
                            }}
                            className="create-finding-screenshot-container"
                        />
                        <div className="create-finding-file-container">
                            <div className="create-finding-log-file">
                                <label className="button create-finding-file-upload" htmlFor="upload">
                                    Upload
                                </label>
                                <input
                                    id="upload"
                                    type="file"
                                    onChange={(event) => {
                                        const file = event.target.files?.[0];
                                        setFile(file);
                                        if (file !== undefined) {
                                            setFileDataURL(URL.createObjectURL(file));
                                        } else if (fileDataURL.length > 0) {
                                            URL.revokeObjectURL(fileDataURL);
                                            setFileDataURL("");
                                        }
                                    }}
                                />
                            </div>
                            {file ? (
                                <div className="create-finding-file-grid">
                                    <button title="Remove file" className="button" onClick={() => setFile(undefined)}>
                                        <PlusIcon />
                                    </button>
                                    <a
                                        className="create-finding-file-name"
                                        download={file.name}
                                        href={URL.createObjectURL(file)}
                                    >
                                        <span>{file.name}</span>
                                    </a>
                                </div>
                            ) : undefined}
                        </div>
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
                            title={"Description"}
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
                    </div>
                    {editor()}
                </div>
            </div>
        </div>
    );
}
