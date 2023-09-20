import React from "react";
import { Api, UUID } from "../../api/api";
import { toast } from "react-toastify";
import "../../styling/workspace.css";
import WorkspaceHeading from "./components/workspace-heading";
import WorkspaceMenu from "./components/workspace-menu";
import { FullWorkspace } from "../../api/generated";
import WorkspaceHost from "./workspace-host";
import WorkspaceHosts from "./workspace-hosts";

type WorkspaceProps = {
    uuid: UUID;
    view: WorkspaceView;
    host_uuid?: UUID;
};
type WorkspaceState = {
    workspace: FullWorkspace | null;
};

export type WorkspaceView = "search" | "attacks" | "hosts" | "data" | "workspace_settings" | "single_host";

export default class Workspace extends React.Component<WorkspaceProps, WorkspaceState> {
    constructor(props: WorkspaceProps) {
        super(props);

        this.state = {
            workspace: null,
        };
    }

    componentDidMount() {
        Api.workspaces.get(this.props.uuid).then((res) =>
            res.match(
                (workspace) => this.setState({ workspace }),
                (err) => toast.error(err.message)
            )
        );
    }

    render() {
        console.log(this.props.host_uuid);

        return (
            <div className={"workspace-container"}>
                <WorkspaceHeading
                    uuid={this.props.uuid}
                    name={this.state.workspace !== null ? this.state.workspace.name : "Loading .."}
                />
                <WorkspaceMenu uuid={this.props.uuid} active={this.props.view} />
                {this.state.workspace === null ? (
                    <></>
                ) : this.props.view === "search" ? (
                    <></>
                ) : this.props.view === "data" ? (
                    <></>
                ) : this.props.view === "workspace_settings" ? (
                    <></>
                ) : this.props.view === "attacks" ? (
                    <></>
                ) : this.props.view === "hosts" ? (
                    <WorkspaceHosts workspace={this.state.workspace} />
                ) : this.props.view === "single_host" && this.props.host_uuid !== undefined ? (
                    <WorkspaceHost workspace={this.state.workspace} host_uuid={this.props.host_uuid} />
                ) : (
                    <></>
                )}
            </div>
        );
    }
}
