import Editor from "@monaco-editor/react";
import { editor as editorNS } from "monaco-editor";
import React from "react";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api, UUID } from "../../api/api";
import { FindingSection } from "../../api/generated";
import { AdminOnly } from "../../components/admin-guard";
import { GithubMarkdown } from "../../components/github-markdown";
import Input from "../../components/input";
import useLiveEditor from "../../components/live-editor";
import { SelectPrimitive } from "../../components/select-menu";
import { ROUTES } from "../../routes";
import ArrowLeftIcon from "../../svg/arrow-left";
import BandageIcon from "../../svg/bandage";
import BookIcon from "../../svg/book";
import FlameIcon from "../../svg/flame";
import InformationIcon from "../../svg/information";
import LibraryIcon from "../../svg/library";
import { handleApiError } from "../../utils/helper";
import { setupMonaco } from "../knowledge-base";
import { SectionSelectionTabs, useSectionsState } from "./finding-definition/sections";

export type EditFindingDefinitionProps = {
    uuid: string;
};

export function EditFindingDefinition(props: EditFindingDefinitionProps) {
    const [name, setName] = React.useState("");
    const [severity, setSeverity] = React.useState("Medium");
    const [cve, setCve] = React.useState("");

    const sections = useSectionsState();

    const [editor, setEditor] = React.useState<editorNS.IStandaloneCodeEditor | null>(null);

    const { cursors, onChange } = useLiveEditor({
        editorInstance: editor,
        target: {
            findingDefinition: {
                findingDefinition: props.uuid,
                findingSection: sections.selected,
            },
        },
        receiveCursor: (target) => {
            if ("findingDefinition" in target && target.findingDefinition.findingDefinition === props.uuid) {
                return { section: target.findingDefinition.findingSection };
            }
        },
        deleteCursors: [props.uuid],
        hideCursors: [sections.selected],
        isCursorHidden: ({ section }: { section: FindingSection }) => section !== sections.selected,
        receiveEdit: (target, editorInstance, monaco) => {
            if ("findingDefinition" in target && target.findingDefinition.findingDefinition === props.uuid) {
                const { findingSection } = target.findingDefinition;

                if (findingSection === sections.selected) {
                    const model = editorInstance && editorInstance.getModel();
                    if (model !== null) return { model };
                }

                const uri = monaco.Uri.parse(findingSection);
                const model =
                    monaco.editor.getModel(uri) ||
                    monaco.editor.createModel(
                        sections[findingSection].value,
                        sections[findingSection].editor.language,
                        uri,
                    );
                return { model, setValue: sections[findingSection].set };
            }
        },
        setValue: sections[sections.selected].set,
    });

    /* Initial load */

    React.useEffect(() => {
        Api.knowledgeBase.findingDefinitions.get(props.uuid).then(
            handleApiError((finding) => {
                setName(finding.name);
                setSeverity(finding.severity);
                setCve(finding.cve || "");
                sections.Summary.set(finding.summary);
                sections.Description.set(finding.description);
                sections.Impact.set(finding.impact);
                sections.Remediation.set(finding.remediation);
                sections.References.set(finding.references);
            }),
        );
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
                            options={["Okay", "Low", "Medium", "High", "Critical"]}
                            onChange={(value) => setSeverity(value || severity)}
                        />
                        <Input maxLength={255} value={cve} onChange={setCve} />
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
                            Summary: cursors.some(({ data: { section } }) => section === FindingSection.Summary),
                            Description: cursors.some(
                                ({ data: { section } }) => section === FindingSection.Description,
                            ),
                            Impact: cursors.some(({ data: { section } }) => section === FindingSection.Impact),
                            Remediation: cursors.some(
                                ({ data: { section } }) => section === FindingSection.Remediation,
                            ),
                            References: cursors.some(({ data: { section } }) => section === FindingSection.References),
                        }}
                    />
                    <Editor
                        className={"knowledge-base-editor"}
                        theme={"custom"}
                        beforeMount={setupMonaco}
                        {...sections[sections.selected].editor}
                        onChange={onChange}
                        onMount={setEditor}
                    />
                    {cursors
                        .filter(({ data: { section } }) => sections.selected === section)
                        .map(({ data: { displayName }, cursor }) =>
                            cursor.render(<div className={"cursor-label"}>{displayName}</div>),
                        )}
                </div>
            </div>
        </div>
    );
}

function DeleteButton({ finding, name }: { finding: UUID; name: string }) {
    const [open, setOpen] = React.useState(false);

    return (
        <Popup
            modal
            nested
            open={open}
            onClose={() => setOpen(false)}
            trigger={
                <div>
                    <button onClick={() => setOpen(true)} className="button danger" type="button">
                        Delete this Finding
                    </button>
                </div>
            }
        >
            <div className="popup-content pane" style={{ width: "78ch" }}>
                <h1 className="heading neon">Are you sure you want to delete the finding definition "{name}"?</h1>
                <div>
                    <p>The following findings will be deleted due to this:</p>
                    <ul>
                        <li>TODO</li>
                    </ul>
                </div>
                <button
                    className="button danger"
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
                <button className="button" type="reset" onClick={() => setOpen(false)}>
                    Cancel
                </button>
            </div>
        </Popup>
    );
}
