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
import { CursorLabels, useCursors } from "./cursors";
import WS from "../../api/websocket";
import { SimpleUser } from "../../api/generated";
import USER_CONTEXT from "../../context/user";
import { FindingSection, SectionSelectionTabs, useSectionsState } from "./finding-definition/sections";
import { ObjectFns } from "../../utils/helper";

export type EditFindingDefinitionProps = {
    uuid: string;
};
export function EditFindingDefinition(props: EditFindingDefinitionProps) {
    const { user } = React.useContext(USER_CONTEXT);

    const [name, setName] = React.useState("");
    const [severity, setSeverity] = React.useState("Medium");
    const [cve, setCve] = React.useState("");

    const sections = useSectionsState();

    const [editor, setEditor] = React.useState<editorNS.IStandaloneCodeEditor | null>(null);

    const summaryCursors = useCursors<SimpleUser>(sections.Summary.selected ? editor : null);
    const descriptionCursors = useCursors<SimpleUser>(sections.Description.selected ? editor : null);
    const impactCursors = useCursors<SimpleUser>(sections.Impact.selected ? editor : null);
    const remediationCursors = useCursors<SimpleUser>(sections.Remediation.selected ? editor : null);
    const referencesCursors = useCursors<SimpleUser>(sections.References.selected ? editor : null);
    const cursors: Record<FindingSection, ReturnType<typeof useCursors<SimpleUser>>> = {
        Summary: summaryCursors,
        Description: descriptionCursors,
        Impact: impactCursors,
        Remediation: remediationCursors,
        References: referencesCursors,
    };
    const usersLastSection = React.useRef<Record<string, FindingSection>>({});

    // Save incoming cursors messages
    /*React.useEffect(() => {
        const handle = WS.addEventListener("message.ChangedCursorFindingDefinition", (event) => {
            if (event.findingDefinition !== props.uuid) return;
            if (event.user.uuid === user.uuid) return;

            // Remove a previous cursor from a different section
            const lastSection = usersLastSection.current[event.user.uuid];
            if (lastSection !== undefined && lastSection !== event.findingSection)
                cursors[lastSection].remove(event.user);

            cursors[event.findingSection].insert(event.user, event.line, event.column);
            usersLastSection.current[event.user.uuid] = event.findingSection;
        });
        return () => {
            WS.removeEventListener(handle);
        };
    }, [
        props.uuid,
        user.uuid,
        summaryCursors.insert,
        descriptionCursors.insert,
        impactCursors.insert,
        remediationCursors.insert,
        referencesCursors.insert,
    ]);*/

    // Send outgoing cursor messages
    React.useEffect(() => {
        if (editor !== null) {
            const disposable = editor.onDidChangeCursorPosition((event) => {
                WS.send({
                    type: "ChangedCursorFindingDefinition",
                    findingDefinition: props.uuid,
                    findingSection: sections.selected,
                    line: event.position.lineNumber,
                    column: event.position.column,
                });
            });
            return () => disposable.dispose();
        }
    }, [editor, sections.selected, props.uuid]);

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
                        <div className={`nested-pane`}>
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
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{sections.Description.value}</GithubMarkdown>
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <FlameIcon />
                            Impact
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{sections.Impact.value}</GithubMarkdown>
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <BandageIcon />
                            Remediation
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{sections.Remediation.value}</GithubMarkdown>
                        </div>
                    </div>

                    <div>
                        <h2 className={"sub-heading"}>
                            <LibraryIcon />
                            References
                        </h2>
                        <div className={`nested-pane`}>
                            <GithubMarkdown>{sections.References.value}</GithubMarkdown>
                        </div>
                    </div>
                </div>
                <div className={"create-finding-definition-editor"}>
                    <SectionSelectionTabs
                        sections={sections}
                        others={{
                            Summary: !ObjectFns.isEmpty(summaryCursors.cursors),
                            Description: !ObjectFns.isEmpty(descriptionCursors.cursors),
                            Impact: !ObjectFns.isEmpty(impactCursors.cursors),
                            Remediation: !ObjectFns.isEmpty(remediationCursors.cursors),
                            References: !ObjectFns.isEmpty(referencesCursors.cursors),
                        }}
                    />
                    <Editor
                        className={"knowledge-base-editor"}
                        theme={"custom"}
                        beforeMount={setupMonaco}
                        language={sections[sections.selected].language}
                        value={sections[sections.selected].value}
                        onChange={(value, event) => {
                            if (value !== undefined) sections[sections.selected].set(value);
                        }}
                        onMount={setEditor}
                    />
                    <CursorLabels {...cursors[sections.selected]}>
                        {({ displayName }) => <div className={"cursor-label"}>{displayName}</div>}
                    </CursorLabels>
                </div>
            </div>
        </div>
    );
}
