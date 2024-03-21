import React from "react";
import { Api } from "../../../api/api";
import { FullDomain, FullHost, FullPort, FullService } from "../../../api/generated";
import Indicator from "../../../components/indicator";
import OsIcon from "../../../components/os-icon";
import RelationLeftIcon from "../../../svg/relation-left";
import CertaintyIcon from "../components/certainty-icon";
import Domain from "../components/domain";
import { useFilter } from "../components/filter-input";
import IpAddr from "../components/host";
import PortNumber from "../components/port";
import ServiceName from "../components/service";
import SeverityIcon from "../components/severity-icon";
import TagList from "../components/tag-list";
import { StatelessWorkspaceTable, useTable } from "../components/workspace-table";
import { WORKSPACE_CONTEXT } from "../workspace";

export type WorkspaceFindingTableProps = {
    hideUuids: string[];
    onAddDomain?: (domain: FullDomain) => void;
    onAddHost?: (host: FullHost) => void;
    onAddService?: (service: FullService) => void;
    onAddPort?: (port: FullPort) => void;
};

const DATA_TAB = { domains: "Domains", hosts: "Hosts", ports: "Ports", services: "Services" };
export default function WorkspaceFindingTable({
    hideUuids,
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
                        {domains
                            .filter((v) => !hideUuids.includes(v.uuid))
                            .map((domain) => (
                                <div key={domain.uuid} className="workspace-table-row">
                                    <span
                                        className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                        onClick={() => onAddDomain?.(domain)}
                                    >
                                        <RelationLeftIcon />
                                    </span>

                                    <Domain domain={domain} />
                                    <TagList tags={domain.tags} filter={domainFilter} />
                                    <span>{domain.comment}</span>
                                    <div className={"workspace-data-certainty-icon"}>
                                        <SeverityIcon severity={domain.severity} />
                                    </div>
                                    <CertaintyIcon certainty={domain.certainty} />
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
                        {hosts
                            .filter((v) => !hideUuids.includes(v.uuid))
                            .map((host) => (
                                <div key={host.uuid} className="workspace-table-row deleted">
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
                                    <div className={"workspace-data-certainty-icon"}>
                                        <SeverityIcon severity={host.severity} />
                                    </div>
                                    <CertaintyIcon certainty={host.certainty} />
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
                        {ports
                            .filter((v) => !hideUuids.includes(v.uuid))
                            .map((port) => (
                                <div key={port.uuid} className="workspace-table-row">
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
                                    <div className={"workspace-data-certainty-icon"}>
                                        <SeverityIcon severity={port.severity} />
                                    </div>
                                    <CertaintyIcon certainty={port.certainty} />
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
                        {services
                            .filter((v) => !hideUuids.includes(v.uuid))
                            .map((service) => (
                                <div key={service.uuid} className="workspace-table-row">
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
                                                !(
                                                    service.protocols &&
                                                    Object.values(service.protocols).find(Boolean)?.raw
                                                )
                                            }
                                        />
                                    </span>
                                    <span>
                                        <Indicator
                                            off={
                                                !(
                                                    service.protocols &&
                                                    Object.values(service.protocols).find(Boolean)?.tls
                                                )
                                            }
                                        />
                                    </span>
                                    <TagList tags={service.tags} filter={serviceFilter} />
                                    <span>{service.comment}</span>
                                    <div className={"workspace-data-certainty-icon"}>
                                        <SeverityIcon severity={service.severity} />
                                    </div>
                                    <CertaintyIcon certainty={service.certainty} />
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
                        key={key}
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
