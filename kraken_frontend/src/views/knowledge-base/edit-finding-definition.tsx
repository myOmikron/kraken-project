import React from "react";
import Input from "../../components/input";
import { SelectPrimitive } from "../../components/select-menu";
import Editor from "@monaco-editor/react";
import { setupMonaco } from "../knowledge-base";
import { GithubMarkdown } from "../../components/github-markdown";
import BandageIcon from "../../svg/bandage";
import LibraryIcon from "../../svg/library";
import FlameIcon from "../../svg/flame";
import InformationIcon from "../../svg/information";
import BookIcon from "../../svg/book";
import { editor as editorNS } from "monaco-editor";
import { FindingSection } from "../../api/generated";
import { SectionSelectionTabs, useSectionsState } from "./finding-definition/sections";
import { Api } from "../../api/api";
import { handleApiError } from "../../utils/helper";
import useLiveEditor from "../../components/live-editor";

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
            <div className={"pane"}>
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
