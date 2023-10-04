import React from "react";
import { FullWorkspace } from "../../api/generated";

type WorkspaceSettingsProps = {
    workspace: FullWorkspace;
};
type WorkspaceSettingsState = {};

export default class WorkspaceSettings extends React.Component<WorkspaceSettingsProps, WorkspaceSettingsState> {
    constructor(props: WorkspaceSettingsProps) {
        super(props);

        this.state = {};
    }

    render() {
        return (
            <div className={"pane workspace-settings-container"}>
                <h1 className={"heading"}> Workspace Settings </h1>

                <div className="pane"></div>
            </div>
        );
    }
}
