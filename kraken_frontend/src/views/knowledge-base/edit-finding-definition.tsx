import React from "react";
import Input from "../../components/input";
import { SelectPrimitive } from "../../components/select-menu";
import Editor, { useMonaco } from "@monaco-editor/react";
import { setupMonaco } from "../knowledge-base";
import { GithubMarkdown } from "../../components/github-markdown";
import BandageIcon from "../../svg/bandage";
import LibraryIcon from "../../svg/library";
import FlameIcon from "../../svg/flame";
import InformationIcon from "../../svg/information";
import BookIcon from "../../svg/book";
import { editor as editorNS } from "monaco-editor";
import Cursor from "../../utils/monaco-cursor";
import WS from "../../api/websocket";
import { FindingSection, SimpleUser } from "../../api/generated";
import USER_CONTEXT from "../../context/user";
import { SectionSelectionTabs, useSectionsState } from "./finding-definition/sections";
import { toast } from "react-toastify";
import { Api } from "../../api/api";
import { handleApiError } from "../../utils/helper";

export type EditFindingDefinitionProps = {
    uuid: string;
};
export function EditFindingDefinition(props: EditFindingDefinitionProps) {
    const { user } = React.useContext(USER_CONTEXT);

    const [name, setName] = React.useState("");
    const [severity, setSeverity] = React.useState("Medium");
    const [cve, setCve] = React.useState("");

    const sections = useSectionsState();

    const monaco = useMonaco();
    const [editor, setEditor] = React.useState<editorNS.IStandaloneCodeEditor | null>(null);

    /* CURSORS */

    const [cursors, rawSetCursors] = React.useState<
        Record<string, { section: FindingSection; user: SimpleUser; cursor: Cursor }>
    >({});
    // Restrict `setCursors` to "update" usage only prohibiting "overwrite" because the contained `Cursor` objects are stateful
    const setCursors = rawSetCursors as (update: (prevState: typeof cursors) => typeof cursors) => void;
    // A few places below will iterate over the cursors
    const cursorList = Object.values(cursors);

    // Reset cursors when switching definition
    React.useEffect(
        () =>
            setCursors((cursors) => {
                for (const { cursor } of Object.values(cursors)) {
                    cursor.delete();
                }
                return {};
            }),
        [props.uuid, setCursors],
    );

    // Update which cursors to show based on the selected section
    React.useEffect(
        () =>
            setCursors((cursors) => {
                for (const { section, cursor } of Object.values(cursors)) {
                    cursor.updateActive(section === sections.selected);
                }
                return { ...cursors };
            }),
        [sections.selected, setCursors],
    );

    // Pass the editor to cursors which have been created before the editor was loaded
    React.useEffect(
        () =>
            setCursors((cursors) => {
                for (const { section, cursor } of Object.values(cursors)) {
                    cursor.updateEditor(editor);
                }
                return { ...cursors };
            }),
        [editor, setCursors],
    );

    // Save incoming cursors messages
    React.useEffect(() => {
        const handle = WS.addEventListener("message.ChangedCursorFindingDefinition", (event) => {
            if (event.findingDefinition !== props.uuid) return;
            if (event.user.uuid === user.uuid) return;

            setCursors((cursors) => {
                let cursor;
                if (event.user.uuid in cursors) {
                    cursor = cursors[event.user.uuid].cursor;
                    cursor.updatePosition(event.line, event.column);
                    cursor.updateActive(event.findingSection === sections.selected);
                } else {
                    cursor = new Cursor(editor, event.line, event.column, event.findingSection === sections.selected);
                }
                return { ...cursors, [event.user.uuid]: { cursor, section: event.findingSection, user: event.user } };
            });
        });
        return () => {
            WS.removeEventListener(handle);
        };
    }, [editor, sections.selected, props.uuid, user.uuid, setCursors]);

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

    /* EDITS */

    // Boolean flag which indicates whether changes to the editor should be sent over the websocket
    //
    // When we apply changed we received from the websocket, they will trigger the `onChange` handler.
    // But those events shouldn't be sent as the user's edits. So this flag can disable the "send changes to ws"-part.
    // This is stored a ref instead of state because it needs to be changed in between renders.
    const sendChanges = React.useRef(true);

    // Save incoming edit messages
    React.useEffect(() => {
        const handle = WS.addEventListener("message.EditFindingDefinition", (event) => {
            if (event.findingDefinition !== props.uuid) return;
            if (event.user.uuid === user.uuid) return; // TODO: might need more consideration

            if (monaco === null) {
                // TODO
                toast.error("Well shit...");
                return;
            }

            // Disable sending edit events
            sendChanges.current = false;

            const editorModel = editor && editor.getModel();
            if (editorModel !== null && event.findingSection === sections.selected) {
                const { text, startColumn, startLine, endColumn, endLine } = event.change;
                const undo = editorModel.applyEdits(
                    [{ range: { startColumn, endColumn, startLineNumber: startLine, endLineNumber: endLine }, text }],
                    true,
                );
                // TODO: use `undo`'s range to highlight changes
            } else {
                const uri = monaco.Uri.parse(event.findingSection);
                const model =
                    monaco.editor.getModel(uri) ||
                    monaco.editor.createModel(
                        sections[event.findingSection].value,
                        sections[event.findingSection].editor.language,
                        uri,
                    );

                const { text, startColumn, startLine, endColumn, endLine } = event.change;
                model.applyEdits(
                    [{ range: { startColumn, endColumn, startLineNumber: startLine, endLineNumber: endLine }, text }],
                    true,
                );
                sections[event.findingSection].set(model.getValue());
            }

            // Re-enable sending edit events
            sendChanges.current = true;
        });
        return () => WS.removeEventListener(handle);
    }, [monaco, editor, props.uuid, user.uuid, sections]);

    // Send outgoing edit messages
    //
    // This function is passed to `<Editor onChange={...} />`.
    // We use `useCallback` because `<Editor />` uses it internally in a dependency list.
    const onChange = React.useCallback(
        (value: string | undefined, event: editorNS.IModelContentChangedEvent) => {
            // Update the React state
            if (value !== undefined) {
                sections[sections.selected].set(value);
            }

            // Send the changes to the websocket
            if (sendChanges.current) {
                for (const change of event.changes) {
                    const {
                        text,
                        range: { startColumn, startLineNumber, endLineNumber, endColumn },
                    } = change;
                    WS.send({
                        type: "EditFindingDefinition",
                        findingDefinition: props.uuid,
                        findingSection: sections.selected,
                        change: { text, startColumn, endColumn, startLine: startLineNumber, endLine: endLineNumber },
                    });
                }
            }
        },
        [sections, props.uuid],
    );

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
                setCursors((cursors) => {
                    for (const { cursor } of Object.values(cursors)) {
                        cursor.delete();
                    }
                    return {};
                });
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
                            Summary: cursorList.some(({ section }) => section === FindingSection.Summary),
                            Description: cursorList.some(({ section }) => section === FindingSection.Description),
                            Impact: cursorList.some(({ section }) => section === FindingSection.Impact),
                            Remediation: cursorList.some(({ section }) => section === FindingSection.Remediation),
                            References: cursorList.some(({ section }) => section === FindingSection.References),
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
                    {cursorList
                        .filter(({ section }) => sections.selected === section)
                        .map(({ user, cursor }) =>
                            cursor.render(<div className={"cursor-label"}>{user.displayName}</div>),
                        )}
                </div>
            </div>
        </div>
    );
}
