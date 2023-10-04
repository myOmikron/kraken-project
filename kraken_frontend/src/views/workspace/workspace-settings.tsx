import React from "react";
import { FullWorkspace } from "../../api/generated";
import "../../styling/workspace-settings.css";
import Input from "../../components/input";
import { Api } from "../../api/api";
import Textarea from "../../components/textarea";
import { toast } from "react-toastify";

type WorkspaceSettingsProps = {
    workspace: FullWorkspace;
};
type WorkspaceSettingsState = {
    workspaceName: string;
    workspaceDescription: string | null | undefined;
};

export default class WorkspaceSettings extends React.Component<WorkspaceSettingsProps, WorkspaceSettingsState> {
    constructor(props: WorkspaceSettingsProps) {
        super(props);

        this.state = {
            workspaceName: this.props.workspace.name,
            workspaceDescription: this.props.workspace.description,
        };
    }

    componentDidMount() {
        this.updateWorkspace().then();
    }

    async updateWorkspace() {
        (
            await Api.workspaces.update(this.props.workspace.uuid, {
                name: this.state.workspaceName,
                description: this.state.workspaceDescription,
            })
        ).match(
            () => toast.success("Workspace updated"),
            (err) => toast.error(err.message)
        );
    }

    render() {
        return (
            <div className={"workspace-settings-layout"}>
                <form
                    className="pane workspace-settings-container"
                    method={"post"}
                    onSubmit={async (x) => {
                        x.preventDefault();
                        await this.updateWorkspace();
                    }}
                >
                    <h2 className={"heading"}> Workspace Settings </h2>
                    <div className={"workspace-settings-table"}>
                        <span>Name</span>
                        <Input
                            value={this.state.workspaceName}
                            onChange={(v) => {
                                this.setState({ workspaceName: v });
                            }}
                            placeholder={this.props.workspace.name}
                        />
                        <span>Description</span>
                        <Textarea
                            value={
                                this.state.workspaceDescription !== null &&
                                this.state.workspaceDescription !== undefined
                                    ? this.state.workspaceDescription
                                    : ""
                            }
                            onChange={(v) => {
                                this.setState({ workspaceDescription: v });
                            }}
                            placeholder={"Description"}
                        />
                    </div>
                    <button className={"button"}>Save</button>
                </form>

                <div className="pane">
                    <h2 className={"heading"}>User control</h2>

                    <div className={"workspace-settings-container"}>
                        <div className={"workspace-settings-invite"}>
                            <button className={"button"}>Invite</button>
                        </div>
                        <div className={"workspace-settings-user-table-heading"}>
                            <span>Username</span>
                            <span>Role</span>
                        </div>
                        <div className={"workspace-settings-user-table-entry"}></div>
                    </div>
                </div>
                <div className="pane">
                    <h2 className={"heading"}>Linked OAuth applications </h2>
                </div>
                {/* <div className="pane">
                    <h2 className={"heading"}>Danger Zone </h2>
                </div>*/}
            </div>
        );
    }
}
