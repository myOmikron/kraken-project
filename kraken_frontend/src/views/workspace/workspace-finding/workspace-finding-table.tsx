import React from "react";
import { Api } from "../../../api/api";
import { FullDomain, FullHost, FullPort, FullService } from "../../../api/generated";
import Indicator from "../../../components/indicator";
import OsIcon from "../../../components/os-icon";
import RelationLeftIcon from "../../../svg/relation-left";
import Domain from "../components/domain";
import { useFilter } from "../components/filter-input";
import IpAddr from "../components/host";
import PortNumber from "../components/port";
import ServiceName from "../components/service";
import TagList from "../components/tag-list";
import { StatelessWorkspaceTable, useTable } from "../components/workspace-table";
import { WORKSPACE_CONTEXT } from "../workspace";
import { CertaintyIcon } from "../workspace-data";

export type WorkspaceFindingTableProps = {
    onAddDomain?: (domain: FullDomain) => void;
    onAddHost?: (host: FullHost) => void;
    onAddService?: (service: FullService) => void;
    onAddPort?: (port: FullPort) => void;
};

const DATA_TAB = { domains: "Domains", hosts: "Hosts", ports: "Ports", services: "Services" };
export default function WorkspaceFindingTable({
    onAddDomain,
    onAddHost,
    onAddService,
    onAddPort,
}: WorkspaceFindingTableProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [dataTab, setDataTab] = React.useState<keyof typeof DATA_TAB>("hosts");

    const domainFilter = useFilter(workspace, "domain");
    const hostFilter = useFilter(workspace, "host");
    const portFilter = useFilter(workspace, "port");
    const serviceFilter = useFilter(workspace, "service");

    const { items: domains, ...domainsTable } = useTable<FullDomain>(
        (limit, offset) =>
            Api.workspaces.domains.all(workspace, limit, offset, {
                domainFilter: domainFilter.applied,
            }),
        [workspace, domainFilter.applied],
    );
    const { items: hosts, ...hostsTable } = useTable<FullHost>(
        (limit, offset) =>
            Api.workspaces.hosts.all(workspace, limit, offset, {
                hostFilter: hostFilter.applied,
            }),
        [workspace, hostFilter.applied],
    );
    const { items: ports, ...portsTable } = useTable<FullPort>(
        (limit, offset) =>
            Api.workspaces.ports.all(workspace, limit, offset, {
                portFilter: portFilter.applied,
            }),
        [workspace, portFilter.applied],
    );
    const { items: services, ...servicesTable } = useTable<FullService>(
        (limit, offset) =>
            Api.workspaces.services.all(workspace, limit, offset, {
                serviceFilter: serviceFilter.applied,
            }),
        [workspace, serviceFilter.applied],
    );

    // Jump to first page if filter changed
    React.useEffect(() => domainsTable.setOffset(0), [domainFilter.applied]);
    React.useEffect(() => hostsTable.setOffset(0), [hostFilter.applied]);
    React.useEffect(() => portsTable.setOffset(0), [portFilter.applied]);
    React.useEffect(() => servicesTable.setOffset(0), [serviceFilter.applied]);

    const tableElement = (() => {
        switch (dataTab) {
            case "domains":
                return (
                    <StatelessWorkspaceTable
                        key={"domain-table"}
                        {...domainsTable}
                        columnsTemplate={"0.3fr 1fr 1fr 1fr 3.5em 4em"}
                        filter={domainFilter}
                        solidBackground={true}
                    >
                        <div className={"workspace-table-header"}>
                            <span className="workspace-data-certainty-icon workspace-finding-selection-arrow">
                                <RelationLeftIcon />
                            </span>
                            <span>Domain</span>
                            <span>Tags</span>
                            <span>Comment</span>
                            <span>Severity</span>
                            <span>Certainty</span>
                        </div>
                        {/*TODO filter items that are not in findings already*/}
                        {domains.map((domain) => (
                            <div className="workspace-table-row">
                                <span
                                    className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                    onClick={() => onAddDomain?.(domain)}
                                >
                                    <RelationLeftIcon />
                                </span>

                                <Domain domain={domain} />
                                <TagList tags={domain.tags} filter={domainFilter} />
                                <span>{domain.comment}</span>
                                <span className="workspace-data-certainty-icon icon"></span>
                                {domain.certainty === "Unverified"
                                    ? CertaintyIcon({ certaintyType: "Unverified" })
                                    : CertaintyIcon({ certaintyType: "Verified" })}
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "hosts":
                return (
                    <StatelessWorkspaceTable
                        key={"host-table"}
                        {...hostsTable}
                        columnsTemplate={"0.3fr 30ch 2em 1fr 1fr 3.5em 4em"}
                        filter={hostFilter}
                        solidBackground={true}
                    >
                        <div className={"workspace-table-header"}>
                            <span className="workspace-data-certainty-icon workspace-finding-selection-arrow">
                                <RelationLeftIcon />
                            </span>
                            <span>IP</span>
                            <span>OS</span>
                            <span>Tags</span>
                            <span>Comment</span>
                            <span>Severity</span>
                            <span>Certainty</span>
                        </div>
                        {/*TODO filter items that are not in findings already*/}
                        {hosts.map((host) => (
                            <div className="workspace-table-row deleted">
                                <span
                                    className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                    onClick={() => onAddHost?.(host)}
                                >
                                    <RelationLeftIcon />
                                </span>
                                <IpAddr host={host} />
                                <OsIcon tooltip os={host.osType} size="2em" />
                                <TagList tags={host.tags} filter={hostFilter} />
                                <span>{host.comment}</span>
                                <span className="workspace-data-certainty-icon icon"></span>
                                {host.certainty === "Verified"
                                    ? CertaintyIcon({ certaintyType: "Verified" })
                                    : host.certainty === "Historical"
                                      ? CertaintyIcon({ certaintyType: "Historical" })
                                      : CertaintyIcon({ certaintyType: "SupposedTo" })}
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "ports":
                return (
                    <StatelessWorkspaceTable
                        key={"port-table"}
                        {...portsTable}
                        columnsTemplate={"0.3fr 5ch 3.75em 30ch 1fr 1fr 3.5em 4em"}
                        filter={portFilter}
                        solidBackground={true}
                    >
                        <div className={"workspace-table-header"}>
                            <span className="workspace-data-certainty-icon workspace-finding-selection-arrow">
                                <RelationLeftIcon />
                            </span>
                            <span>Port</span>
                            <span>Protocol</span>
                            <span>IP</span>
                            <span>Tags</span>
                            <span>Comment</span>
                            <span>Severity</span>
                            <span>Certainty</span>
                        </div>
                        {/*TODO filter items that are not in findings already*/}
                        {ports.map((port) => (
                            <div className="workspace-table-row">
                                <span
                                    className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                    onClick={() => onAddPort?.(port)}
                                >
                                    <RelationLeftIcon />
                                </span>
                                <PortNumber port={port} />
                                <span>{port.protocol.toUpperCase()}</span>
                                <IpAddr host={port.host} />
                                <TagList tags={port.tags} filter={portFilter} />
                                <span>{port.comment}</span>
                                <span className="workspace-data-certainty-icon icon"></span>
                                {port.certainty === "Verified"
                                    ? CertaintyIcon({ certaintyType: "Verified" })
                                    : port.certainty === "Historical"
                                      ? CertaintyIcon({ certaintyType: "Historical" })
                                      : CertaintyIcon({ certaintyType: "SupposedTo" })}
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "services":
                return (
                    <StatelessWorkspaceTable
                        key={"service-table"}
                        {...servicesTable}
                        columnsTemplate={"0.3fr 0.8fr 30ch 5ch 3.75em 2em 2em 1fr 1fr 3.5em 4em"}
                        filter={serviceFilter}
                        solidBackground={true}
                    >
                        <div className={"workspace-table-header"}>
                            <span className="workspace-data-certainty-icon workspace-finding-selection-arrow">
                                <RelationLeftIcon />
                            </span>
                            <span>Service</span>
                            <span>IP</span>
                            <span>Port</span>
                            <span>Protocol</span>
                            <span>Raw</span>
                            <span>TLS</span>
                            <span>Tags</span>
                            <span>Comment</span>
                            <span>Severity</span>
                            <span>Certainty</span>
                        </div>
                        {/*TODO filter items that are not in findings already*/}
                        {services.map((service) => (
                            <div className="workspace-table-row">
                                <span
                                    className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                    onClick={() => onAddService?.(service)}
                                >
                                    <RelationLeftIcon />
                                </span>
                                <ServiceName service={service} />
                                <IpAddr host={service.host} />
                                {service.port ? <PortNumber port={service.port} /> : <span></span>}
                                <span>{service.port?.protocol?.toUpperCase()}</span>
                                <span>
                                    <Indicator
                                        off={
                                            !(service.protocols && Object.values(service.protocols).find(Boolean)?.raw)
                                        }
                                    />
                                </span>
                                <span>
                                    <Indicator
                                        off={
                                            !(service.protocols && Object.values(service.protocols).find(Boolean)?.tls)
                                        }
                                    />
                                </span>
                                <TagList tags={service.tags} filter={serviceFilter} />
                                <span>{service.comment}</span>
                                <span className="workspace-data-certainty-icon icon"></span>
                                {service.certainty === "Historical"
                                    ? CertaintyIcon({ certaintyType: "Historical" })
                                    : service.certainty === "SupposedTo"
                                      ? CertaintyIcon({ certaintyType: "SupposedTo" })
                                      : service.certainty === "UnknownService"
                                        ? CertaintyIcon({ certaintyType: "UnknownService" })
                                        : service.certainty === "MaybeVerified"
                                          ? CertaintyIcon({ certaintyType: "MaybeVerified" })
                                          : CertaintyIcon({ certaintyType: "DefinitelyVerified" })}
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            default:
                return "Unimplemented";
        }
    })();

    return (
        <div className="workspace-data-table">
            <div className="tabs-selector-container">
                {Object.entries(DATA_TAB).map(([key, displayName]) => (
                    <div
                        className={`tabs ${dataTab !== key ? "" : " selected-tab"}`}
                        onClick={() => setDataTab(key as keyof typeof DATA_TAB)}
                    >
                        <h3 className={"heading"}>{displayName}</h3>
                    </div>
                ))}
            </div>
            {tableElement}
        </div>
    );
}
