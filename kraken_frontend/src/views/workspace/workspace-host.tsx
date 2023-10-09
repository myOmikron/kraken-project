import React from "react";
import "../../styling/workspace-host.css";
import { FullHost, FullWorkspace, SimpleHost } from "../../api/generated";
import { Api, UUID } from "../../api/api";
import { toast } from "react-toastify";
import { ROUTES } from "../../routes";
import Input from "../../components/input";
import OsIcon from "../../components/os-icon";

type WorkspaceProps = {
    workspace: FullWorkspace;
    host_uuid: UUID;
};
type WorkspaceState = {
    selectedTab: "domains" | "ips" | "ports" | "services" | "other";
    host: FullHost | null;
    hostList: Array<SimpleHost>;
    searchTerm: string;
};

export default class WorkspaceHost extends React.Component<WorkspaceProps, WorkspaceState> {
    constructor(props: WorkspaceProps) {
        super(props);

        this.state = { selectedTab: "domains", host: null, hostList: [], searchTerm: "" };
    }

    async getHostList() {
        (await Api.workspaces.hosts.all(this.props.workspace.uuid, 1000, 0)).match(
            ({ items }) => {
                this.setState({ hostList: items.filter(({ uuid }) => uuid !== this.props.host_uuid) });
            },
            (err) => toast.error(err.message)
        );
    }

    async getHost() {
        (await Api.workspaces.hosts.get(this.props.workspace.uuid, this.props.host_uuid)).match(
            (host) => this.setState({ host }),
            (err) => toast.error(err.message)
        );
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceProps>, prevState: Readonly<WorkspaceState>, snapshot?: any) {
        if (prevProps.host_uuid !== this.props.host_uuid) {
            Promise.all([this.getHost(), this.getHostList()]).then();
        }
    }

    componentDidMount() {
        Promise.all([this.getHost(), this.getHostList()]).then();
    }

    render() {
        return (
            <div className={"workspace-host-container"}>
                <div className={"workspace-host-hosts-list"}>
                    <button
                        key={"back"}
                        className={"pane workspace-host-hosts-back"}
                        onClick={() => {
                            ROUTES.WORKSPACE_HOSTS.visit({
                                uuid: this.props.workspace.uuid,
                            });
                        }}
                    >
                        <h2 className={"sub-heading"}>Back</h2>
                    </button>
                    <div className={"pane workspace-host-hosts-search"}>
                        <Input
                            placeholder={"Search host"}
                            value={this.state.searchTerm}
                            onChange={(searchTerm) => this.setState({ searchTerm })}
                        />
                    </div>
                    {this.state.hostList.map((host) => {
                        return (
                            <button
                                key={host.uuid}
                                className={"pane workspace-host-hosts-item"}
                                onClick={() => {
                                    ROUTES.WORKSPACE_SINGLE_HOST.visit({
                                        w_uuid: this.props.workspace.uuid,
                                        h_uuid: host.uuid,
                                    });
                                }}
                            >
                                <OsIcon os={host.osType} />
                                <div className={"workspace-host-hosts-info"}>
                                    <h2 className={"sub-heading"}>{host.ipAddr}</h2>
                                    <span>{host.comment}</span>
                                </div>
                            </button>
                        );
                    })}
                </div>
                <div className={"pane workspace-host-host-container"}>
                    {this.state.host !== null ? (
                        <>
                            <OsIcon os={this.state.host.osType} />
                            <div className={"workspace-host-details"}>
                                <h2 className={"heading"}>Host {this.state.host.ipAddr}</h2>
                                <span>OS: {this.state.host.osType}</span>
                                <span>Comment: {this.state.host.comment}</span>
                            </div>
                        </>
                    ) : (
                        <div>Loading ..</div>
                    )}
                </div>
                <div className={"workspace-host-section-selector"}>
                    <div
                        className={this.state.selectedTab === "domains" ? "pane workspace-host-selected-tab" : "pane"}
                        onClick={() => {
                            this.setState({ selectedTab: "domains" });
                        }}
                    >
                        <h3 className={"heading"}>Domains</h3>
                    </div>
                    <div
                        className={this.state.selectedTab === "ports" ? "pane workspace-host-selected-tab" : "pane"}
                        onClick={() => {
                            this.setState({ selectedTab: "ports" });
                        }}
                    >
                        <h3 className={"heading"}>Ports</h3>
                    </div>
                    <div
                        className={this.state.selectedTab === "services" ? "pane workspace-host-selected-tab" : "pane"}
                        onClick={() => {
                            this.setState({ selectedTab: "services" });
                        }}
                    >
                        <h3 className={"heading"}>Services</h3>
                    </div>
                    <div
                        className={this.state.selectedTab === "other" ? "pane workspace-host-selected-tab" : "pane"}
                        onClick={() => {
                            this.setState({ selectedTab: "other" });
                        }}
                    >
                        <h3 className={"heading"}>Other</h3>
                    </div>
                </div>
                <div className={"workspace-host-content-table"}>
                    <div className={"pane workspace-host-content-row"}>
                        <span>Domain</span>
                        <span>DNS</span>
                        <span>Tags</span>
                        <span>Attacks</span>
                        <span>Comment</span>
                    </div>
                    <div className={"pane workspace-host-content-row"}>
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
                <div className={"workspace-host-content-details pane"}>
                    <h2 className={"heading"}>Details</h2>
                </div>
            </div>
        );
    }
}
