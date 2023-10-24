import React from "react";
import "../../styling/workspace-data.css";
import WorkspaceDataTable, { WorkspaceDataTableProps } from "./workspace-data/workspace-data-table";
import { WorkspaceDataDomains } from "./workspace-data/workspace-data-domains";
import { WorkspaceDataHosts } from "./workspace-data/workspace-data-hosts";
import { WorkspaceDataPorts } from "./workspace-data/workspace-data-ports";
import { WorkspaceDataServices } from "./workspace-data/workspace-data-services";

const TABS = { domains: "Domains", hosts: "Hosts", ports: "Ports", services: "Services", other: "Other" };

type WorkspaceDataProps = {
    /** Workspace uuid */
    workspace: string;
};
type WorkspaceDataState = {
    selectedTab: keyof typeof TABS;
    selected: { type: keyof typeof TABS; uuid: string } | null;
};

export default class WorkspaceData extends React.Component<WorkspaceDataProps, WorkspaceDataState> {
    state: WorkspaceDataState = {
        selectedTab: "domains",
        selected: null,
    };

    render() {
        const { selectedTab } = this.state;
        const table = (() => {
            switch (selectedTab) {
                case "domains":
                    return (
                        <WorkspaceDataDomains
                            workspace={this.props.workspace}
                            onSelect={(uuid) => this.setState({ selected: { type: "domains", uuid } })}
                        />
                    );
                case "hosts":
                    return (
                        <WorkspaceDataHosts
                            workspace={this.props.workspace}
                            onSelect={(uuid) => this.setState({ selected: { type: "hosts", uuid } })}
                        />
                    );
                case "ports":
                    return (
                        <WorkspaceDataPorts
                            workspace={this.props.workspace}
                            onSelect={(uuid) => this.setState({ selected: { type: "ports", uuid } })}
                        />
                    );
                case "services":
                    return (
                        <WorkspaceDataServices
                            workspace={this.props.workspace}
                            onSelect={(uuid) => this.setState({ selected: { type: "services", uuid } })}
                        />
                    );
                default:
                    return "Unimplemented";
            }
        })();
        return (
            <div className={"workspace-data-container"}>
                <div className={"workspace-data-selector"}>
                    {Object.entries(TABS).map(([key, displayName]) => (
                        <div
                            className={"pane" + (this.state.selectedTab !== key ? "" : " workspace-data-selected-tab")}
                            onClick={() => this.setState({ selectedTab: key as keyof typeof TABS })}
                        >
                            <h3 className={"heading"}>{displayName}</h3>
                        </div>
                    ))}
                </div>
                {table}
                <div className={"workspace-data-details pane"}>
                    <h2 className={"heading"}>Details</h2>
                </div>
            </div>
        );
    }
}

export const WorkspaceDataOther = (props: WorkspaceDataTableProps<never>) => WorkspaceDataTable(props);
