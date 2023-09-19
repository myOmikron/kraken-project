import React from "react";
import "../../styling/workspace.css";
import { FullWorkspace } from "../../api/generated";
import { Api, UUID } from "../../api/api";
import { toast } from "react-toastify";
import CopyIcon from "../../svg/copy";
import { copyToClipboard } from "../../utils/helper";
import TuxIcon from "../../svg/tux";
import WindowsIcon from "../../svg/windows";
import FreeBSDIcon from "../../svg/freebsd";
import AppleIcon from "../../svg/apple";
import AnonymousIcon from "../../svg/anonymous";
import WorkspaceMenu from "./components/workspace-menu";

type WorkspaceProps = {
    uuid: UUID;
};
type WorkspaceState = {
    workspace: FullWorkspace | null;
    selectedTab: "domains" | "ips" | "ports" | "services" | "other";
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
                <div className={"pane workspace-host-list"}></div>
                <div className={"pane workspace-host-container"}>
                    <FreeBSDIcon />
                    <div className={"workspace-host-details"}>
                        <h2 className={"heading"}>Host 231.321.231.213</h2>
                        <span>OS: Linux</span>
                        <span>Online: Yes</span>
                    </div>
                </div>
                <WorkspaceMenu
                    additionalClassName={"workspace-menu-ct"}
                    uuid={this.state.workspace !== null ? this.state.workspace.uuid : ""}
                />
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
                    <div
                        className={this.state.selectedTab === "other" ? "pane workspace-selected-tab" : "pane"}
                        onClick={() => {
                            this.setState({ selectedTab: "other" });
                        }}
                    >
                        <h3 className={"heading"}>Other</h3>
                    </div>
                </div>
                <div className={"pane workspace-content-table"}>
                    <div className={"workspace-content-row"}>
                        <span>Domain</span>
                        <span>DNS</span>
                        <span>Tags</span>
                        <span>Attacks</span>
                        <span>Comment</span>
                    </div>
                    <div className={"workspace-content-row"}>
                        <span>trufflepig-forensics.com</span>
                        <div className={"bubble-list"}>
                            <div className={"bubble"}>A</div>
                            <div className={"bubble"}>AAAA</div>
                            <div className={"bubble"}>MX</div>
                            <div className={"bubble"}>TXT</div>
                        </div>
                        <div className={"bubble-list"}>
                            <div className={"bubble red"}>Critical</div>
                        </div>
                        <div className={"bubble-list"}>
                            <div className={"bubble"}>CT 2</div>
                            <div className={"bubble"}>BS 17</div>
                        </div>
                        <span>Netscaler</span>
                    </div>
                </div>
                <div className={"workspace-content-details pane"}>
                    <h2 className={"heading"}>Details</h2>
                </div>
            </div>
        );
    }
}
