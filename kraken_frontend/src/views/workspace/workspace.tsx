import React from "react";
import { Api, UUID } from "../../api/api";
import { toast } from "react-toastify";
import "../../styling/workspace.css";
import WorkspaceHeading from "./components/workspace-heading";
import WorkspaceMenu from "./components/workspace-menu";
import { FullWorkspace } from "../../api/generated";
import { handleApiError } from "../../utils/helper";

export type WorkspaceContext = { workspace: FullWorkspace };
export const WORKSPACE_CONTEXT = React.createContext<WorkspaceContext>({
    workspace: {
        name: "",
        description: "",
        uuid: "",
        owner: { uuid: "", username: "", displayName: "" },
        attacks: [],
        members: [],
        notes: "",
        createdAt: new Date(),
    },
});
WORKSPACE_CONTEXT.displayName = "WorkspaceContext";

type WorkspaceProps = {
    uuid: UUID;
    view: WorkspaceView;
    children: React.ReactNode;
};
type WorkspaceState = {
    workspace: FullWorkspace | null;
};

export type WorkspaceView = "search" | "attacks" | "findings" | "data" | "hosts" | "settings" | "notes";

export default class Workspace extends React.Component<WorkspaceProps, WorkspaceState> {
    constructor(props: WorkspaceProps) {
        super(props);

        this.state = {
            workspace: null,
        };
    }

    componentDidMount() {
        Api.workspaces.get(this.props.uuid).then(handleApiError((workspace) => this.setState({ workspace })));
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceProps>, prevState: Readonly<WorkspaceState>, snapshot?: any) {
        if (prevProps.uuid !== this.props.uuid) {
            Api.workspaces.get(this.props.uuid).then(handleApiError((workspace) => this.setState({ workspace })));
        }
    }

    render() {
        return (
            <div className={"workspace-container"}>
                <WorkspaceHeading
                    uuid={this.props.uuid}
                    name={this.state.workspace !== null ? this.state.workspace.name : "Loading .."}
                />
                <WorkspaceMenu
                    uuid={this.props.uuid}
                    owner={this.state.workspace !== null ? this.state.workspace.owner.uuid : ""}
                    active={this.props.view}
                />
                {this.state.workspace && (
                    <WORKSPACE_CONTEXT.Provider value={{ workspace: this.state.workspace }}>
                        {this.props.children}
                    </WORKSPACE_CONTEXT.Provider>
                )}
            </div>
        );
    }
}
