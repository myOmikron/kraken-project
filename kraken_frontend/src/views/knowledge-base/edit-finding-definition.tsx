import React, { useEffect } from "react";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api, UUID } from "../../api/api";
import {
    FindingDefinitionUsage,
    FindingSection,
    FindingSeverity,
    ListFindingDefinitionUsages,
    SimpleFindingCategory,
} from "../../api/generated";
import WS from "../../api/websocket";
import { AdminOnly } from "../../components/admin-guard";
import { GithubMarkdown } from "../../components/github-markdown";
import Input from "../../components/input";
import ModelEditor from "../../components/model-editor";
import { SelectPrimitive } from "../../components/select-menu";
import { ROUTES } from "../../routes";
import ArrowLeftIcon from "../../svg/arrow-left";
import BandageIcon from "../../svg/bandage";
import BookIcon from "../../svg/book";
import FlameIcon from "../../svg/flame";
import InformationIcon from "../../svg/information";
import LibraryIcon from "../../svg/library";
import { handleApiError } from "../../utils/helper";
import { useSyncedCursors } from "../../utils/monaco-cursor";
import CollapsibleSection from "../workspace/components/collapsible-section";
import EditableCategories from "../workspace/components/editable-categories";
import SeverityIcon from "../workspace/components/severity-icon";
import { SectionSelectionTabs, useSectionsState } from "./finding-definition/sections";

export type EditFindingDefinitionProps = {
    uuid: string;
};

export function EditFindingDefinition(props: EditFindingDefinitionProps) {
    const [name, setName] = React.useState("");
    const [severity, setSeverity] = React.useState<FindingSeverity>(FindingSeverity.Medium);
    const [cve, setCve] = React.useState("");
    const [categories, setCategories] = React.useState<Array<SimpleFindingCategory>>([]);

    useTimeoutOnChange([name, severity, cve, categories], [props.uuid], 1000, () => {
        Api.knowledgeBase.findingDefinitions
            .update(props.uuid, { name, severity, cve, categories: categories.map((c) => c.uuid) })
            .then(handleApiError);
    });

    const sections = useSectionsState();
    const { cursors, setEditor } = useSyncedCursors({
        target: {
            findingDefinition: {
                findingDefinition: props.uuid,
                findingSection: sections.selected,
            },
        },
        receiveCursor: (target) => {
            if (
                "findingDefinition" in target &&
                target["findingDefinition"] &&
                target.findingDefinition.findingDefinition === props.uuid
            )
                return { section: target.findingDefinition.findingSection };
        },
        deleteCursors: [props.uuid],
        hideCursors: [sections.selected],
        isCursorHidden: ({ section }) => section !== sections.selected,
    });

    /* Initial load */
    React.useEffect(() => {
        Api.knowledgeBase.findingDefinitions.get(props.uuid).then(
            handleApiError((finding) => {
                setName(finding.name);
                setSeverity(finding.severity);
                setCve(finding.cve || "");
                setCategories(finding.categories);

                const target = (findingSection: FindingSection) => ({
                    findingDefinition: {
                        findingDefinition: props.uuid,
                        findingSection,
                    },
                });
                sections.Summary.set(finding.summary, target(FindingSection.Summary));
                sections.Description.set(finding.description, target(FindingSection.Description));
                sections.Impact.set(finding.impact, target(FindingSection.Impact));
                sections.Remediation.set(finding.remediation, target(FindingSection.Remediation));
                sections.References.set(finding.references, target(FindingSection.References));
            }),
        );

        const handle = WS.addEventListener("message.UpdatedFindingDefinition", ({ uuid, update }) => {
            if (uuid !== props.uuid) return;

            if (update.name !== undefined && update.name !== null) {
                setName(update.name);
            }
            if (update.severity !== undefined && update.severity !== null) {
                setSeverity(update.severity);
            }
            if (update.cve !== undefined) {
                setCve(update.cve || "");
            }
        });
        return () => WS.removeEventListener(handle);
    }, [props.uuid]);

    return (
        <div className={"create-finding-definition-container"}>
            <div className={"pane"} style={{ flex: "row" }}>
                <ArrowLeftIcon title={"Back"} {...ROUTES.FINDING_DEFINITION_LIST.clickHandler({})} />
                <h1 className={"heading"}>Edit Finding Definition</h1>
            </div>
            <div className={"pane"}>
                <div className={"create-finding-definition-form"}>
                    <div className={"create-finding-definition-header"}>
                        <h2 className={"sub-heading"}>Name</h2>
                        <h2 className={"sub-heading"}>Severity</h2>
                        <h2 className={"sub-heading"}>CVE</h2>
                        <Input maxLength={255} value={name} onChange={setName} />
                        <SelectPrimitive
                            value={severity}
                            options={Object.values(FindingSeverity)}
                            onChange={(value) => setSeverity(value || severity)}
                        />
                        <Input maxLength={255} value={cve} onChange={setCve} />
                    </div>

                    <div className="categories-selector">
                        <EditableCategories categories={categories} onChange={setCategories} />
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <InformationIcon />
                            Summary
                        </h2>
                        <div>
                            {sections.Summary.value.length === 0
                                ? null
                                : sections.Summary.value.split("\n\n").map((line) => <p>{line}</p>)}
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <BookIcon />
                            Description
                        </h2>
                        <GithubMarkdown>{sections.Description.value}</GithubMarkdown>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <FlameIcon />
                            Impact
                        </h2>
                        <GithubMarkdown>{sections.Impact.value}</GithubMarkdown>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <BandageIcon />
                            Remediation
                        </h2>
                        <GithubMarkdown>{sections.Remediation.value}</GithubMarkdown>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <LibraryIcon />
                            References
                        </h2>
                        <GithubMarkdown>{sections.References.value}</GithubMarkdown>
                    </div>

                    <AdminOnly>
                        <DeleteButton finding={props.uuid} name={name} />
                    </AdminOnly>
                </div>
                <div className={"create-finding-definition-editor"}>
                    <SectionSelectionTabs
                        sections={sections}
                        others={{
                            [FindingSection.Summary]: cursors.some(
                                ({ data: { section } }) => section === FindingSection.Summary,
                            ),
                            [FindingSection.Description]: cursors.some(
                                ({ data: { section } }) => section === FindingSection.Description,
                            ),
                            [FindingSection.Impact]: cursors.some(
                                ({ data: { section } }) => section === FindingSection.Impact,
                            ),
                            [FindingSection.Remediation]: cursors.some(
                                ({ data: { section } }) => section === FindingSection.Remediation,
                            ),
                            [FindingSection.References]: cursors.some(
                                ({ data: { section } }) => section === FindingSection.References,
                            ),
                        }}
                    />
                    <ModelEditor model={sections[sections.selected].model} setEditor={setEditor} />
                    {cursors.map(({ cursor, data }) =>
                        cursor.render(<div className={"cursor-label"}>{data.displayName}</div>),
                    )}
                </div>
            </div>
        </div>
    );
}

function DeleteButton({ finding, name }: { finding: UUID; name: string }) {
    const [open, setOpen] = React.useState(false);

    const [usage, setUsage] = React.useState<ListFindingDefinitionUsages>();

    useEffect(() => {
        Api.knowledgeBase.findingDefinitions.getUsage(finding).then(handleApiError(setUsage));
    }, [finding]);

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
                        Delete finding definition
                    </button>
                </div>
            }
        >
            <div
                className="popup-content pane workspace-setting-popup"
                style={{ width: "100ch", backgroundColor: "rgba(30,0,0,0.25)" }}
            >
                <h2 className="heading neon">Are you sure you want to delete the finding definition "{name}"?</h2>
                <p>The following findings will be deleted as a result:</p>
                {usage ? <UsageList usage={usage} /> : "Loading..."}
                <button className="button workspace-settings-red-button" type="reset" onClick={() => setOpen(false)}>
                    Cancel
                </button>
                <button
                    className="button workspace-settings-red-button"
                    type="button"
                    onClick={() => {
                        toast.promise(
                            Api.knowledgeBase.findingDefinitions.admin
                                .delete(finding)
                                .then(() => ROUTES.FINDING_DEFINITION_LIST.visit({})),
                            {
                                pending: "Deleting finding definition...",
                                error: "Failed to delete finding definition!",
                                success: "Successfully deleted finding definition",
                            },
                        );
                    }}
                >
                    Delete
                </button>
            </div>
        </Popup>
    );
}

function UsageList({ usage: { usages: usage } }: { usage: ListFindingDefinitionUsages }) {
    if (!usage.length) return "None";

    let workspaces = Object.fromEntries(usage.map((u) => [u.workspace.uuid, u.workspace]));

    let usageByWorkspace: { [workspace: UUID]: FindingDefinitionUsage[] } = {};
    for (const u of usage) {
        if (!(u.workspace.uuid in usageByWorkspace)) usageByWorkspace[u.workspace.uuid] = [];

        usageByWorkspace[u.workspace.uuid].push(u);
    }

    return Object.entries(workspaces).map(([wUuid, workspace]) => (
        <CollapsibleSection summary={"Workspace " + workspace.name} defaultVisible>
            <div className="workspace-findings-table" style={{ "--columns": "4em 1fr 25ch" } as Record<string, string>}>
                <div className={"workspace-table-header"}>
                    <span className={"workspace-data-certainty-icon"}>Severity</span>
                    <span className={"workspace-data-certainty-icon"}>Affected</span>
                    <span>Created At</span>
                </div>
                <div className="workspace-table-body">
                    {usageByWorkspace[wUuid].map((f) => (
                        <div key={f.uuid} className="workspace-table-row">
                            <span className="workspace-data-certainty-icon">
                                <SeverityIcon severity={f.severity} />
                            </span>
                            <span>
                                {[
                                    f.affectedDomains + " Domains",
                                    f.affectedHosts + " Hosts",
                                    f.affectedPorts + " Ports",
                                    f.affectedServices + " Services",
                                ]
                                    .filter((v) => !v.startsWith("0 "))
                                    .join(", ")}
                            </span>
                            <span>{f.createdAt.toLocaleString()}</span>
                        </div>
                    ))}
                </div>
            </div>
        </CollapsibleSection>
    ));
}

function useTimeoutOnChange(
    trigger: React.DependencyList,
    commit: React.DependencyList,
    timeout: number,
    effect: () => void,
) {
    const { current: state } = React.useRef({ timeout: null as null | number, effect, initial: true });
    state.effect = effect;
    React.useEffect(() => {
        if (state.initial) {
            state.initial = false;
            return;
        }

        if (state.timeout === null) {
            state.timeout = setTimeout(() => {
                state.effect.call(null);
                state.timeout = null;
            }, timeout);
        }
    }, trigger);

    // Commit any pending change before the timeout
    function commitChange() {
        if (state.timeout !== null) {
            state.effect.call(null);
            clearTimeout(state.timeout);
            state.timeout = null;
        }
    }

    // Commit any pending change if `commit` changed
    React.useEffect(commitChange, commit);
    // Commit any pending change if we'll unmount
    React.useEffect(() => commitChange, []);
}
