import React from "react";
import { Api } from "../../api/api";
import { GithubMarkdown } from "../../components/github-markdown";
import ModelEditor from "../../components/model-editor";
import "../../styling/workspace-notes.css";
import { handleApiError } from "../../utils/helper";
import { useModel } from "../../utils/model-controller";
import { useSyncedCursors } from "../../utils/monaco-cursor";
import { WORKSPACE_CONTEXT } from "./workspace";

/**
 * Page in workspace with Markdown Editor to write notes
 *
 * @returns page with markdown editor
 */
export default function WorkspaceNotes() {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [text, setText, model] = useModel({ language: "markdown" });
    const { cursors, setEditor } = useSyncedCursors({
        target: { workspaceNotes: { workspace } },
        receiveCursor: (target) => {
            if (
                "workspaceNotes" in target &&
                target["workspaceNotes"] &&
                target.workspaceNotes.workspace === workspace
            ) {
                return true;
            }
        },
        deleteCursors: [workspace],
    });

    React.useEffect(() => {
        Api.workspaces
            .get(workspace)
            .then(handleApiError(({ notes }) => setText(notes, { workspaceNotes: { workspace } })));
    }, [workspace, setText]);

    return (
        <div className={"workspace-notes-container pane"}>
            <GithubMarkdown>{text}</GithubMarkdown>
            <ModelEditor model={model} setEditor={setEditor} />
            {cursors.map(({ cursor, data }) => cursor.render(<div className={"cursor-label"}>{data.displayName}</div>))}
        </div>
    );
}
