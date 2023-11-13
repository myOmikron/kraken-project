import React from "react";
import { FullHost, FullWorkspace, SimpleHost } from "../../api/generated";
import "../../styling/workspace-hosts.css";
import Input from "../../components/input";
import { Api } from "../../api/api";
import { toast } from "react-toastify";
import { ROUTES } from "../../routes";
import OsIcon from "../../components/os-icon";
import ArrowFirstIcon from "../../svg/arrow-first";
import ArrowLeftIcon from "../../svg/arrow-left";
import ArrowRightIcon from "../../svg/arrow-right";
import ArrowLastIcon from "../../svg/arrow-last";
import Select from "react-select";
import { selectStyles } from "../../components/select-menu";

type WorkspaceHostsProps = {
    workspace: FullWorkspace;
};
type WorkspaceHostsState = {
    searchTerm: string;
    hosts: SimpleHost[];
    total: number;
    limit: number;
    offset: number;
};

export default class WorkspaceHosts extends React.Component<WorkspaceHostsProps, WorkspaceHostsState> {
    constructor(props: WorkspaceHostsProps) {
        super(props);

        this.state = { searchTerm: "", hosts: [], total: 0, offset: 0, limit: 28 };
    }

    async retrieveHosts() {
        (await Api.workspaces.hosts.all(this.props.workspace.uuid, this.state.limit, this.state.offset)).match(
            ({ items, total }) => this.setState({ hosts: items, total }),
            (err) => toast.error(err.message)
        );
    }

    componentDidMount() {
        this.retrieveHosts().then();
    }

    componentDidUpdate(
        prevProps: Readonly<WorkspaceHostsProps>,
        prevState: Readonly<WorkspaceHostsState>,
        snapshot?: any
    ) {
        if (prevState.offset !== this.state.offset || this.state.limit !== prevState.limit) {
            this.retrieveHosts().then();
        }
    }

    render() {
        const { offset, limit, total } = this.state;
        const lastOffset = Math.floor(total / limit) * limit;
        const setOffset = (offset: number) => {
            if (offset < 0) {
                this.setState({ offset: 0 });
            } else if (offset > lastOffset) {
                this.setState({ offset: lastOffset });
            } else {
                this.setState({ offset });
            }
        };

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
                <div className={"workspace-table-controls"}>
                    <div className={"workspace-table-controls-button-container"}>
                        <button
                            className={"workspace-table-button"}
                            disabled={offset === 0}
                            onClick={() => setOffset(0)}
                        >
                            <ArrowFirstIcon />
                        </button>
                        <button
                            className={"workspace-table-button"}
                            disabled={offset === 0}
                            onClick={() => setOffset(offset - limit)}
                        >
                            <ArrowLeftIcon />
                        </button>
                        <button
                            className={"workspace-table-button"}
                            disabled={offset === lastOffset}
                            onClick={() => setOffset(offset + limit)}
                        >
                            <ArrowRightIcon />
                        </button>
                        <button
                            className={"workspace-table-button"}
                            disabled={offset === lastOffset}
                            onClick={() => setOffset(lastOffset)}
                        >
                            <ArrowLastIcon />
                        </button>
                    </div>
                    <div className={"workspace-table-controls-page-container"}>
                        <span>{`${offset + 1} - ${Math.min(total, offset + limit + 1)} of ${total}`}</span>
                        <Select<{ label: string; value: number }, false>
                            menuPlacement={"auto"}
                            value={{ value: limit, label: String(limit) }}
                            options={[10, 20, 30, 40, 50, 60, 70, 80, 90, 100].map((n) => ({
                                value: n,
                                label: String(n),
                            }))}
                            onChange={(value) => {
                                this.setState({ limit: value?.value || limit });
                            }}
                            styles={selectStyles("default")}
                        />
                    </div>
                </div>
            </div>
        );
    }
}
