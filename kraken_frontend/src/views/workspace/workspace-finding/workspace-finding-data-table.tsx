import React, { forwardRef, useImperativeHandle } from "react";
import { Api } from "../../../api/api";
import { AggregationType, FullDomain, FullHost, FullPort, FullService } from "../../../api/generated";
import { FullHttpService } from "../../../api/generated/models/FullHttpService";
import Indicator from "../../../components/indicator";
import OsIcon from "../../../components/os-icon";
import SelectableText from "../../../components/selectable-text";
import RelationLeftIcon from "../../../svg/relation-left";
import CertaintyIcon from "../components/certainty-icon";
import { DataTabsSelector, useDataTabs } from "../components/data-tabs";
import Domain from "../components/domain";
import { UseFilterReturn, useFilter } from "../components/filter-input";
import IpAddr from "../components/host";
import HttpServiceName from "../components/http-service";
import PortNumber from "../components/port";
import ServiceName from "../components/service";
import { Severity } from "../components/severity-icon";
import TagList from "../components/tag-list";
import { StatelessWorkspaceTable, useTable } from "../components/workspace-table";
import { WORKSPACE_CONTEXT } from "../workspace";

export type WorkspaceFindingDataTableProps = {
    hideUuids: string[];
    onAddDomains?: (domains: FullDomain[]) => void;
    onAddHosts?: (hosts: FullHost[]) => void;
    onAddServices?: (services: FullService[]) => void;
    onAddHttpServices?: (services: FullHttpService[]) => void;
    onAddPorts?: (ports: FullPort[]) => void;
};

export type WorkspaceFindingDataTableRef = {
    addFilterColumn(column: string, value: string, negate: boolean): void;
};

export const WorkspaceFindingDataTable = forwardRef<WorkspaceFindingDataTableRef, WorkspaceFindingDataTableProps>(
    ({ hideUuids, onAddDomains, onAddHosts, onAddServices, onAddPorts, onAddHttpServices }, ref) => {
        const {
            workspace: { uuid: workspace },
        } = React.useContext(WORKSPACE_CONTEXT);

        const [dataTab, setDataTab] = useDataTabs();

        const domainFilter = useFilter(workspace, "domain");
        const hostFilter = useFilter(workspace, "host");
        const portFilter = useFilter(workspace, "port");
        const serviceFilter = useFilter(workspace, "service");
        const httpServiceFilter = useFilter(workspace, "httpService");

        useImperativeHandle(ref, () => ({
            addFilterColumn(column: string, value: string, negate: boolean) {
                let filter: UseFilterReturn;
                switch (dataTab) {
                    case "Domain":
                        filter = domainFilter;
                        break;
                    case "Host":
                        filter = hostFilter;
                        break;
                    case "Port":
                        filter = portFilter;
                        break;
                    case "Service":
                        filter = serviceFilter;
                        break;
                    case "HttpService":
                        filter = httpServiceFilter;
                        break;
                }
                filter.addColumn(column, value, negate);
            },
        }));

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
        const { items: httpServices, ...httpServicesTable } = useTable<FullHttpService>(
            (limit, offset) =>
                Api.workspaces.httpServices.all(workspace, limit, offset, {
                    httpServiceFilter: httpServiceFilter.applied,
                }),
            [workspace, httpServiceFilter.applied],
        );

        // Jump to first page if filter changed
        React.useEffect(() => domainsTable.setOffset(0), [domainFilter.applied]);
        React.useEffect(() => hostsTable.setOffset(0), [hostFilter.applied]);
        React.useEffect(() => portsTable.setOffset(0), [portFilter.applied]);
        React.useEffect(() => servicesTable.setOffset(0), [serviceFilter.applied]);
        React.useEffect(() => httpServicesTable.setOffset(0), [httpServiceFilter.applied]);

        const tableElement = (() => {
            switch (dataTab) {
                case AggregationType.Domain:
                    return (
                        <StatelessWorkspaceTable
                            key={"domain-table"}
                            {...domainsTable}
                            columnsTemplate={"0.3fr 1fr 1fr 1fr 3.5em 4em"}
                            filter={domainFilter}
                            solidBackground={true}
                        >
                            <div className={"workspace-table-header"}>
                                <span
                                    className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                    onClick={() => onAddDomains?.(domains.filter((v) => !hideUuids.includes(v.uuid)))}
                                >
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
                                            onClick={() => onAddDomains?.([domain])}
                                        >
                                            <RelationLeftIcon />
                                        </span>

                                        <Domain domain={domain} />
                                        <TagList tags={domain.tags} filter={domainFilter} />
                                        <span>{domain.comment}</span>
                                        <Severity
                                            severity={domain.severity}
                                            dataType={"Domain"}
                                            uuid={domain.uuid}
                                            workspace={workspace}
                                        />
                                        <CertaintyIcon certainty={domain.certainty} />
                                    </div>
                                ))}
                        </StatelessWorkspaceTable>
                    );
                case AggregationType.Host:
                    return (
                        <StatelessWorkspaceTable
                            key={"host-table"}
                            {...hostsTable}
                            columnsTemplate={"0.3fr 25ch 2em 1fr 1fr 3.5em 4em"}
                            filter={hostFilter}
                            solidBackground={true}
                        >
                            <div className={"workspace-table-header"}>
                                <span
                                    className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                    onClick={() => onAddHosts?.(hosts.filter((v) => !hideUuids.includes(v.uuid)))}
                                >
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
                                            onClick={() => onAddHosts?.([host])}
                                        >
                                            <RelationLeftIcon />
                                        </span>
                                        <IpAddr host={host} />
                                        <OsIcon tooltip os={host.osType} size="2em" />
                                        <TagList tags={host.tags} filter={hostFilter} />
                                        <span>{host.comment}</span>
                                        <Severity
                                            severity={host.severity}
                                            dataType={"Host"}
                                            uuid={host.uuid}
                                            workspace={workspace}
                                        />
                                        <CertaintyIcon certainty={host.certainty} />
                                    </div>
                                ))}
                        </StatelessWorkspaceTable>
                    );
                case AggregationType.Port:
                    return (
                        <StatelessWorkspaceTable
                            key={"port-table"}
                            {...portsTable}
                            columnsTemplate={"0.3fr 5ch 3.75em 20ch 1fr 1fr 3.5em 4em"}
                            filter={portFilter}
                            solidBackground={true}
                        >
                            <div className={"workspace-table-header"}>
                                <span
                                    className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                    onClick={() => onAddPorts?.(ports.filter((v) => !hideUuids.includes(v.uuid)))}
                                >
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
                                            onClick={() => onAddPorts?.([port])}
                                        >
                                            <RelationLeftIcon />
                                        </span>
                                        <PortNumber port={port} />
                                        <span>{port.protocol.toUpperCase()}</span>
                                        <IpAddr host={port.host} />
                                        <TagList tags={port.tags} filter={portFilter} />
                                        <span>{port.comment}</span>
                                        <Severity
                                            severity={port.severity}
                                            dataType={"Port"}
                                            uuid={port.uuid}
                                            workspace={workspace}
                                        />
                                        <CertaintyIcon certainty={port.certainty} />
                                    </div>
                                ))}
                        </StatelessWorkspaceTable>
                    );
                case AggregationType.Service:
                    return (
                        <StatelessWorkspaceTable
                            key={"service-table"}
                            {...servicesTable}
                            columnsTemplate={"0.3fr 0.8fr 20ch 5ch 3.75em 2em 2em 1fr 1fr 3.5em 4em"}
                            filter={serviceFilter}
                            solidBackground={true}
                        >
                            <div className={"workspace-table-header"}>
                                <span
                                    className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                    onClick={() => onAddServices?.(services.filter((v) => !hideUuids.includes(v.uuid)))}
                                >
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
                                            onClick={() => onAddServices?.([service])}
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
                                        <Severity
                                            severity={service.severity}
                                            dataType={"Service"}
                                            uuid={service.uuid}
                                            workspace={workspace}
                                        />
                                        <CertaintyIcon certainty={service.certainty} />
                                    </div>
                                ))}
                        </StatelessWorkspaceTable>
                    );
                case AggregationType.HttpService:
                    return (
                        <StatelessWorkspaceTable
                            key={"http-service-table"}
                            {...httpServicesTable}
                            columnsTemplate={"0.3fr 0.8fr 20ch 5ch 1fr 0.8fr 2em 2em 1fr 1fr 3.5em 4em"}
                            filter={httpServiceFilter}
                            solidBackground={true}
                        >
                            <div className={"workspace-table-header"}>
                                <span
                                    className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                    onClick={() =>
                                        onAddHttpServices?.(httpServices.filter((v) => !hideUuids.includes(v.uuid)))
                                    }
                                >
                                    <RelationLeftIcon />
                                </span>
                                <span>HTTP Service</span>
                                <span>IP</span>
                                <span>Port</span>
                                <span>Domain</span>
                                <span>Base Path</span>
                                <span>TLS</span>
                                <span>SNI</span>
                                <span>Tags</span>
                                <span>Comment</span>
                                <span>Severity</span>
                                <span>Certainty</span>
                            </div>
                            {httpServices
                                .filter((v) => !hideUuids.includes(v.uuid))
                                .map((httpService) => (
                                    <div key={httpService.uuid} className="workspace-table-row">
                                        <span
                                            className="workspace-data-certainty-icon workspace-finding-selection-arrow"
                                            onClick={() => onAddHttpServices?.([httpService])}
                                        >
                                            <RelationLeftIcon />
                                        </span>
                                        <HttpServiceName httpService={httpService} />
                                        <IpAddr host={httpService.host} />
                                        <PortNumber port={httpService.port} />
                                        {httpService.domain ? <Domain domain={httpService.domain} /> : <span></span>}
                                        <SelectableText>{httpService.basePath}</SelectableText>
                                        <span>
                                            <Indicator off={!httpService.tls} />
                                        </span>
                                        <span>
                                            <Indicator off={!httpService.sniRequired} />
                                        </span>
                                        <TagList tags={httpService.tags} filter={serviceFilter} />
                                        <span>{httpService.comment}</span>
                                        <Severity
                                            severity={httpService.severity}
                                            dataType={"Service"}
                                            uuid={httpService.uuid}
                                            workspace={workspace}
                                        />
                                        <CertaintyIcon certainty={httpService.certainty} />
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
                <DataTabsSelector value={dataTab} onChange={setDataTab} />
                {tableElement}
            </div>
        );
    },
);

export default WorkspaceFindingDataTable;
