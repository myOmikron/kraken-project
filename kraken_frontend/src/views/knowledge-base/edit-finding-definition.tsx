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
import { FindingSection, SimpleUser, WsClientMessageOneOf } from "../../api/generated";
import USER_CONTEXT from "../../context/user";
import { SectionSelectionTabs, useSectionsState } from "./finding-definition/sections";
import { toast } from "react-toastify";

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

    // Use a ref instead of state because our effects need to manipulate it directly
    //
    // `cursors` is an object whose pointer won't change between renders,
    // so including it in dependency lists of effect hooks won't do anything
    const { current: cursors } = React.useRef<Record<string, { section: FindingSection; cursor: Cursor<SimpleUser> }>>(
        {},
    );
    // A few places below will iterate over the cursors
    const cursorList = Object.values(cursors);

    // Hack to trigger re-renders manually if a mutation to `cursors` would require it
    const [_, setDummy] = React.useState(0);
    const rerender = () => setDummy((dummy) => dummy + 1);

    // Update which cursors to show based on the selected section
    React.useEffect(() => {
        for (const { section, cursor } of Object.values(cursors)) {
            if (section === sections.selected) {
                cursor.updateEditor(editor);
            }
        }
        return () => {
            for (const { section, cursor } of Object.values(cursors)) {
                if (section === sections.selected) {
                    cursor.updateEditor(null);
                }
            }
        };
    }, [editor, sections.selected]);

    // Save incoming cursors messages
    React.useEffect(() => {
        const handle = WS.addEventListener("message.ChangedCursorFindingDefinition", (event) => {
            if (event.findingDefinition !== props.uuid) return;
            if (event.user.uuid === user.uuid) return;

            if (event.user.uuid in cursors) {
                const { cursor, section } = cursors[event.user.uuid];

                cursor.updatePosition(event.line, event.column);
                if (event.findingSection === sections.selected && section !== sections.selected) {
                    cursor.updateEditor(editor);
                } else if (event.findingSection !== sections.selected && section === sections.selected) {
                    cursor.updateEditor(null);
                }

                cursors[event.user.uuid] = { cursor, section: event.findingSection };
            } else {
                cursors[event.user.uuid] = {
                    section: event.findingSection,
                    cursor: new Cursor(
                        event.findingSection === sections.selected ? editor : null,
                        event.user,
                        event.line,
                        event.column,
                    ),
                };
            }
            rerender();
        });
        return () => {
            WS.removeEventListener(handle);
        };
    }, [editor, sections.selected, props.uuid, user.uuid]);

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
        // const handle = WS.addEventListener("message.EditFindingDefinition", (event) => {
        WS_MOCK.onReceive = (event) => {
            if (event.findingDefinition !== props.uuid) return;
            // if (event.user.uuid === user.uuid) return; // TODO: might need more consideration

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
        };
        //return () => WS.removeEventListener(handle);
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
                    WS_MOCK.send({
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
                        .map(({ cursor }) =>
                            cursor.render(<div className={"cursor-label"}>{cursor.user.displayName}</div>),
                        )}
                </div>
            </div>
        </div>
    );
}

// TODO replace mockup with backend
const WS_MOCK = {
    delay: 5_000,
    timeout: null as number | null,
    events: [] as Array<{ event: WsClientMessageOneOf; delay: number }>,
    firstSend: new Date().getTime(),
    send(event: WsClientMessageOneOf) {
        console.debug("Got send");
        if (this.events.length === 0) {
            this.events.push({ event, delay: 0 });
            this.firstSend = new Date().getTime();
        } else {
            const now = new Date().getTime();
            this.events.push({ event, delay: now - this.firstSend });
        }

        if (this.timeout !== null) clearTimeout(this.timeout);
        this.timeout = setTimeout(() => {
            console.debug("Started echoing");
            for (const { event, delay } of this.events) {
                setTimeout(() => this.onReceive(event), delay);
            }
            this.events = [];
            this.timeout = null;
        }, this.delay);
    },
    onReceive: (_: WsClientMessageOneOf) => {
        console.debug("Got default receive");
    },
};
// @ts-ignore
window.clearMock = () => {
    WS_MOCK.events = [];
    if (WS_MOCK.timeout !== null) clearTimeout(WS_MOCK.timeout);
    WS_MOCK.timeout = null;
};
