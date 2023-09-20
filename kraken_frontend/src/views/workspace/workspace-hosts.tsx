import React from "react";
import { FullWorkspace, OsType, SimpleHost } from "../../api/generated";
import "../../styling/workspace-hosts.css";
import Input from "../../components/input";
import { Api } from "../../api/api";
import { toast } from "react-toastify";
import AnonymousIcon from "../../svg/anonymous";
import TuxIcon from "../../svg/tux";
import AppleIcon from "../../svg/apple";
import WindowsIcon from "../../svg/windows";
import FreeBSDIcon from "../../svg/freebsd";
import AndroidIcon from "../../svg/android";
import { getOsIcon } from "../../utils/helper";
import { ROUTES } from "../../routes";

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
        (await Api.workspaces.hosts.all(this.props.workspace.uuid)).match(
            (hosts) => this.setState({ hosts: hosts.hosts }),
            (err) => toast.error(err.message)
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
                        onChange={(v) => this.setState({ searchTerm: v })}
                    />
                </div>
                {this.state.hosts.map((x) => {
                    return (
                        <div
                            key={x.uuid}
                            className={"workspace-hosts-host pane"}
                            onClick={() => {
                                ROUTES.WORKSPACE_SINGLE_HOST.visit({
                                    w_uuid: this.props.workspace.uuid,
                                    h_uuid: x.uuid,
                                });
                            }}
                        >
                            {getOsIcon(x.osType)}
                            <div className={"workspace-hosts-host-info"}>
                                <h2 className={"sub-heading"}>{x.ipAddr}</h2>
                                <span>{x.comment}</span>
                            </div>
                        </div>
                    );
                })}
            </div>
        );
    }
}
