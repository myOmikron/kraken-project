import React from "react";
import { WORKSPACE_CONTEXT } from "./workspace";
import { GithubMarkdown } from "../../components/github-markdown";
import Editor from "@monaco-editor/react";
import { setupMonaco } from "../knowledge-base";
import { editor } from "monaco-editor";
import "../../styling/workspace-notes.css";
import useLiveEditor from "../../components/live-editor";
import { Api } from "../../api/api";
import { handleApiError } from "../../utils/helper";

export type WorkspaceNotesProps = {};

export default function WorkspaceNotes(props: WorkspaceNotesProps) {
    const { workspace } = React.useContext(WORKSPACE_CONTEXT);

    const [text, setText] = React.useState("");

    const [editorInstance, setEditorInstance] = React.useState<editor.IStandaloneCodeEditor | null>(null);

    const { cursors, onChange } = useLiveEditor({
        editorInstance,
        target: {
            workspaceNotes: {
                workspace: workspace.uuid,
            },
        },
        receiveCursor: (target) => {
            if ("workspaceNotes" in target && target.workspaceNotes.workspace === workspace.uuid) {
                return true;
            }
        },
        deleteCursors: [workspace.uuid],
        receiveEdit: (target, editorInstance) => {
            if ("workspaceNotes" in target && target.workspaceNotes.workspace === workspace.uuid) {
                const model = editorInstance && editorInstance.getModel();
                if (model !== null) {
                    return { model };
                }
            }
        },
        setValue: setText,
        onUserEdit: (value) => {
            window.localStorage.setItem(`workspaces/${workspace.uuid}/notes`, value);
        },
    });

    React.useEffect(() => {
        Api.workspaces.get(workspace.uuid).then(handleApiError(({ notes }) => setText(notes)));
    }, [workspace.uuid]);

    return (
        <div className={"workspace-notes-container pane"}>
            <GithubMarkdown>{text}</GithubMarkdown>
            <Editor
                className={"knowledge-base-editor"}
                theme={"custom"}
                beforeMount={setupMonaco}
                value={text}
                language={"markdown"}
                onChange={onChange}
                onMount={setEditorInstance}
            />
            {cursors.map(({ data: { displayName }, cursor }) =>
                cursor.render(<div className={"cursor-label"}>{displayName}</div>),
            )}
        </div>
    );
}
