import React from "react";
import { FullWorkspace, SimpleHost } from "../../api/generated";
import "../../styling/workspace-hosts.css";
import Input from "../../components/input";
import { Api } from "../../api/api";
import { toast } from "react-toastify";
import { ROUTES } from "../../routes";
import OsIcon from "../../components/os-icon";

type WorkspaceHostsProps = {
    workspace: FullWorkspace;
};
type WorkspaceHostsState = {
    searchTerm: string;
    hosts: SimpleHost[];
};

export default class WorkspaceHosts extends React.Component<WorkspaceHostsProps, WorkspaceHostsState> {
    constructor(props: WorkspaceHostsProps) {
        super(props);

        this.state = { searchTerm: "", hosts: [] };
    }

    async retrieveHosts() {
        (await Api.workspaces.hosts.all(this.props.workspace.uuid, 1000, 0)).match(
            ({ items }) => this.setState({ hosts: items }),
            (err) => toast.error(err.message),
        );
    }

    componentDidMount() {
        this.retrieveHosts().then();
    }

    render() {
        console.log(this.state.hosts);

        return (
            <div className={"workspace-hosts-container"}>
                <div className={"pane workspace-hosts-search-pane"}>
                    <Input
                        placeholder={"Search host"}
                        value={this.state.searchTerm}
                        onChange={(searchTerm) => this.setState({ searchTerm })}
                    />
                </div>
                <div className={"workspace-hosts-list"}>
                    {this.state.hosts.map((host) => {
                        return (
                            <div
                                key={host.uuid}
                                className={"workspace-hosts-host pane"}
                                onClick={() => {
                                    ROUTES.WORKSPACE_SINGLE_HOST.visit({
                                        w_uuid: this.props.workspace.uuid,
                                        h_uuid: host.uuid,
                                    });
                                }}
                            >
                                <OsIcon os={host.osType} />
                                <div className={"workspace-hosts-host-info"}>
                                    <h2 className={"sub-heading"}>{host.ipAddr}</h2>
                                    <span>{host.comment}</span>
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>
        );
    }
}
