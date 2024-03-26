import React from "react";
import { WORKSPACE_CONTEXT } from "./workspace";
import { GithubMarkdown } from "../../components/github-markdown";
import "../../styling/workspace-notes.css";
import { Api } from "../../api/api";
import { handleApiError } from "../../utils/helper";
import { useModel } from "../../utils/model-controller";
import ModelEditor from "../../components/model-editor";
import { useSyncedCursors } from "../../utils/monaco-cursor";

export type WorkspaceNotesProps = {};

export default function WorkspaceNotes(props: WorkspaceNotesProps) {
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
