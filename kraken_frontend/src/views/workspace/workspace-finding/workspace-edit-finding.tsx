import Editor from "@monaco-editor/react";
import React, { useEffect } from "react";
import { Api } from "../../../api/api";
import {
    AggregationType,
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
    UpdateFindingRequest,
} from "../../../api/generated";
import { GithubMarkdown } from "../../../components/github-markdown";
import { SelectPrimitive } from "../../../components/select-menu";
import BookIcon from "../../../svg/book";
import FileIcon from "../../../svg/file";
import InformationIcon from "../../../svg/information";
import RelationLeftRightIcon from "../../../svg/relation-left-right";
import ScreenshotIcon from "../../../svg/screenshot";
import { handleApiError } from "../../../utils/helper";
import { setupMonaco } from "../../knowledge-base";
import { WORKSPACE_CONTEXT } from "../workspace";

import { toast } from "react-toastify";
import WS from "../../../api/websocket";
import ArrowDownIcon from "../../../svg/arrow-down";
import CloseIcon from "../../../svg/close";
import GraphIcon from "../../../svg/graph";
import Domain from "../components/domain";
import { UploadingFileInput } from "../components/file-input";
import IpAddr from "../components/host";
import MarkdownEditorPopup from "../components/markdown-editor-popup";
import PortNumber from "../components/port";
import SelectFindingDefinition from "../components/select-finding-definition";
import ServiceName from "../components/service";
import TagList from "../components/tag-list";
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
    const [details, setDetails] = React.useState("");

    const [description, setDescription] = React.useState<boolean>(true);
    const [affectedVisible, setAffectedVisible] = React.useState<boolean>(true);
    const [affected, setAffected] = React.useState<Array<FullFindingAffected>>([]);
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
            handleApiError(async (x) => {
                setFindingDef(x.definition);
                setSeverity(x.severity);
                setDetails(x.userDetails || "");
                setScreenshot(x.screenshot || "");
                setLogFile(x.logFile || "");

                try {
                    setAffected(
                        await Promise.all(
                            x.affected.map((a) =>
                                Api.workspaces.findings
                                    .getAffected(workspace, finding, a.affectedUuid)
                                    .then((v) => v.unwrap()),
                            ),
                        ),
                    );
                } catch (e) {
                    toast.error("Failed to read affected data");
                }
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
            WS.addEventListener("message.AddedFindingAffected", ({ workspace: w, finding: f }) => {
                if (w !== workspace || f !== finding) return;
                // addAffected(affected)
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

    const affectedType = (a: FullFindingAffected): AggregationType =>
        (a.affected as any)["domain"]
            ? "Domain"
            : (a.affected as any)["host"]
              ? "Host"
              : (a.affected as any)["port"]
                ? "Port"
                : (a.affected as any)["service"]
                  ? "Service"
                  : (() => {
                        throw new Error("unexpected finding type");
                    })();
    const extractAffected = (a: FullFindingAffected) =>
        (a.affected as any)["domain"]
            ? (a.affected as FindingAffectedObjectOneOf).domain
            : (a.affected as any)["host"]
              ? (a.affected as FindingAffectedObjectOneOf1).host
              : (a.affected as any)["port"]
                ? (a.affected as FindingAffectedObjectOneOf2).port
                : (a.affected as any)["service"]
                  ? (a.affected as FindingAffectedObjectOneOf3).service
                  : (() => {
                        throw new Error("unexpected finding type");
                    })();

    const addAffected = (newAffected: FullFindingAffected) => {
        setAffected((affected) => {
            if (affected.some((a) => extractAffected(a).uuid == extractAffected(newAffected).uuid)) return affected;

            return [...affected, newAffected].sort((a, b) => {
                if (affectedType(a) < affectedType(b)) return -1;
                if (affectedType(a) > affectedType(b)) return 1;
                // TODO: type-based sorters
                if (extractAffected(a).uuid < extractAffected(b).uuid) return -1;
                if (extractAffected(a).uuid > extractAffected(b).uuid) return 1;
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
                                Api.workspaces.findings.addAffected(workspace, finding, {
                                    type: "Domain",
                                    uuid: d.uuid,
                                    details: "",
                                })
                            }
                            onAddHost={(d) =>
                                Api.workspaces.findings.addAffected(workspace, finding, {
                                    type: "Host",
                                    uuid: d.uuid,
                                    details: "",
                                })
                            }
                            onAddPort={(d) =>
                                Api.workspaces.findings.addAffected(workspace, finding, {
                                    type: "Port",
                                    uuid: d.uuid,
                                    details: "",
                                })
                            }
                            onAddService={(d) =>
                                Api.workspaces.findings.addAffected(workspace, finding, {
                                    type: "Service",
                                    uuid: d.uuid,
                                    details: "",
                                })
                            }
                        />
                    </div>
                );
            case "network":
                return (
                    <EditingTreeGraph
                        uuid={finding}
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
                                    affected.map((a, index) => {
                                        const label =
                                            affectedType(a) == "Domain" ? (
                                                <Domain domain={extractAffected(a) as SimpleDomain} pretty />
                                            ) : affectedType(a) == "Host" ? (
                                                <IpAddr host={extractAffected(a) as SimpleHost} pretty />
                                            ) : affectedType(a) == "Port" ? (
                                                <PortNumber port={extractAffected(a) as SimplePort} pretty />
                                            ) : affectedType(a) == "Service" ? (
                                                <ServiceName service={extractAffected(a) as SimpleService} pretty />
                                            ) : (
                                                "not implemented"
                                            );

                                        return (
                                            <div
                                                key={extractAffected(a).uuid}
                                                className={`affected affected-${affectedType(a)}`}
                                            >
                                                <div className="name">
                                                    <div
                                                        title={"Remove affected"}
                                                        className="remove"
                                                        onClick={() => {
                                                            Api.workspaces.findings.removeAffected(
                                                                workspace,
                                                                finding,
                                                                extractAffected(a).uuid,
                                                            );
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
                                                    content={a.userDetails}
                                                    onChange={(d) => {
                                                        // TODO: websocket / live editor here
                                                    }}
                                                />
                                                <TagList tags={extractAffected(a).tags || []} />
                                                <UploadingFileInput
                                                    image
                                                    shortText
                                                    className="screenshot"
                                                    file={a.screenshot ?? undefined}
                                                    onUploaded={(v) => {
                                                        Api.workspaces.findings
                                                            .updateAffected(
                                                                workspace,
                                                                finding,
                                                                extractAffected(a).uuid,
                                                                {
                                                                    screenshot: v,
                                                                },
                                                            )
                                                            .then(handleApiError);
                                                        setAffected((affected) =>
                                                            affected.map((orig) =>
                                                                extractAffected(a).uuid == extractAffected(orig).uuid
                                                                    ? {
                                                                          ...orig,
                                                                          screenshot: v,
                                                                      }
                                                                    : orig,
                                                            ),
                                                        );
                                                    }}
                                                >
                                                    <ScreenshotIcon />
                                                </UploadingFileInput>
                                                <UploadingFileInput
                                                    shortText
                                                    className="logfile"
                                                    file={a.logFile ?? undefined}
                                                    onUploaded={(v) => {
                                                        Api.workspaces.findings
                                                            .updateAffected(
                                                                workspace,
                                                                finding,
                                                                extractAffected(a).uuid,
                                                                {
                                                                    logFile: v,
                                                                },
                                                            )
                                                            .then(handleApiError);
                                                        setAffected((affected) =>
                                                            affected.map((orig) =>
                                                                extractAffected(a).uuid == extractAffected(orig).uuid
                                                                    ? {
                                                                          ...orig,
                                                                          logFile: v,
                                                                      }
                                                                    : orig,
                                                            ),
                                                        );
                                                    }}
                                                >
                                                    <FileIcon />
                                                </UploadingFileInput>
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
                        <UploadingFileInput
                            image
                            file={screenshot}
                            onUploaded={(newScreenshot) => {
                                setScreenshot((oldScreenshot) => {
                                    updateFinding({ screenshot: newScreenshot ?? null }, () => {
                                        setScreenshot(oldScreenshot);
                                    });
                                    return newScreenshot ?? "";
                                });
                            }}
                        />
                        <UploadingFileInput
                            file={logFile}
                            onUploaded={(newFile) => {
                                setLogFile((oldFile) => {
                                    updateFinding({ logFile: newFile ?? null }, () => {
                                        setLogFile(oldFile);
                                    });
                                    return newFile ?? "";
                                });
                            }}
                        />
                    </div>
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
    );
}
