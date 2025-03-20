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
    FindingAffectedObjectOneOf4,
    FindingSeverity,
    FullDomain,
    FullFindingAffected,
    FullHost,
    FullHttpService,
    FullPort,
    FullService,
    SimpleDomain,
    SimpleFindingCategory,
    SimpleFindingDefinition,
    SimpleHost,
    SimplePort,
    SimpleService,
    UpdateFindingRequest,
} from "../../../api/generated";
import { SimpleHttpService } from "../../../api/generated/models/SimpleHttpService";
import WS from "../../../api/websocket";
import FindingCategoryList from "../../../components/finding-category-list";
import { GithubMarkdown } from "../../../components/github-markdown";
import Input from "../../../components/input";
import ModelEditor from "../../../components/model-editor";
import { SelectPrimitive } from "../../../components/select-menu";
import { ROUTES } from "../../../routes";
import ArrowLeftIcon from "../../../svg/arrow-left";
import BookExportIcon from "../../../svg/book_export";
import BookToolIcon from "../../../svg/book_tool";
import BookUserIcon from "../../../svg/book_user";
import CloseIcon from "../../../svg/close";
import EditIcon from "../../../svg/edit";
import FileIcon from "../../../svg/file";
import GraphIcon from "../../../svg/graph";
import InformationIcon from "../../../svg/information";
import PersonCircleIcon from "../../../svg/person-circle";
import PlusIcon from "../../../svg/plus";
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
import { ObjectFns, handleApiError } from "../../../utils/helper";
import { useModel, useModelStore } from "../../../utils/model-controller";
import { useSyncedCursors } from "../../../utils/monaco-cursor";
import { useTimeoutOnChange } from "../../knowledge-base/edit-finding-definition";
import CollapsibleSection from "../components/collapsible-section";
import Domain from "../components/domain";
import EditableCategories from "../components/editable-categories";
import EditorPopup from "../components/editor-popup";
import { UploadingFileInput } from "../components/file-input";
import IpAddr from "../components/host";
import HttpServiceName from "../components/http-service";
import PortNumber from "../components/port";
import SelectFindingDefinition from "../components/select-finding-definition";
import ServiceName from "../components/service";
import TagList from "../components/tag-list";
import { WORKSPACE_CONTEXT } from "../workspace";
import { FindingDefinitionDetails } from "./workspace-create-finding";
import WorkspaceFindingDataTable, { WorkspaceFindingDataTableRef } from "./workspace-finding-data-table";
import EditingTreeGraph, { EditingTreeGraphRef } from "./workspace-finding-editing-tree";

/** React props for {@link WorkspaceEditFindingProps `<WorkspaceEditFindingProps />`} */
export type WorkspaceEditFindingProps = {
    /** The finding to edit identified by its uuid */
    uuid: string;
};

/** Enum of the tabs controlling the right panel */
type Section = "definition" | "userDetails" | "exportDetails" | "affected" | "network";

/** View for editing an existing findings */
export default function WorkspaceEditFinding(props: WorkspaceEditFindingProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const { uuid: finding } = props;

    const [section, setSection] = React.useState<Section>("definition");

    const [severity, setSeverity] = React.useState<FindingSeverity>("Medium");
    const [remediationDuration, setRemediationDuration] = React.useState("");
    const [categories, setCategories] = React.useState<Array<SimpleFindingCategory>>([]);
    const [findingDef, setFindingDef] = React.useState<SimpleFindingDefinition>();
    const [hoveredFindingDef, setHoveredFindingDef] = React.useState<SimpleFindingDefinition>();
    const [userDetails, setUserDetails, userDetailsModel] = useModel({ language: "markdown" });
    const [exportDetails, setExportDetails, exportDetailsModel] = useModel({ language: "text" });
    const [toolDetails, setToolDetails] = React.useState("");
    const [logFile, setLogFile] = React.useState("");
    const [screenshot, setScreenshot] = React.useState("");

    const [affected, setAffected] = React.useState<
        Record<UUID, Omit<FullFindingAffected, "userDetails" | "exportDetails">>
    >({});
    const affectedModels = useModelStore();

    /** Small wrapper around `affectedModels.addModel` prefilling most parameters */
    const addAffectedModel = (uuid: string, type: "Export" | "User", value: string) => {
        affectedModels.addModel(`${uuid}-${type.toLowerCase()}`, {
            value,
            language: "markdown",
            syncTarget: {
                findingAffected: {
                    finding: finding,
                    affected: uuid,
                    findingDetails: type,
                },
            },
        });
    };

    const dataTableRef = React.useRef<WorkspaceFindingDataTableRef>(null);
    const graphRef = React.useRef<EditingTreeGraphRef>(null);

    // Load categories from backend
    const [allCategories, setAllCategories] = React.useState<Array<SimpleFindingCategory>>([]);
    React.useEffect(() => {
        Api.findingCategories.all().then(handleApiError((v) => setAllCategories(v.categories)));
    }, []);

    useTimeoutOnChange([remediationDuration], [finding], 1000, () => {
        Api.workspaces.findings
            .update(workspace, finding, {
                remediationDuration,
            })
            .then(handleApiError);
    });

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
                setRemediationDuration(fullFinding.remediationDuration);
                setCategories(fullFinding.categories);
                setUserDetails(fullFinding.userDetails || "", {
                    finding: {
                        finding: finding,
                        findingDetails: "User",
                    },
                });
                setExportDetails(fullFinding.exportDetails || "", {
                    finding: {
                        finding: finding,
                        findingDetails: "Export",
                    },
                });
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
                        affected.map(([uuid, { userDetails, exportDetails, ...fullAffected }]) => {
                            if (userDetails.length > 0) addAffectedModel(uuid, "User", userDetails);
                            if (exportDetails.length > 0) addAffectedModel(uuid, "Export", exportDetails);
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
            WS.addEventListener("message.DeletedFinding", ({ workspace: w, finding: f }) => {
                if (w === workspace && f === finding) {
                    toast.warn("This finding was deleted by another user");
                    ROUTES.WORKSPACE_FINDINGS_LIST.visit({ uuid: workspace });
                }
            }),
            WS.addEventListener("message.UpdatedFinding", ({ workspace: w, finding: f, update }) => {
                if (w !== workspace || f !== finding) return;
                const { categories: newCategoriesUuids } = update;
                if (update.severity) {
                    setSeverity(update.severity);
                }
                if (typeof update.remediationDuration === "string") {
                    setRemediationDuration(update.remediationDuration);
                }
                if (Array.isArray(newCategoriesUuids)) {
                    let missing = false;
                    const newCategories = [];
                    for (const uuid of newCategoriesUuids) {
                        const newCategory = allCategories.find((c) => uuid === c.uuid);
                        if (newCategory !== undefined) {
                            newCategories.push(newCategory);
                        } else {
                            missing = true;
                            break;
                        }
                    }
                    if (!missing) {
                        setCategories(newCategories);
                    } else {
                        Api.findingCategories.all().then(
                            handleApiError(({ categories }) => {
                                setCategories(
                                    newCategoriesUuids.map((uuid) => categories.find((c) => uuid === c.uuid)!),
                                );
                                setAllCategories(categories);
                            }),
                        );
                    }
                }
                if (typeof update.definition === "string") {
                    Api.knowledgeBase.findingDefinitions.get(update.definition).then(handleApiError(setFindingDef));
                }
                if (typeof update.screenshot === "string") {
                    setScreenshot(update.screenshot);
                }
                if (typeof update.logFile === "string") {
                    setLogFile(update.logFile);
                }
            }),
            WS.addEventListener("message.AddedFindingAffected", ({ workspace: w, finding: f, affectedUuid }) => {
                if (w !== workspace || f !== finding) return;
                Api.workspaces.findings.getAffected(workspace, finding, affectedUuid).then(
                    handleApiError(({ userDetails, exportDetails, ...newAffected }) => {
                        setAffected((affected) => ({
                            ...affected,
                            [affectedUuid]: newAffected,
                        }));
                        if (userDetails.length > 0) addAffectedModel(affectedUuid, "User", userDetails);
                        if (exportDetails.length > 0) addAffectedModel(affectedUuid, "Export", exportDetails);
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
                affectedModels.removeModel(`${affectedUuid}-user`);
                affectedModels.removeModel(`${affectedUuid}-export`);
            }),
            WS.addEventListener("message.EditorChangedContent", ({ target }) => {
                if (!("findingAffected" in target)) return;
                if (target.findingAffected.finding !== finding) return;
                const affectedUuid = target.findingAffected.affected;
                if (affectedModels.models[`${affectedUuid}-${target.findingAffected.findingDetails.toLowerCase()}`])
                    return;

                Api.workspaces.findings.getAffected(workspace, finding, affectedUuid).then(
                    handleApiError(({ userDetails, exportDetails, ...newAffected }) => {
                        setAffected((affected) => ({
                            ...affected,
                            [affectedUuid]: newAffected,
                        }));
                        addAffectedModel(affectedUuid, "User", userDetails);
                        addAffectedModel(affectedUuid, "Export", exportDetails);
                    }),
                );
            }),
        ];
        return () => {
            setAffected({});
            affectedModels.removeAll();
            for (const handle of handles) {
                WS.removeEventListener(handle);
            }
        };
    }, [workspace, finding, allCategories]);

    const { cursors, setEditor } = useSyncedCursors({
        target: { finding: { finding, findingDetails: section === "userDetails" ? "User" : "Export" } },
        receiveCursor: (target) => {
            if ("finding" in target && target.finding.finding === finding) {
                return { findingDetails: target.finding.findingDetails };
            }
        },
        hideCursors: [section],
        isCursorHidden: ({ findingDetails }) => {
            switch (section) {
                case "userDetails":
                    return findingDetails !== "User";
                case "exportDetails":
                    return findingDetails !== "Export";
                default:
                    return true;
            }
        },
        deleteCursors: [finding],
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
                                setRemediationDuration(newDef.remediationDuration);
                                Api.workspaces.findings
                                    .update(workspace, finding, { definition: newDef.uuid })
                                    .then(handleApiError);
                            }}
                            onHover={setHoveredFindingDef}
                        />

                        <div className="categories">
                            <h2 className="sub-heading">Categories</h2>
                            <EditableCategories
                                categories={categories}
                                onChange={(newCat) => {
                                    setCategories(newCat);
                                    Api.workspaces.findings
                                        .update(workspace, finding, { categories: newCat.map((c) => c.uuid) })
                                        .then(handleApiError);
                                }}
                            />
                        </div>

                        <div>
                            <h2 className="sub-heading">Remediation Duration</h2>
                            <Input
                                value={hoveredFindingDef?.remediationDuration ?? remediationDuration}
                                onChange={setRemediationDuration}
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
                                    <BookUserIcon />
                                    User Details
                                </>
                            }
                        >
                            <GithubMarkdown>{userDetails}</GithubMarkdown>
                        </CollapsibleSection>

                        <CollapsibleSection
                            summary={
                                <>
                                    <BookExportIcon />
                                    Export Details
                                </>
                            }
                        >
                            <div>{exportDetails}</div>
                        </CollapsibleSection>

                        <CollapsibleSection
                            summary={
                                <>
                                    <BookToolIcon />
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
                                            if (aggregationTypeOrdering[aType] < aggregationTypeOrdering[bType])
                                                return -1;
                                            if (aggregationTypeOrdering[aType] > aggregationTypeOrdering[bType])
                                                return 1;
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
                                                case "HttpService":
                                                    return compareHttpService(
                                                        aObj as SimpleHttpService,
                                                        bObj as SimpleHttpService,
                                                    );
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
                                                                        affectedModels.removeModel(
                                                                            `${affectedUuid}-user`,
                                                                        );
                                                                        affectedModels.removeModel(
                                                                            `${affectedUuid}-export`,
                                                                        );
                                                                    }),
                                                                );
                                                        }}
                                                    >
                                                        <CloseIcon />
                                                    </div>
                                                    <AffectedLabel affected={fullAffected.affected} pretty />
                                                </div>
                                                <EditorPopup
                                                    trigger={
                                                        <div className="details">
                                                            {affectedModels.models[`${affectedUuid}-user`]?.value
                                                                ?.length > 0
                                                                ? ["Edit User Details", <EditIcon />]
                                                                : ["Add User Details", <PlusIcon />]}
                                                        </div>
                                                    }
                                                    onOpen={() => {
                                                        if (!affectedModels.models[`${affectedUuid}-user`]) {
                                                            addAffectedModel(affectedUuid, "User", "");
                                                        }
                                                    }}
                                                    value={affectedModels.models[`${affectedUuid}-user`]?.value}
                                                    heading={"User Details"}
                                                    subHeading={
                                                        <AffectedLabel affected={fullAffected.affected} pretty />
                                                    }
                                                    model={affectedModels.models[`${affectedUuid}-user`]?.model}
                                                />
                                                <EditorPopup
                                                    trigger={
                                                        <div className="details">
                                                            {affectedModels.models[`${affectedUuid}-export`]?.value
                                                                ?.length > 0
                                                                ? ["Edit Export Details", <EditIcon />]
                                                                : ["Add Export Details", <PlusIcon />]}
                                                        </div>
                                                    }
                                                    onOpen={() => {
                                                        if (!affectedModels.models[`${affectedUuid}-export`]) {
                                                            addAffectedModel(affectedUuid, "Export", "");
                                                        }
                                                    }}
                                                    value={affectedModels.models[`${affectedUuid}-export`]?.value}
                                                    preview={null}
                                                    heading={"Export Details"}
                                                    subHeading={
                                                        <AffectedLabel affected={fullAffected.affected} pretty />
                                                    }
                                                    model={affectedModels.models[`${affectedUuid}-export`]?.model}
                                                />
                                                <TagList
                                                    tags={fullAffected.affectedTags}
                                                    onClickTag={(event, tag) => {
                                                        dataTableRef.current?.addFilterColumn(
                                                            "tag",
                                                            tag.name,
                                                            event.altKey,
                                                        );
                                                        graphRef.current?.addTag(tag, event.altKey);
                                                    }}
                                                />
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
                        <DangerZone
                            workspace={workspace}
                            finding={finding}
                            categories={categories}
                            severity={severity}
                        />
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
                            title={"User Details"}
                            className={`knowledge-base-editor-tab ${section === "userDetails" ? "selected" : ""}`}
                            onClick={() => {
                                setSection("userDetails");
                            }}
                        >
                            <BookUserIcon />
                            {cursors.some(({ data: { findingDetails } }) => findingDetails === "User") ? (
                                <PersonCircleIcon />
                            ) : null}
                        </button>
                        <button
                            title={"Export Details"}
                            className={`knowledge-base-editor-tab ${section === "exportDetails" ? "selected" : ""}`}
                            onClick={() => {
                                setSection("exportDetails");
                            }}
                        >
                            <BookExportIcon />
                            {cursors.some(({ data: { findingDetails } }) => findingDetails === "Export") ? (
                                <PersonCircleIcon />
                            ) : null}
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
                            case "userDetails":
                            case "exportDetails":
                                return (
                                    <>
                                        <ModelEditor
                                            model={section === "userDetails" ? userDetailsModel : exportDetailsModel}
                                            setEditor={setEditor}
                                        />
                                        {cursors.map(({ data: { displayName }, cursor }) =>
                                            cursor.render(<div className={"cursor-label"}>{displayName}</div>),
                                        )}
                                    </>
                                );
                            case "affected":
                                const addAffected: {
                                    (type: "Domain", domains: Array<FullDomain>): void;
                                    (type: "Host", hosts: Array<FullHost>): void;
                                    (type: "Port", ports: Array<FullPort>): void;
                                    (type: "Service", services: Array<FullService>): void;
                                    (type: "HttpService", httpServices: Array<FullHttpService>): void;
                                } = (type: AggregationType, objects) =>
                                    Api.workspaces.findings
                                        .addAffectedBulk(
                                            workspace,
                                            finding,
                                            objects.map(({ uuid }) => ({ uuid, type })),
                                        )
                                        .then(handleApiError);
                                return (
                                    <div className="workspace-finding-data-table">
                                        <WorkspaceFindingDataTable
                                            ref={dataTableRef}
                                            hideUuids={Object.keys(affected)}
                                            onAddDomains={(v) => addAffected(AggregationType.Domain, v)}
                                            onAddHosts={(v) => addAffected(AggregationType.Host, v)}
                                            onAddPorts={(v) => addAffected(AggregationType.Port, v)}
                                            onAddServices={(v) => addAffected(AggregationType.Service, v)}
                                            onAddHttpServices={(v) => addAffected(AggregationType.HttpService, v)}
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
WorkspaceEditFinding.displayName = "WorkspaceEditFinding";

/** React props for {@link DangerZone `<DangerZone />`} */
type DangerZoneProps = {
    workspace: UUID;
    /** The finding definition to delete identified by its uuid */
    finding: UUID;
    severity: FindingSeverity;
    categories: SimpleFindingCategory[];
};

/** Danger zone containing the button and confirmation popup to delete the finding */
function DangerZone(props: DangerZoneProps) {
    const { workspace, finding, severity, categories } = props;

    const [open, setOpen] = React.useState(false);

    return (
        <>
            <div className="workspace-data-danger-pane">
                <h2 className={"sub-heading"}>Danger Zone</h2>
                <button type="button" onClick={() => setOpen(true)} className="workspace-settings-red-button button">
                    Delete finding
                </button>
            </div>
            <Popup modal nested open={open} onClose={() => setOpen(false)}>
                <div
                    className="popup-content pane danger "
                    style={{ width: "50ch", backgroundColor: "rgba(30,0,0,0.25)" }}
                >
                    <div className="workspace-setting-popup">
                        <h2 className="heading neon">
                            Are you sure you want to delete this {severity} Severity finding?
                        </h2>
                        <FindingCategoryList categories={categories} />
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
        </>
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
    if (isAffectedHttpService(affected)) return <HttpServiceName httpService={affected.httpService} pretty={pretty} />;
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

function isAffectedHttpService(obj: FindingAffectedObject): obj is FindingAffectedObjectOneOf4 {
    return "httpService" in obj && obj["httpService"] !== undefined;
}

export function getAffectedType({ affected }: { affected: FindingAffectedObject }): AggregationType {
    if (isAffectedDomain(affected)) return AggregationType.Domain;
    if (isAffectedHost(affected)) return AggregationType.Host;
    if (isAffectedPort(affected)) return AggregationType.Port;
    if (isAffectedService(affected)) return AggregationType.Service;
    if (isAffectedHttpService(affected)) return AggregationType.HttpService;
    const _exhaustiveCheck: never = affected;
    throw new Error("unknown affected type?!");
}

export function getAffectedData({ affected }: { affected: FindingAffectedObject }) {
    if (isAffectedDomain(affected)) return affected.domain;
    if (isAffectedHost(affected)) return affected.host;
    if (isAffectedPort(affected)) return affected.port;
    if (isAffectedService(affected)) return affected.service;
    if (isAffectedHttpService(affected)) return affected.httpService;
    const _exhaustiveCheck: never = affected;
    throw new Error("unknown affected type?!");
}
