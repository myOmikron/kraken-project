import React, { ReactNode } from "react";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api } from "../../api/api";
import { ApiError } from "../../api/error";
import { AggregationType, FullDomain, FullHost, FullPort, FullService, SimpleTag, TagType } from "../../api/generated";
import { FullHttpService } from "../../api/generated/models/FullHttpService";
import Checkbox from "../../components/checkbox";
import Indicator from "../../components/indicator";
import OsIcon from "../../components/os-icon";
import SelectableText from "../../components/selectable-text";
import { ROUTES } from "../../routes";
import "../../styling/tabs.css";
import "../../styling/workspace-data.css";
import AttackIcon from "../../svg/attack";
import ClockActivityIcon from "../../svg/clock-activity";
import FindingIcon from "../../svg/finding";
import LinkIcon from "../../svg/link";
import PlusIcon from "../../svg/plus";
import TagIcon from "../../svg/tag";
import { ObjectFns, handleApiError } from "../../utils/helper";
import { Result } from "../../utils/result";
import CertaintyIcon from "./components/certainty-icon";
import ContextMenu, { ContextMenuEntry, GroupedMenuItem, PlainMenuItem } from "./components/context-menu";
import { DataTabsSelector, useDataTabs } from "./components/data-tabs";
import Domain from "./components/domain";
import EditableTags from "./components/editable-tags";
import FilterInput, { UseFilterReturn, useFilter } from "./components/filter-input";
import IpAddr from "./components/host";
import HttpServiceName from "./components/http-service";
import PortNumber from "./components/port";
import ServiceName from "./components/service";
import { Severity } from "./components/severity-icon";
import TableRow from "./components/table-row";
import TagList from "./components/tag-list";
import { StatelessWorkspaceTable, useTable } from "./components/workspace-table";
import { WORKSPACE_CONTEXT } from "./workspace";
import { CreateDomainForm } from "./workspace-data/workspace-data-create-domain";
import { CreateHostForm } from "./workspace-data/workspace-data-create-host";
import { CreateHttpServiceForm } from "./workspace-data/workspace-data-create-http-service";
import { CreatePortForm } from "./workspace-data/workspace-data-create-port";
import { CreateServiceForm } from "./workspace-data/workspace-data-create-service";
import { WorkspaceDataDomainDetails } from "./workspace-data/workspace-data-domain-details";
import { WorkspaceDataHostDetails } from "./workspace-data/workspace-data-host-details";
import { WorkspaceDataHttpServiceDetails } from "./workspace-data/workspace-data-http-service-details";
import { WorkspaceDataPortDetails } from "./workspace-data/workspace-data-port-details";
import { WorkspaceDataServiceDetails } from "./workspace-data/workspace-data-service-details";
import {
    CreateFindingObject,
    getCreateAffectedData,
    getCreateAffectedType,
} from "./workspace-finding/workspace-create-finding";
import WorkspaceFindingsQuickAttach from "./workspace-findings-quick-attach";

const DETAILS_TAB = { general: "General", results: "Results", relations: "Relations", findings: "Findings" };
type SelectedUuids = Record<AggregationType, Record<string, true>>;

type WorkspaceDataProps = {};

export default function WorkspaceData(props: WorkspaceDataProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [dataTab, setDataTab] = useDataTabs();

    const [detailTab, setDetailTab] = React.useState<keyof typeof DETAILS_TAB>("general");
    const [selected, setSelected] = React.useState<{ type: AggregationType; uuid: string } | null>(null);
    const [createForm, setCreateForm] = React.useState<AggregationType | null>(null);
    const [selectedUuids, setSelectedUuids] = React.useState<SelectedUuids>({
        [AggregationType.Domain]: {},
        [AggregationType.Host]: {},
        [AggregationType.Port]: {},
        [AggregationType.Service]: {},
        [AggregationType.HttpService]: {},
    });
    const [attaching, setAttaching] = React.useState<CreateFindingObject>();

    const globalFilter = useFilter(workspace, "global");
    const domainFilter = useFilter(workspace, "domain");
    const hostFilter = useFilter(workspace, "host");
    const portFilter = useFilter(workspace, "port");
    const serviceFilter = useFilter(workspace, "service");
    const httpServiceFilter = useFilter(workspace, "httpService");

    const { items: domains, ...domainsTable } = useTable<FullDomain>(
        (limit, offset) =>
            Api.workspaces.domains.all(workspace, limit, offset, {
                globalFilter: globalFilter.applied,
                domainFilter: domainFilter.applied,
            }),
        [workspace, globalFilter.applied, domainFilter.applied],
    );
    const { items: hosts, ...hostsTable } = useTable<FullHost>(
        (limit, offset) =>
            Api.workspaces.hosts.all(workspace, limit, offset, {
                globalFilter: globalFilter.applied,
                hostFilter: hostFilter.applied,
            }),
        [workspace, globalFilter.applied, hostFilter.applied],
    );
    const { items: ports, ...portsTable } = useTable<FullPort>(
        (limit, offset) =>
            Api.workspaces.ports.all(workspace, limit, offset, {
                globalFilter: globalFilter.applied,
                portFilter: portFilter.applied,
            }),
        [workspace, globalFilter.applied, portFilter.applied],
    );
    const { items: services, ...servicesTable } = useTable<FullService>(
        (limit, offset) =>
            Api.workspaces.services.all(workspace, limit, offset, {
                globalFilter: globalFilter.applied,
                serviceFilter: serviceFilter.applied,
            }),
        [workspace, globalFilter.applied, serviceFilter.applied],
    );
    const { items: httpServices, ...httpServicesTable } = useTable<FullHttpService>(
        (limit, offset) =>
            Api.workspaces.httpServices.all(workspace, limit, offset, {
                globalFilter: globalFilter.applied,
                httpServiceFilter: httpServiceFilter.applied,
            }),
        [workspace, globalFilter.applied, httpServiceFilter.applied],
    );

    // Jump to first page if filter changed
    React.useEffect(() => {
        domainsTable.setOffset(0);
        hostsTable.setOffset(0);
        portsTable.setOffset(0);
        servicesTable.setOffset(0);
        httpServicesTable.setOffset(0);
    }, [globalFilter.applied]);
    React.useEffect(() => domainsTable.setOffset(0), [domainFilter.applied]);
    React.useEffect(() => hostsTable.setOffset(0), [hostFilter.applied]);
    React.useEffect(() => portsTable.setOffset(0), [portFilter.applied]);
    React.useEffect(() => servicesTable.setOffset(0), [serviceFilter.applied]);
    React.useEffect(() => httpServicesTable.setOffset(0), [httpServiceFilter.applied]);

    function findingActions(item: CreateFindingObject): ContextMenuEntry[] {
        return [
            {
                icon: <FindingIcon />,
                group: "Finding",
                items: [
                    [
                        <>
                            <PlusIcon />
                            New with affected
                        </>,
                        () => {
                            // TODO: once we have support for passing hidden data
                            // across browser tabs, open in new tab, with hidden
                            // data, when `e.ctrlKey` is true
                            ROUTES.WORKSPACE_FINDINGS_CREATE.visit(
                                {
                                    uuid: workspace,
                                },
                                {
                                    affected: [item],
                                },
                            );
                        },
                    ],
                    [
                        <>
                            <LinkIcon />
                            Add as affected
                        </>,
                        (e) => {
                            if (e.ctrlKey)
                                ROUTES.WORKSPACE_FINDINGS_QUICK_ATTACH.open({
                                    workspace,
                                    type: getCreateAffectedType(item),
                                    uuid: getCreateAffectedData(item).uuid,
                                });
                            else setAttaching(item);
                        },
                    ],
                ],
            },
        ];
    }

    function copyTagsAction(tags: SimpleTag[], filter: UseFilterReturn): PlainMenuItem {
        return [
            <>
                <TagIcon />
                Copy tags into search
            </>,
            tags.length > 0
                ? (e) => {
                      for (const tag of tags) {
                          (e.ctrlKey ? globalFilter : filter).addColumn("tag", tag.name, e.altKey);
                      }
                  }
                : undefined,
        ];
    }

    function filterActionImpl(
        title: ReactNode,
        filter: UseFilterReturn,
        column: string,
        value: string | [string, string],
        overrideLabel?: ReactNode,
    ): PlainMenuItem {
        return [
            <>
                {overrideLabel ? (
                    overrideLabel
                ) : (
                    <>
                        {title}{" "}
                        <pre>
                            {column}:{value}
                        </pre>
                    </>
                )}
            </>,
            (e) => {
                if (Array.isArray(value))
                    (e.ctrlKey ? globalFilter : filter).addRange(column, value[0], value[1], e.altKey);
                else (e.ctrlKey ? globalFilter : filter).addColumn(column, value, e.altKey);
            },
        ];
    }

    const findSimilarAction = (filter: UseFilterReturn, column: string, value: string) =>
        filterActionImpl(
            <>
                <LinkIcon />
                Find similar
            </>,
            filter,
            column,
            value,
        );

    const filterAction = (
        filter: UseFilterReturn,
        column: string,
        value: string,
        {
            icon,
        }: {
            icon?: ReactNode;
        } = {},
    ) => filterActionImpl(icon ? <>{icon} Filter</> : "Filter", filter, column, value);

    const dateWithinAction = (
        filter: UseFilterReturn,
        column: string,
        date: Date,
        deltaPlusMinusMs: number,
        amountHuman: string,
    ) =>
        filterActionImpl(
            "",
            filter,
            column,
            [
                new Date(date.getTime() - deltaPlusMinusMs).toISOString(),
                new Date(date.getTime() + deltaPlusMinusMs).toISOString(),
            ],
            "Date within " + amountHuman,
        );

    function singleOrSubmenu(label: string, items: ContextMenuEntry[]): ContextMenuEntry[] {
        return items.length > 1
            ? [
                  {
                      group: label,
                      items: items,
                  },
              ]
            : items;
    }

    function createdAtAction(filter: UseFilterReturn, createdAt: Date): GroupedMenuItem {
        return {
            icon: <ClockActivityIcon />,
            group: "Filter relative to creation date",
            items: [
                dateWithinAction(filter, "createdAt", createdAt, 1000 * 60, "1 minute"),
                dateWithinAction(filter, "createdAt", createdAt, 1000 * 60 * 60, "1 hour"),
                dateWithinAction(filter, "createdAt", createdAt, 1000 * 60 * 60 * 24, "1 day"),
                dateWithinAction(filter, "createdAt", createdAt, 1000 * 60 * 60 * 24 * 7, "1 week"),
                dateWithinAction(filter, "createdAt", createdAt, 1000 * 60 * 60 * 24 * 7 * 4, "4 weeks"),
            ],
        };
    }

    const tableElement = (() => {
        switch (dataTab) {
            case AggregationType.Domain:
                return (
                    <StatelessWorkspaceTable
                        key={"domain-table"}
                        {...domainsTable}
                        columnsTemplate={"min-content 1fr 1fr 1fr 3.5em 4em 2.25em"}
                        onAdd={() => setCreateForm(AggregationType.Domain)}
                        filter={domainFilter}
                    >
                        <div className={"workspace-table-header"}>
                            <MultiSelectButton
                                items={domains}
                                uuids={selectedUuids[AggregationType.Domain]}
                                setUuids={(domains) =>
                                    setSelectedUuids({
                                        ...selectedUuids,
                                        [AggregationType.Domain]: domains,
                                    })
                                }
                            />
                            <span>Domain</span>
                            <span>Tags</span>
                            <span>Comment</span>
                            <span>Severity</span>
                            <span>Certainty</span>
                            <span />
                        </div>
                        {domains.map((domain) => (
                            <ContextMenu
                                key={domain.uuid}
                                as={TableRow}
                                className={
                                    domain.uuid === selected?.uuid
                                        ? "workspace-table-row workspace-table-row-selected"
                                        : "workspace-table-row"
                                }
                                onClick={() => {
                                    if (selected?.type !== AggregationType.Domain) {
                                        setDetailTab("general");
                                    }
                                    setSelected({ type: AggregationType.Domain, uuid: domain.uuid });
                                }}
                                menu={[
                                    ...findingActions({ domain }),
                                    /* TODO: certainty filter, then uncomment this:
                                    [
                                        <>
                                            {domain.certainty === "Unverified"
                                                ? CertaintyIcon({ certaintyType: "Unverified" })
                                                : CertaintyIcon({ certaintyType: "Verified" })}
                                            Filter <pre>certainty:{domain.certainty}</pre>
                                        </>,
                                        undefined,
                                    ], */
                                    copyTagsAction(domain.tags, domainFilter),
                                    () =>
                                        Api.workspaces.domains
                                            .relations(workspace, domain.uuid)
                                            .then((r) => {
                                                const data = r.unwrap();
                                                return [
                                                    ...singleOrSubmenu(
                                                        "Filter Host",
                                                        ObjectFns.uniqueObjects([
                                                            ...data.directHosts.map((h) => ["ips", h.ipAddr]),
                                                            ...data.indirectHosts.map((h) => ["ips", h.ipAddr]),
                                                            ...data.directHosts.map((h) => ["ips.os", h.osType]),
                                                            ...data.indirectHosts.map((h) => ["ips.os", h.osType]),
                                                        ]).map(([k, v]) => findSimilarAction(domainFilter, k, v)),
                                                    ),
                                                    ...singleOrSubmenu(
                                                        "Filter HTTP Service",
                                                        ObjectFns.uniqueObjects([
                                                            ...data.httpServices.map((s) => ["httpServices", s.name]),
                                                        ]).map(([k, v]) => findSimilarAction(domainFilter, k, v)),
                                                    ),
                                                ];
                                            })
                                            .catch(() => [["Failed loading hosts", undefined]]),
                                    createdAtAction(domainFilter, domain.createdAt),
                                ]}
                            >
                                <SelectButton
                                    uuid={domain.uuid}
                                    uuids={selectedUuids[AggregationType.Domain]}
                                    setUuids={(domains) =>
                                        setSelectedUuids({
                                            ...selectedUuids,
                                            [AggregationType.Domain]: domains,
                                        })
                                    }
                                />
                                <Domain domain={domain} />
                                <TagList tags={domain.tags} globalFilter={globalFilter} filter={domainFilter} />
                                <div>{domain.comment}</div>
                                <Severity
                                    severity={domain.severity}
                                    dataType={"Domain"}
                                    uuid={domain.uuid}
                                    workspace={workspace}
                                />
                                <CertaintyIcon certainty={domain.certainty} />
                                <AttackButton
                                    workspaceUuid={workspace}
                                    targetUuid={domain.uuid}
                                    targetType={"domain"}
                                />
                            </ContextMenu>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case AggregationType.Host:
                return (
                    <StatelessWorkspaceTable
                        key={"host-table"}
                        {...hostsTable}
                        columnsTemplate={"min-content 25ch 2em 1fr 1fr 3.5em 4em 2.25em"}
                        onAdd={() => setCreateForm(AggregationType.Host)}
                        filter={hostFilter}
                    >
                        <div className={"workspace-table-header"}>
                            <MultiSelectButton
                                items={hosts}
                                uuids={selectedUuids[AggregationType.Host]}
                                setUuids={(hosts) =>
                                    setSelectedUuids({
                                        ...selectedUuids,
                                        [AggregationType.Host]: hosts,
                                    })
                                }
                            />
                            <span>IP</span>
                            <span>OS</span>
                            <span>Tags</span>
                            <span>Comment</span>
                            <span>Severity</span>
                            <span>Certainty</span>
                            <span />
                        </div>
                        {hosts.map((host) => (
                            <ContextMenu
                                key={host.uuid}
                                as={TableRow}
                                className={
                                    host.uuid === selected?.uuid
                                        ? "workspace-table-row workspace-table-row-selected"
                                        : "workspace-table-row"
                                }
                                onClick={() => {
                                    if (selected?.type !== AggregationType.Host) {
                                        setDetailTab("general");
                                    }
                                    setSelected({ type: AggregationType.Host, uuid: host.uuid });
                                }}
                                menu={[
                                    ...findingActions({ host }),
                                    copyTagsAction(host.tags, hostFilter),
                                    filterAction(hostFilter, "os", host.osType, { icon: <OsIcon os={host.osType} /> }),
                                    () =>
                                        Api.workspaces.hosts
                                            .relations(workspace, host.uuid)
                                            .then((r) => {
                                                const data = r.unwrap();
                                                return [
                                                    ...singleOrSubmenu(
                                                        "Filter Domain",
                                                        ObjectFns.uniqueObjects(
                                                            [...data.directDomains, ...data.indirectDomains].map(
                                                                (d) => ["domains", d.domain],
                                                            ),
                                                        ).map(([k, v]) => findSimilarAction(hostFilter, k, v)),
                                                    ),
                                                    ...singleOrSubmenu(
                                                        "Filter Protocol",
                                                        ObjectFns.uniqueObjects(
                                                            data.ports.map((p) => ["port.protocols", p.protocol]),
                                                        ).map(([k, v]) => findSimilarAction(hostFilter, k, v)),
                                                    ),
                                                    ...singleOrSubmenu(
                                                        "Filter Port",
                                                        ObjectFns.uniqueObjects(
                                                            data.ports.map((p) => ["ports", p.port + ""]),
                                                        ).map(([k, v]) => findSimilarAction(hostFilter, k, v)),
                                                    ),
                                                    ...singleOrSubmenu(
                                                        "Filter Service",
                                                        ObjectFns.uniqueObjects(
                                                            data.services.map((s) => ["services", s.name]),
                                                        ).map(([k, v]) => findSimilarAction(hostFilter, k, v)),
                                                    ),
                                                    ...singleOrSubmenu(
                                                        "Filter HTTP Service",
                                                        ObjectFns.uniqueObjects(
                                                            data.httpServices.map((s) => ["httpServices", s.name]),
                                                        ).map(([k, v]) => findSimilarAction(hostFilter, k, v)),
                                                    ),
                                                ];
                                            })
                                            .catch(() => [["Failed loading hosts", undefined]]),
                                    createdAtAction(hostFilter, host.createdAt),
                                ]}
                            >
                                <SelectButton
                                    uuid={host.uuid}
                                    uuids={selectedUuids[AggregationType.Host]}
                                    setUuids={(hosts) =>
                                        setSelectedUuids({
                                            ...selectedUuids,
                                            [AggregationType.Host]: hosts,
                                        })
                                    }
                                />
                                <IpAddr host={host} />
                                <OsIcon tooltip os={host.osType} size="2em" />
                                <TagList tags={host.tags} globalFilter={globalFilter} filter={hostFilter} />
                                <div>{host.comment}</div>
                                <Severity
                                    severity={host.severity}
                                    dataType={"Host"}
                                    uuid={host.uuid}
                                    workspace={workspace}
                                />
                                <CertaintyIcon certainty={host.certainty} />
                                <AttackButton workspaceUuid={workspace} targetUuid={host.uuid} targetType={"host"} />
                            </ContextMenu>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case AggregationType.Port:
                return (
                    <StatelessWorkspaceTable
                        key={"port-table"}
                        {...portsTable}
                        columnsTemplate={"min-content 5ch 3.75em 20ch 1fr 1fr 3.5em 4em 2.25em"}
                        onAdd={() => setCreateForm(AggregationType.Port)}
                        filter={portFilter}
                    >
                        <div className={"workspace-table-header"}>
                            <MultiSelectButton
                                items={ports}
                                uuids={selectedUuids[AggregationType.Port]}
                                setUuids={(ports) =>
                                    setSelectedUuids({
                                        ...selectedUuids,
                                        [AggregationType.Port]: ports,
                                    })
                                }
                            />
                            <span>Port</span>
                            <span>Protocol</span>
                            <span>IP</span>
                            <span>Tags</span>
                            <span>Comment</span>
                            <span>Severity</span>
                            <span>Certainty</span>
                            <span />
                        </div>
                        {ports.map((port) => (
                            <ContextMenu
                                key={port.uuid}
                                as={TableRow}
                                className={
                                    port.uuid === selected?.uuid
                                        ? "workspace-table-row workspace-table-row-selected"
                                        : "workspace-table-row"
                                }
                                onClick={() => {
                                    if (selected?.type !== AggregationType.Port) {
                                        setDetailTab("general");
                                    }
                                    setSelected({ type: AggregationType.Port, uuid: port.uuid });
                                }}
                                menu={[
                                    ...findingActions({ port }),
                                    copyTagsAction(port.tags, portFilter),
                                    filterAction(portFilter, "ports", port.port + ""),
                                    filterAction(portFilter, "ips", port.host.ipAddr),
                                    filterAction(portFilter, "ips.os", port.host.osType),
                                    filterAction(portFilter, "protocols", port.protocol),
                                    () =>
                                        Api.workspaces.ports
                                            .relations(workspace, port.uuid)
                                            .then((r) => {
                                                const data = r.unwrap();
                                                return [
                                                    ...singleOrSubmenu(
                                                        "Filter Service",
                                                        ObjectFns.uniqueObjects(
                                                            data.services.map((s) => ["services", s.name]),
                                                        ).map(([k, v]) => findSimilarAction(portFilter, k, v)),
                                                    ),
                                                    ...singleOrSubmenu(
                                                        "Filter HTTP Service",
                                                        ObjectFns.uniqueObjects(
                                                            data.httpServices.map((s) => ["httpServices", s.name]),
                                                        ).map(([k, v]) => findSimilarAction(portFilter, k, v)),
                                                    ),
                                                ];
                                            })
                                            .catch(() => [["Failed loading hosts", undefined]]),
                                    createdAtAction(portFilter, port.createdAt),
                                ]}
                            >
                                <SelectButton
                                    uuid={port.uuid}
                                    uuids={selectedUuids[AggregationType.Port]}
                                    setUuids={(ports) =>
                                        setSelectedUuids({
                                            ...selectedUuids,
                                            [AggregationType.Port]: ports,
                                        })
                                    }
                                />
                                <PortNumber port={port} />
                                <span>{port.protocol.toUpperCase()}</span>
                                <IpAddr host={port.host} />
                                <TagList tags={port.tags} globalFilter={globalFilter} filter={portFilter} />
                                <span>{port.comment}</span>
                                <Severity
                                    severity={port.severity}
                                    dataType={"Port"}
                                    uuid={port.uuid}
                                    workspace={workspace}
                                />
                                <CertaintyIcon certainty={port.certainty} />
                                <AttackButton workspaceUuid={workspace} targetUuid={port.uuid} targetType={"port"} />
                            </ContextMenu>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case AggregationType.Service:
                return (
                    <StatelessWorkspaceTable
                        key={"service-table"}
                        {...servicesTable}
                        columnsTemplate={"min-content 0.8fr 20ch 5ch 3.75em 2em 2em 1fr 1fr 3.5em 4em 2.25em"}
                        onAdd={() => setCreateForm(AggregationType.Service)}
                        filter={serviceFilter}
                    >
                        <div className={"workspace-table-header"}>
                            <MultiSelectButton
                                items={services}
                                uuids={selectedUuids[AggregationType.Service]}
                                setUuids={(services) =>
                                    setSelectedUuids({
                                        ...selectedUuids,
                                        [AggregationType.Service]: services,
                                    })
                                }
                            />
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
                            <span />
                        </div>
                        {services.map((service) => (
                            <ContextMenu
                                key={service.uuid}
                                as={TableRow}
                                className={
                                    service.uuid === selected?.uuid
                                        ? "workspace-table-row workspace-table-row-selected"
                                        : "workspace-table-row"
                                }
                                onClick={() => {
                                    if (selected?.type !== AggregationType.Service) {
                                        setDetailTab("general");
                                    }
                                    setSelected({ type: AggregationType.Service, uuid: service.uuid });
                                }}
                                menu={[
                                    ...findingActions({ service }),
                                    copyTagsAction(service.tags, serviceFilter),
                                    filterAction(serviceFilter, "service", service.name),
                                    filterAction(serviceFilter, "ips", service.host.ipAddr),
                                    filterAction(serviceFilter, "ips.os", service.host.osType),
                                    ...(service.port
                                        ? [filterAction(serviceFilter, "ports", service.port.port + "")]
                                        : []),
                                    ...(service.protocols
                                        ? (() => {
                                              const res = [];
                                              const p = service.protocols;
                                              if ("sctp" in p && p.sctp) {
                                                  res.push(filterAction(serviceFilter, "protocols", "Sctp"));
                                              } else if ("tcp" in p && p.tcp) {
                                                  res.push(filterAction(serviceFilter, "protocols", "Tcp"));
                                              } else if ("udp" in p && p.udp) {
                                                  res.push(filterAction(serviceFilter, "protocols", "Udp"));
                                              } else if ("unknown" in p && p.unknown) {
                                                  res.push(filterAction(serviceFilter, "protocols", "Unknown"));
                                              }
                                              return res;
                                          })()
                                        : []),
                                    createdAtAction(serviceFilter, service.createdAt),
                                ]}
                            >
                                <SelectButton
                                    uuid={service.uuid}
                                    uuids={selectedUuids[AggregationType.Service]}
                                    setUuids={(services) =>
                                        setSelectedUuids({
                                            ...selectedUuids,
                                            [AggregationType.Service]: services,
                                        })
                                    }
                                />
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
                                <TagList tags={service.tags} globalFilter={globalFilter} filter={serviceFilter} />
                                <span>{service.comment}</span>
                                <Severity
                                    severity={service.severity}
                                    dataType={"Service"}
                                    uuid={service.uuid}
                                    workspace={workspace}
                                />
                                <CertaintyIcon certainty={service.certainty} />
                                <AttackButton
                                    workspaceUuid={workspace}
                                    targetUuid={service.uuid}
                                    targetType={"service"}
                                />
                            </ContextMenu>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case AggregationType.HttpService:
                return (
                    <StatelessWorkspaceTable
                        key={"http-service-table"}
                        {...httpServicesTable}
                        columnsTemplate={"min-content 0.7fr 20ch 5ch 1fr 0.6fr 2em 2em 1fr 1fr 3.5em 4em 2.25em"}
                        onAdd={() => setCreateForm(AggregationType.HttpService)}
                        filter={httpServiceFilter}
                    >
                        <div className={"workspace-table-header"}>
                            <MultiSelectButton
                                items={httpServices}
                                uuids={selectedUuids[AggregationType.HttpService]}
                                setUuids={(httpServices) =>
                                    setSelectedUuids({
                                        ...selectedUuids,
                                        [AggregationType.HttpService]: httpServices,
                                    })
                                }
                            />
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
                            <span />
                        </div>
                        {httpServices.map((httpService) => (
                            <ContextMenu
                                key={httpService.uuid}
                                as={TableRow}
                                className={
                                    httpService.uuid === selected?.uuid
                                        ? "workspace-table-row workspace-table-row-selected"
                                        : "workspace-table-row"
                                }
                                onClick={() => {
                                    if (selected?.type !== AggregationType.Service) {
                                        setDetailTab("general");
                                    }
                                    setSelected({ type: AggregationType.Service, uuid: httpService.uuid });
                                }}
                                menu={[
                                    ...findingActions({ httpService }),
                                    copyTagsAction(httpService.tags, httpServiceFilter),
                                    // TODO:HTTP AST relation operations
                                    createdAtAction(httpServiceFilter, httpService.createdAt),
                                ]}
                            >
                                <SelectButton
                                    uuid={httpService.uuid}
                                    uuids={selectedUuids[AggregationType.HttpService]}
                                    setUuids={(services) =>
                                        setSelectedUuids({
                                            ...selectedUuids,
                                            [AggregationType.HttpService]: services,
                                        })
                                    }
                                />
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
                                <TagList tags={httpService.tags} globalFilter={globalFilter} filter={serviceFilter} />
                                <span>{httpService.comment}</span>
                                <Severity
                                    severity={httpService.severity}
                                    dataType={"Service"}
                                    uuid={httpService.uuid}
                                    workspace={workspace}
                                />
                                <CertaintyIcon certainty={"UnknownService"} />
                                <AttackButton
                                    workspaceUuid={workspace}
                                    targetUuid={httpService.uuid}
                                    targetType={"service"}
                                />
                            </ContextMenu>
                        ))}
                    </StatelessWorkspaceTable>
                );
            default:
                return "Unimplemented";
        }
    })();
    const detailsElement = (() => {
        switch (selected?.type) {
            case AggregationType.Domain:
                return (
                    <WorkspaceDataDomainDetails
                        domain={selected.uuid}
                        updateDomain={domainsTable.updateItem}
                        tab={detailTab}
                    />
                );
            case AggregationType.Host:
                return (
                    <WorkspaceDataHostDetails host={selected.uuid} updateHost={hostsTable.updateItem} tab={detailTab} />
                );
            case AggregationType.Port:
                return (
                    <WorkspaceDataPortDetails port={selected.uuid} updatePort={portsTable.updateItem} tab={detailTab} />
                );
            case AggregationType.Service:
                return (
                    <WorkspaceDataServiceDetails
                        service={selected.uuid}
                        updateService={servicesTable.updateItem}
                        tab={detailTab}
                    />
                );
            case AggregationType.HttpService:
                return (
                    <WorkspaceDataHttpServiceDetails
                        httpService={selected.uuid}
                        updateHttpService={httpServicesTable.updateItem}
                        tab={detailTab}
                    />
                );
            case undefined:
                return null;
        }
    })();
    const createElement = (() => {
        switch (createForm) {
            case null:
                return null;
            case AggregationType.Domain:
                return (
                    <CreateDomainForm
                        onSubmit={() => {
                            setCreateForm(null);
                            domainsTable.reload();
                        }}
                    />
                );
            case AggregationType.Host:
                return (
                    <CreateHostForm
                        onSubmit={() => {
                            setCreateForm(null);
                            hostsTable.reload();
                        }}
                    />
                );
            case AggregationType.Port:
                return (
                    <CreatePortForm
                        onSubmit={() => {
                            setCreateForm(null);
                            portsTable.reload();
                            // also reload hosts table, since a new host may be implicitly created
                            hostsTable.reload();
                        }}
                    />
                );
            case AggregationType.Service:
                return (
                    <CreateServiceForm
                        onSubmit={() => {
                            setCreateForm(null);
                            servicesTable.reload();
                            // also reload hosts and ports table, since a new host/port may be implicitly created
                            hostsTable.reload();
                            portsTable.reload();
                        }}
                    />
                );
            case AggregationType.HttpService:
                return (
                    <CreateHttpServiceForm
                        onSubmit={() => {
                            setCreateForm(null);
                            httpServicesTable.reload();
                            // also reload hosts, ports and domains table, since a new host/port/domain may be implicitly created
                            hostsTable.reload();
                            portsTable.reload();
                            domainsTable.reload();
                        }}
                    />
                );
        }
    })();
    return (
        <>
            <div className={"workspace-data-container"}>
                <div className={"workspace-data-filter pane"}>
                    <FilterInput {...globalFilter} />
                </div>
                <div className="workspace-data-table">
                    <DataTabsSelector value={dataTab} onChange={setDataTab} />
                    {tableElement}
                </div>
                <div className={"workspace-data-details pane"}>
                    {ObjectFns.isEmpty(selectedUuids[AggregationType.Domain]) &&
                    ObjectFns.isEmpty(selectedUuids[AggregationType.Host]) &&
                    ObjectFns.isEmpty(selectedUuids[AggregationType.Port]) &&
                    ObjectFns.isEmpty(selectedUuids[AggregationType.Service]) &&
                    ObjectFns.isEmpty(selectedUuids[AggregationType.HttpService]) ? (
                        selected ? (
                            <>
                                <h2 className={"sub-heading"}>
                                    <span>{selected.type} </span>
                                    Details
                                </h2>
                                <div className={"workspace-data-details-selector"}>
                                    {Object.entries(DETAILS_TAB).map(([key, displayName]) => (
                                        <h3
                                            className={
                                                "heading " +
                                                (detailTab !== key ? "" : "workspace-data-details-selected-tab")
                                            }
                                            onClick={() => setDetailTab(key as keyof typeof DETAILS_TAB)}
                                        >
                                            {displayName}
                                        </h3>
                                    ))}
                                </div>
                                {detailsElement}
                            </>
                        ) : null
                    ) : (
                        <MultiSelectMenu
                            selectedUuids={selectedUuids}
                            setSelectedUuids={setSelectedUuids}
                            onUpdate={() => {
                                if (!ObjectFns.isEmpty(selectedUuids[AggregationType.Domain])) domainsTable.reload();
                                if (!ObjectFns.isEmpty(selectedUuids[AggregationType.Host])) hostsTable.reload();
                                if (!ObjectFns.isEmpty(selectedUuids[AggregationType.Port])) portsTable.reload();
                                if (!ObjectFns.isEmpty(selectedUuids[AggregationType.Service])) servicesTable.reload();
                                if (!ObjectFns.isEmpty(selectedUuids[AggregationType.HttpService]))
                                    httpServicesTable.reload();
                            }}
                            onDelete={() => {
                                domainsTable.reload();
                                hostsTable.reload();
                                portsTable.reload();
                                servicesTable.reload();
                                httpServicesTable.reload();
                                setSelected(null);
                            }}
                        />
                    )}
                </div>
            </div>
            <Popup nested modal open={createForm !== null} onClose={() => setCreateForm(null)}>
                {createElement}
            </Popup>
            {attaching && (
                <Popup nested modal open onClose={() => setAttaching(undefined)}>
                    <div className="pane-thin">
                        <WorkspaceFindingsQuickAttach
                            type={getCreateAffectedType(attaching)}
                            data={getCreateAffectedData(attaching)}
                            onAttached={(f, wantMore) => {
                                if (!wantMore) setAttaching(undefined);
                            }}
                        />
                    </div>
                </Popup>
            )}
        </>
    );
}

export function AttackButton(props: Parameters<typeof ROUTES.WORKSPACE_TARGETED_ATTACKS.clickHandler>[0]) {
    return (
        <button className={"icon-button"} type={"button"} {...ROUTES.WORKSPACE_TARGETED_ATTACKS.clickHandler(props)}>
            <AttackIcon />
        </button>
    );
}

type MultiSelectMenuProps = {
    selectedUuids: SelectedUuids;
    setSelectedUuids: React.Dispatch<React.SetStateAction<SelectedUuids>>;
    onUpdate: () => void;
    onDelete: () => void;
};

export function MultiSelectMenu(props: MultiSelectMenuProps) {
    const { selectedUuids, setSelectedUuids, onUpdate, onDelete } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [newTags, setNewTags] = React.useState<Array<SimpleTag>>([]);
    const [deleteData, setDeleteData] = React.useState(false);

    const domainsLen = ObjectFns.len(selectedUuids[AggregationType.Domain]);
    const hostsLen = ObjectFns.len(selectedUuids[AggregationType.Host]);
    const portsLen = ObjectFns.len(selectedUuids[AggregationType.Port]);
    const servicesLen = ObjectFns.len(selectedUuids[AggregationType.Service]);
    const httpServicesLen = ObjectFns.len(selectedUuids[AggregationType.HttpService]);
    const totalLen = domainsLen + hostsLen + portsLen + servicesLen + httpServicesLen;

    return (
        <>
            <div className="workspace-data-pane">
                <h2 className={"sub-heading"}>Selection</h2>
                <div className={"workspace-data-multi-select-table"}>
                    {domainsLen === 0
                        ? null
                        : [
                              <span>Domains</span>,
                              <span>{domainsLen}</span>,
                              <button
                                  type={"button"}
                                  className={"button"}
                                  onClick={() => setSelectedUuids({ ...selectedUuids, [AggregationType.Domain]: {} })}
                              >
                                  Unselect all domains
                              </button>,
                          ]}
                    {hostsLen === 0
                        ? null
                        : [
                              <span>Hosts</span>,
                              <span>{hostsLen}</span>,
                              <button
                                  type={"button"}
                                  className={"button"}
                                  onClick={() => setSelectedUuids({ ...selectedUuids, [AggregationType.Host]: {} })}
                              >
                                  Unselect all hosts
                              </button>,
                          ]}
                    {portsLen === 0
                        ? null
                        : [
                              <span>Ports</span>,
                              <span>{portsLen}</span>,
                              <button
                                  type={"button"}
                                  className={"button"}
                                  onClick={() => setSelectedUuids({ ...selectedUuids, [AggregationType.Port]: {} })}
                              >
                                  Unselect all ports
                              </button>,
                          ]}
                    {servicesLen === 0
                        ? null
                        : [
                              <span>Services</span>,
                              <span>{servicesLen}</span>,
                              <button
                                  type={"button"}
                                  className={"button"}
                                  onClick={() => setSelectedUuids({ ...selectedUuids, [AggregationType.Service]: {} })}
                              >
                                  Unselect all services
                              </button>,
                          ]}
                    {httpServicesLen === 0
                        ? null
                        : [
                              <span>HTTP Services</span>,
                              <span>{httpServicesLen}</span>,
                              <button
                                  type={"button"}
                                  className={"button"}
                                  onClick={() =>
                                      setSelectedUuids({ ...selectedUuids, [AggregationType.HttpService]: {} })
                                  }
                              >
                                  Unselect all HTTP services
                              </button>,
                          ]}
                    <span className={"workspace-data-multi-select-total"}>Total</span>
                    <span className={"workspace-data-multi-select-total"}>{totalLen}</span>
                    <button
                        type={"button"}
                        className={"button workspace-data-multi-select-total"}
                        onClick={() =>
                            setSelectedUuids({
                                [AggregationType.Domain]: {},
                                [AggregationType.Host]: {},
                                [AggregationType.Port]: {},
                                [AggregationType.Service]: {},
                                [AggregationType.HttpService]: {},
                            })
                        }
                    >
                        Unselect all
                    </button>
                </div>
                <div className={"workspace-data-multi-select-tag-buttons"}>
                    <button
                        type={"button"}
                        className={"button mixed-button"}
                        onClick={() => {
                            ROUTES.WORKSPACE_SELECTION_ATTACKS.visit(
                                {
                                    workspaceUuid: workspace,
                                },
                                {
                                    domains: Object.keys(selectedUuids[AggregationType.Domain]),
                                    hosts: Object.keys(selectedUuids[AggregationType.Host]),
                                    ports: Object.keys(selectedUuids[AggregationType.Port]),
                                    services: Object.keys(selectedUuids[AggregationType.Service]),
                                    httpServices: Object.keys(selectedUuids[AggregationType.HttpService]),
                                },
                            );
                        }}
                    >
                        <AttackIcon />
                        Attack selected
                    </button>
                </div>
                <div className={"workspace-data-multi-select-tag-buttons"}>
                    <button
                        type={"button"}
                        className={"button mixed-button"}
                        onClick={async () => {
                            const affected = await resolveSelection(workspace, selectedUuids, true);
                            ROUTES.WORKSPACE_FINDINGS_CREATE.visit(
                                {
                                    uuid: workspace,
                                },
                                {
                                    affected: [
                                        ...affected.domains.map<CreateFindingObject>((d) => ({ domain: d })),
                                        ...affected.hosts.map<CreateFindingObject>((d) => ({ host: d })),
                                        ...affected.services.map<CreateFindingObject>((d) => ({ service: d })),
                                        ...affected.httpServices.map<CreateFindingObject>((d) => ({ httpService: d })),
                                        ...affected.ports.map<CreateFindingObject>((d) => ({ port: d })),
                                    ],
                                },
                            );
                        }}
                    >
                        <FindingIcon />
                        Create new finding
                    </button>
                </div>
            </div>

            <div className="workspace-data-pane">
                <h2 className={"sub-heading"}>Modify tags</h2>
                <EditableTags workspace={workspace} tags={newTags} onChange={setNewTags} />
                <div className={"workspace-data-multi-select-tag-buttons"}>
                    <button
                        type={"button"}
                        className={"button"}
                        onClick={() => {
                            updateTags(workspace, selectedUuids, addStrategy, newTags).then(() => {
                                toast.success("Added tags for selected items");
                                setNewTags([]);
                                onUpdate();
                            });
                        }}
                    >
                        Add tags
                    </button>
                    <button
                        type={"button"}
                        className={"button"}
                        onClick={() => {
                            updateTags(workspace, selectedUuids, overwriteStrategy, newTags).then(() => {
                                toast.success("Overwrote tags for selected items");
                                setNewTags([]);
                                onUpdate();
                            });
                        }}
                    >
                        Overwrite tags
                    </button>
                    <button
                        type={"button"}
                        className={"button"}
                        onClick={() => {
                            updateTags(workspace, selectedUuids, removeStrategy, newTags).then(() => {
                                toast.success("Removed tags for selected items");
                                setNewTags([]);
                                onUpdate();
                            });
                        }}
                    >
                        Remove tags
                    </button>
                </div>
            </div>
            <div className="workspace-data-danger-pane">
                <h2 className={"sub-heading"}>Danger Zone</h2>
                <button
                    onClick={() => {
                        setDeleteData(true);
                    }}
                    className="workspace-settings-red-button button"
                >
                    Delete selected data
                </button>
            </div>
            <Popup
                modal
                nested
                open={deleteData}
                onClose={() => {
                    setDeleteData(false);
                }}
            >
                <div className="pane danger">
                    <div className="workspace-data-popup">
                        <h2 className="sub-heading">Are you sure to delete this data?</h2>
                        <button
                            className="workspace-settings-red-button button"
                            onClick={() => {
                                setDeleteData(false);
                            }}
                        >
                            No
                        </button>
                        <button
                            className="workspace-settings-red-button button"
                            onClick={() => {
                                const promises: Array<Promise<void>> = [];
                                let numOk = 0;
                                let numErr = 0;
                                const stillSelected: SelectedUuids = {
                                    [AggregationType.Domain]: {},
                                    [AggregationType.Host]: {},
                                    [AggregationType.Port]: {},
                                    [AggregationType.Service]: {},
                                    [AggregationType.HttpService]: {},
                                };
                                if (domainsLen !== 0) {
                                    Object.keys(selectedUuids[AggregationType.Domain]).map((u) => {
                                        promises.push(
                                            Api.workspaces.domains.delete(workspace, u).then((result) => {
                                                if (result.is_err()) {
                                                    numErr += 1;
                                                    handleApiError(result);
                                                    stillSelected[AggregationType.Domain][u] = true;
                                                } else if (result.is_ok()) {
                                                    numOk += 1;
                                                }
                                            }),
                                        );
                                    });
                                }
                                if (hostsLen !== 0) {
                                    Object.keys(selectedUuids[AggregationType.Host]).map((u) => {
                                        promises.push(
                                            Api.workspaces.hosts.delete(workspace, u).then((result) => {
                                                if (result.is_err()) {
                                                    numErr += 1;
                                                    handleApiError(result);
                                                    stillSelected[AggregationType.Host][u] = true;
                                                } else if (result.is_ok()) {
                                                    numOk += 1;
                                                }
                                            }),
                                        );
                                    });
                                }
                                if (portsLen !== 0) {
                                    Object.keys(selectedUuids[AggregationType.Port]).map((u) => {
                                        promises.push(
                                            Api.workspaces.ports.delete(workspace, u).then((result) => {
                                                if (result.is_err()) {
                                                    numErr += 1;
                                                    handleApiError(result);
                                                    stillSelected[AggregationType.Port][u] = true;
                                                } else if (result.is_ok()) {
                                                    numOk += 1;
                                                }
                                            }),
                                        );
                                    });
                                }
                                if (servicesLen !== 0) {
                                    Object.keys(selectedUuids[AggregationType.Service]).map((u) => {
                                        promises.push(
                                            Api.workspaces.services.delete(workspace, u).then((result) => {
                                                if (result.is_err()) {
                                                    numErr += 1;
                                                    handleApiError(result);
                                                    stillSelected[AggregationType.Service][u] = true;
                                                } else if (result.is_ok()) {
                                                    numOk += 1;
                                                }
                                            }),
                                        );
                                    });
                                }
                                if (httpServicesLen !== 0) {
                                    Object.keys(selectedUuids[AggregationType.HttpService]).map((u) => {
                                        promises.push(
                                            Api.workspaces.httpServices.delete(workspace, u).then((result) => {
                                                if (result.is_err()) {
                                                    numErr += 1;
                                                    handleApiError(result);
                                                    stillSelected[AggregationType.HttpService][u] = true;
                                                } else if (result.is_ok()) {
                                                    numOk += 1;
                                                }
                                            }),
                                        );
                                    });
                                }
                                Promise.all(promises).then(() => {
                                    if (numErr === 0) {
                                        toast.success("Deleted successfully");
                                        onDelete();
                                        setSelectedUuids(stillSelected);
                                        setDeleteData(false);
                                    } else {
                                        toast.info(`Deleted ${numOk}, failed to delete ${numErr} `);
                                        onDelete();
                                        setSelectedUuids(stillSelected);
                                        setDeleteData(false);
                                    }
                                });
                            }}
                        >
                            Yes
                        </button>
                    </div>
                </div>
            </Popup>
        </>
    );
}

type MultiSelectButtonProps = {
    items: Array<{ uuid: string }>;
    uuids: Record<string, true>;
    setUuids: (uuids: Record<string, true>) => void;
};

export function MultiSelectButton(props: MultiSelectButtonProps) {
    const { items, uuids, setUuids } = props;
    return (
        <Checkbox
            value={items.find(({ uuid }) => uuid in uuids) !== undefined}
            onChange={(selected) => {
                if (selected) {
                    setUuids({ ...Object.fromEntries(items.map(({ uuid }) => [uuid, true])), ...uuids });
                } else {
                    const remaining = { ...uuids };
                    for (const { uuid } of items) {
                        delete remaining[uuid];
                    }
                    setUuids(remaining);
                }
            }}
        />
    );
}

type SelectButtonProps = {
    uuid: string;
    uuids: Record<string, true>;
    setUuids: (uuids: Record<string, true>) => void;
};

export function SelectButton(props: SelectButtonProps) {
    const { uuid, uuids, setUuids } = props;
    return (
        <Checkbox
            value={uuid in uuids}
            onChange={(selected) => {
                if (selected) {
                    setUuids({ [uuid]: true, ...uuids });
                } else {
                    const { [uuid]: _, ...remaining } = uuids;
                    setUuids(remaining);
                }
            }}
        />
    );
}

type UpdateStrategy = (
    curTags: Array<SimpleTag>,
    newTags: Array<SimpleTag>,
) => { workspaceTags: Array<string>; globalTags: Array<string> };

async function resolveSelection(
    workspace: string,
    uuids: SelectedUuids,
    skipInvalid?: boolean,
): Promise<{
    domains: FullDomain[];
    hosts: FullHost[];
    services: FullService[];
    httpServices: FullHttpService[];
    ports: FullPort[];
}> {
    function unwrap<T>(e: Result<T, ApiError>) {
        if (skipInvalid && !e.is_ok()) return undefined;
        else return e.unwrap();
    }

    return {
        domains: (
            await Promise.all(
                Object.keys(uuids[AggregationType.Domain]).map((uuid) =>
                    Api.workspaces.domains.get(workspace, uuid).then(unwrap),
                ),
            )
        ).filter((v) => v !== undefined) as Array<FullDomain>,
        hosts: (
            await Promise.all(
                Object.keys(uuids[AggregationType.Host]).map((uuid) =>
                    Api.workspaces.hosts.get(workspace, uuid).then(unwrap),
                ),
            )
        ).filter((v) => v !== undefined) as Array<FullHost>,
        services: (
            await Promise.all(
                Object.keys(uuids[AggregationType.Service]).map((uuid) =>
                    Api.workspaces.services.get(workspace, uuid).then(unwrap),
                ),
            )
        ).filter((v) => v !== undefined) as Array<FullService>,
        httpServices: (
            await Promise.all(
                Object.keys(uuids[AggregationType.HttpService]).map((uuid) =>
                    Api.workspaces.httpServices.get(workspace, uuid).then(unwrap),
                ),
            )
        ).filter((v) => v !== undefined) as Array<FullHttpService>,
        ports: (
            await Promise.all(
                Object.keys(uuids[AggregationType.Port]).map((uuid) =>
                    Api.workspaces.ports.get(workspace, uuid).then(unwrap),
                ),
            )
        ).filter((v) => v !== undefined) as Array<FullPort>,
    };
}

async function updateTags(workspace: string, uuids: SelectedUuids, strategy: UpdateStrategy, tags: Array<SimpleTag>) {
    await Promise.all(
        Object.keys(uuids[AggregationType.Domain]).map((uuid) =>
            Api.workspaces.domains.get(workspace, uuid).then((result) => {
                let promise = null;
                handleApiError(result, ({ tags: curTags }) => {
                    promise = Api.workspaces.domains
                        .update(workspace, uuid, strategy(curTags, tags))
                        .then(handleApiError);
                });
                return promise;
            }),
        ),
    );
    await Promise.all(
        Object.keys(uuids[AggregationType.Host]).map((uuid) =>
            Api.workspaces.hosts.get(workspace, uuid).then((result) => {
                let promise = null;
                handleApiError(result, ({ tags: curTags }) => {
                    promise = Api.workspaces.hosts
                        .update(workspace, uuid, strategy(curTags, tags))
                        .then(handleApiError);
                });
                return promise;
            }),
        ),
    );
    await Promise.all(
        Object.keys(uuids[AggregationType.Port]).map((uuid) =>
            Api.workspaces.ports.get(workspace, uuid).then((result) => {
                let promise = null;
                handleApiError(result, ({ tags: curTags }) => {
                    promise = Api.workspaces.ports
                        .update(workspace, uuid, strategy(curTags, tags))
                        .then(handleApiError);
                });
                return promise;
            }),
        ),
    );
    await Promise.all(
        Object.keys(uuids[AggregationType.Service]).map((uuid) =>
            Api.workspaces.services.get(workspace, uuid).then((result) => {
                let promise = null;
                handleApiError(result, ({ tags: curTags }) => {
                    promise = Api.workspaces.services
                        .update(workspace, uuid, strategy(curTags, tags))
                        .then(handleApiError);
                });
                return promise;
            }),
        ),
    );
    await Promise.all(
        Object.keys(uuids[AggregationType.HttpService]).map((uuid) =>
            Api.workspaces.httpServices.get(workspace, uuid).then((result) => {
                let promise = null;
                handleApiError(result, ({ tags: curTags }) => {
                    promise = Api.workspaces.httpServices
                        .update(workspace, uuid, strategy(curTags, tags))
                        .then(handleApiError);
                });
                return promise;
            }),
        ),
    );
}

function addStrategy(curTags: Array<SimpleTag>, newTags: Array<SimpleTag>) {
    const workspaceTags = [
        ...new Set( // Use Set to eliminate duplicates
            [...curTags, ...newTags].filter(({ tagType }) => tagType === TagType.Workspace).map(({ uuid }) => uuid),
        ).keys(),
    ];
    const globalTags = [
        ...new Set( // Use Set to eliminate duplicates
            [...curTags, ...newTags].filter(({ tagType }) => tagType === TagType.Global).map(({ uuid }) => uuid),
        ).keys(),
    ];
    return { workspaceTags, globalTags };
}

function overwriteStrategy(_curTags: Array<SimpleTag>, newTags: Array<SimpleTag>) {
    const workspaceTags = newTags.filter(({ tagType }) => tagType === TagType.Workspace).map(({ uuid }) => uuid);
    const globalTags = newTags.filter(({ tagType }) => tagType === TagType.Global).map(({ uuid }) => uuid);
    return { workspaceTags, globalTags };
}

function removeStrategy(curTags: Array<SimpleTag>, newTags: Array<SimpleTag>) {
    const removedTags = Object.fromEntries(newTags.map(({ uuid }) => [uuid, null]));
    const workspaceTags = curTags
        .filter(({ uuid, tagType }) => tagType === TagType.Workspace && !(uuid in removedTags))
        .map(({ uuid }) => uuid);
    const globalTags = curTags
        .filter(({ uuid, tagType }) => tagType === TagType.Global && !(uuid in removedTags))
        .map(({ uuid }) => uuid);
    return { workspaceTags, globalTags };
}
