import React from "react";
import "../../styling/workspace-host.css";
import { FullHost, FullWorkspace, SimpleDomain, SimpleHost, SimplePort, SimpleService } from "../../api/generated";
import { Api, UUID } from "../../api/api";
import { toast } from "react-toastify";
import { ROUTES } from "../../routes";
import Input from "../../components/input";
import OsIcon from "../../components/os-icon";
import ArrowLeftIcon from "../../svg/arrow-left";

import { WorkspaceHostDomains } from "./workspace-host/workspace-host-domains";
import { WorkspaceHostPorts } from "./workspace-host/workspace-host-ports";
import { WorkspaceHostServices } from "./workspace-host/workspace-host-services";
import { WORKSPACE_CONTEXT } from "./workspace";
import { handleApiError } from "../../utils/helper";
import TagList from "./components/tag-list";

const TABS = { domains: "Domains", ports: "Ports", services: "Services", other: "Other" };

type WorkspaceProps = {
    uuid: UUID;
};
type WorkspaceState = {
    selectedTab: keyof typeof TABS;
    selected: { type: keyof typeof TABS; uuid: string } | null;
    host: FullHost | null;
    domains: Array<SimpleDomain>;
    ports: Array<SimplePort>;
    services: Array<SimpleService>;
    hostList: Array<FullHost>;
    searchTerm: string;
    limit: number;
    offset: number;
    totalDomains: number;
};

export default class WorkspaceHost extends React.Component<WorkspaceProps, WorkspaceState> {
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: WorkspaceProps) {
        super(props);

        this.state = {
            selectedTab: "domains",
            selected: null,
            host: null,
            hostList: [],
            domains: [],
            ports: [],
            services: [],
            searchTerm: "",
            limit: 5,
            offset: 0,
            totalDomains: 0,
        };
    }

    async getHostList() {
        await Api.workspaces.hosts.all(this.context.workspace.uuid, 1000, 0).then(
            handleApiError(({ items }) => {
                this.setState({ hostList: items.filter(({ uuid }) => uuid !== this.props.uuid) });
            })
        );
    }

    async getHost() {
        await Api.workspaces.hosts
            .get(this.context.workspace.uuid, this.props.uuid)
            .then(handleApiError((host) => this.setState({ host })));
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceProps>, prevState: Readonly<WorkspaceState>, snapshot?: any) {
        if (prevProps.uuid !== this.props.uuid) {
            Promise.all([this.getHost(), this.getHostList()]).then();
        }
    }

    componentDidMount() {
        Promise.all([this.getHost(), this.getHostList()]).then();
    }

    render() {
        const { selectedTab } = this.state;
        const { host } = this.state;
        const table = (() => {
            switch (selectedTab) {
                case "domains":
                    return (
                        <WorkspaceHostDomains
                            onSelect={(uuid) => this.setState({ selected: { type: "domains", uuid } })}
                            host={this.state.host}
                        />
                    );
                case "ports":
                    return (
                        <WorkspaceHostPorts
                            onSelect={(uuid) => this.setState({ selected: { type: "ports", uuid } })}
                            host={this.state.host}
                        />
                    );
                case "services":
                    return (
                        <WorkspaceHostServices
                            onSelect={(uuid) => this.setState({ selected: { type: "services", uuid } })}
                            host={this.state.host}
                        />
                    );
                default:
                    return "Unimplemented";
            }
        })();
        return (
            <div className={"workspace-host-container"}>
                <div className={"workspace-host-hosts-list"}>
                    <div className={"workspace-host-hosts-list-header"}>
                        <div className={"pane workspace-host-hosts-search"}>
                            <ArrowLeftIcon
                                key={"back"}
                                onClick={() => {
                                    ROUTES.WORKSPACE_HOSTS.visit({
                                        uuid: this.context.workspace.uuid,
                                    });
                                }}
                            />

                            <Input
                                className={"workspace-host-search-bar"}
                                placeholder={"Search host"}
                                value={this.state.searchTerm}
                                onChange={(searchTerm) => this.setState({ searchTerm })}
                            />
                        </div>
                    </div>
                    <div className={"workspace-host-hosts-list-entries"}>
                        {this.state.hostList.map((host) => {
                            return (
                                <button
                                    key={host.uuid}
                                    className={"pane workspace-host-hosts-item"}
                                    onClick={() => {
                                        ROUTES.WORKSPACE_SINGLE_HOST.visit({
                                            w_uuid: this.context.workspace.uuid,
                                            h_uuid: host.uuid,
                                        });
                                    }}
                                >
                                    <OsIcon os={host.osType} />
                                    <div className={"workspace-host-hosts-info"}>
                                        <h2 className={"sub-heading"}>{host.ipAddr}</h2>
                                        <span className="workspace-host-comment-overflow">{host.comment}</span>
                                    </div>
                                </button>
                            );
                        })}
                    </div>
                </div>
                <div className={"pane workspace-host-host-container"}>
                    {this.state.host !== null ? (
                        <>
                            <OsIcon os={this.state.host.osType} />
                            <div className={"workspace-host-details"}>
                                <h2 className={"heading"}>Host {this.state.host.ipAddr}</h2>
                                <span>OS: {this.state.host.osType}</span>
                                <span>Comment: {this.state.host.comment}</span>
                                <TagList tags={this.state.host.tags} />
                            </div>
                        </>
                    ) : (
                        <div>Loading ..</div>
                    )}
                </div>

                <div className={"workspace-host-section-selector"}>
                    {Object.entries(TABS).map(([key, displayName]) => (
                        <div
                            className={"pane" + (this.state.selectedTab !== key ? "" : " workspace-host-selected-tab")}
                            onClick={() => this.setState({ selectedTab: key as keyof typeof TABS })}
                        >
                            <h3 className={"heading"}>{displayName}</h3>
                        </div>
                    ))}
                </div>
                {table}

                <div className={"workspace-host-content-details pane"}>
                    <h2 className={"heading"}>Details</h2>
                </div>
            </div>
        );
    }
}
