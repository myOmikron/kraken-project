import React from "react";
import "../styling/workspace.css";
import { FullWorkspace } from "../api/generated";
import { Api, UUID } from "../api/api";
import { toast } from "react-toastify";

type WorkspaceProps = {
    uuid: UUID;
};
type WorkspaceState = {
    workspace: FullWorkspace | null;
    selectedTab: "domains" | "ips" | "ports" | "services";
};

export default class Workspace extends React.Component<WorkspaceProps, WorkspaceState> {
    constructor(props: WorkspaceProps) {
        super(props);

        this.state = { workspace: null, selectedTab: "domains" };
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
        return (
            <div className={"workspace-outer-container"}>
                <div className={"pane workspace-heading"}>
                    <h2 className={"heading"}>{this.state.workspace?.name}</h2>
                </div>
                <div className={"workspace-section-selector"}>
                    <div
                        className={this.state.selectedTab === "domains" ? "pane workspace-selected-tab" : "pane"}
                        onClick={() => {
                            this.setState({ selectedTab: "domains" });
                        }}
                    >
                        <h3 className={"heading"}>Domains</h3>
                    </div>
                    <div
                        className={this.state.selectedTab === "ips" ? "pane workspace-selected-tab" : "pane"}
                        onClick={() => {
                            this.setState({ selectedTab: "ips" });
                        }}
                    >
                        <h3 className={"heading"}>IPs</h3>
                    </div>
                    <div
                        className={this.state.selectedTab === "ports" ? "pane workspace-selected-tab" : "pane"}
                        onClick={() => {
                            this.setState({ selectedTab: "ports" });
                        }}
                    >
                        <h3 className={"heading"}>Ports</h3>
                    </div>
                    <div
                        className={this.state.selectedTab === "services" ? "pane workspace-selected-tab" : "pane"}
                        onClick={() => {
                            this.setState({ selectedTab: "services" });
                        }}
                    >
                        <h3 className={"heading"}>Services</h3>
                    </div>
                </div>
                <div className={"workspace-content-container"}></div>
            </div>
        );
    }
}
