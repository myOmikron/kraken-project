import React from "react";
import { handleApiError } from "../../utils/helper";
import { Api } from "../../api/api";
import "../../styling/workspace-data.css";
import { Result } from "../../utils/result";
import { ApiError } from "../../api/error";
import { SimpleDomain, SimpleHost, SimplePort, SimpleService } from "../../api/generated";
import SelectMenu, { selectStyles } from "../../components/select-menu";
import Select from "react-select";

const TABS = { domains: "Domains", hosts: "Hosts", ports: "Ports", services: "Services", other: "Other" };

type WorkspaceDataProps = {
    /** Workspace uuid */
    workspace: string;
};
type WorkspaceDataState = {
    selectedTab: keyof typeof TABS;
};

export default class WorkspaceData extends React.Component<WorkspaceDataProps, WorkspaceDataState> {
    state: WorkspaceDataState = {
        selectedTab: "domains",
    };

    render() {
        const { workspace } = this.props;
        const { selectedTab } = this.state;
        const table = (function () {
            switch (selectedTab) {
                case "domains":
                    return (
                        <WorkspaceDataDomains
                            query={(limit, offset) => Api.workspaces.domains.all(workspace, limit, offset)}
                            queryDeps={[workspace]}
                        >
                            <div className={"workspace-data-table-header pane"}>
                                <span>Name</span>
                                <span>Comment</span>
                            </div>
                            {(domain) => (
                                <div className={"workspace-data-table-row pane"}>
                                    <span>{domain.domain}</span>
                                    <span>{domain.comment}</span>
                                </div>
                            )}
                        </WorkspaceDataDomains>
                    );
                case "hosts":
                    return (
                        <WorkspaceDataHosts
                            query={(limit, offset) => Api.workspaces.hosts.all(workspace, limit, offset)}
                            queryDeps={[workspace]}
                        >
                            <div className={"workspace-data-table-header pane"}>
                                <span>IP</span>
                                <span>Comment</span>
                            </div>
                            {(host) => (
                                <div className={"workspace-data-table-row pane"}>
                                    <span>{host.ipAddr}</span>
                                    <span>{host.comment}</span>
                                </div>
                            )}
                        </WorkspaceDataHosts>
                    );
                case "ports":
                    return (
                        <WorkspaceDataPorts
                            query={(limit, offset) => Api.workspaces.ports.all(workspace, limit, offset)}
                            queryDeps={[workspace]}
                        >
                            <div className={"workspace-data-table-header pane"}>
                                <span>Number</span>
                                <span>Host</span>
                                <span>Comment</span>
                            </div>
                            {(port) => (
                                <div className={"workspace-data-table-row pane"}>
                                    <span>{port.port}</span>
                                    <span>{port.host}</span>
                                    <span>{port.comment}</span>
                                </div>
                            )}
                        </WorkspaceDataPorts>
                    );
                case "services":
                    return (
                        <WorkspaceDataServices
                            query={(limit, offset) => Api.workspaces.services.all(workspace, limit, offset)}
                            queryDeps={[workspace]}
                        >
                            <div className={"workspace-data-table-header pane"}>
                                <span>Name</span>
                                <span>Host</span>
                                <span>Number</span>
                                <span>Comment</span>
                            </div>
                            {(service) => (
                                <div className={"workspace-data-table-row pane"}>
                                    <span>{service.name}</span>
                                    <span>{service.host}</span>
                                    <span>{service.port}</span>
                                    <span>{service.comment}</span>
                                </div>
                            )}
                        </WorkspaceDataServices>
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

export const WorkspaceDataDomains = (props: WorkspaceDataTableProps<SimpleDomain>) => WorkspaceDataTable(props);
export const WorkspaceDataHosts = (props: WorkspaceDataTableProps<SimpleHost>) => WorkspaceDataTable(props);
export const WorkspaceDataPorts = (props: WorkspaceDataTableProps<SimplePort>) => WorkspaceDataTable(props);
export const WorkspaceDataServices = (props: WorkspaceDataTableProps<SimpleService>) => WorkspaceDataTable(props);
export const WorkspaceDataOther = (props: WorkspaceDataTableProps<never>) => WorkspaceDataTable(props);

type WorkspaceDataTableProps<T> = {
    query: (limit: number, offset: number) => Promise<Result<GenericPage<T>, ApiError>>;
    queryDeps?: React.DependencyList;
    children: [React.ReactNode, (item: T) => React.ReactNode];
};
type GenericPage<T> = {
    items: Array<T>;
    limit: number;
    offset: number;
    total: number;
};
function WorkspaceDataTable<T>(props: WorkspaceDataTableProps<T>) {
    const {
        query,
        queryDeps,
        children: [header, renderItem],
    } = props;

    const [limit, setLimit] = React.useState(10);
    const [page, setRawPage] = React.useState(1);
    const [total, setTotal] = React.useState(0);
    const [items, setItems] = React.useState<Array<T>>([]);

    React.useEffect(() => {
        query(limit, limit * (page - 1)).then(
            handleApiError(({ items, total }) => {
                setItems(items);
                setTotal(total);
            }),
        );
    }, [limit, page, ...(queryDeps || [])]);

    const lastPage = Math.ceil(total / limit) || 1;
    function setPage(page: number) {
        if (page <= 0) {
            setRawPage(1);
        } else if (page > lastPage) {
            setRawPage(lastPage);
        } else {
            setRawPage(page);
        }
    }

    return (
        <>
            {header}
            <div className={"workspace-data-table-body"}>{items.map(renderItem)}</div>
            <div className={"workspace-data-table-controls"}>
                <Select<{ label: string; value: number }, false>
                    menuPlacement={"auto"}
                    value={{ value: 10, label: String(limit) }}
                    options={[10, 20, 30, 40, 50].map((n) => ({ value: n, label: String(n) }))}
                    onChange={(value) => {
                        setLimit(value?.value || limit);
                    }}
                    styles={selectStyles("default")}
                />
                <button className={"button"} disabled={page === 1} onClick={() => setPage(1)}>
                    First
                </button>
                <button className={"button"} disabled={page === 1} onClick={() => setPage(page - 1)}>
                    Prev
                </button>
                <form
                    onSubmit={(event) => {
                        event.preventDefault();
                        const input = event.currentTarget["page"] as HTMLInputElement;
                        const page = Number(input.value);
                        input.value = "";
                        setPage(page);
                    }}
                >
                    <input
                        className={"input"}
                        title={"Page number"}
                        name={"page"}
                        pattern={"[1-9]\\d*"}
                        placeholder={`Page ${page} of ${lastPage}`}
                    />
                </form>
                <button className={"button"} disabled={page === lastPage} onClick={() => setPage(page + 1)}>
                    Next
                </button>
                <button className={"button"} disabled={page === lastPage} onClick={() => setPage(Infinity)}>
                    Last
                </button>
            </div>
        </>
    );
}
