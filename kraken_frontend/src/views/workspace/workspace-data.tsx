import React, { ReactNode } from "react";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api } from "../../api/api";
import { FullDomain, FullHost, FullPort, FullService, SimpleTag, TagType } from "../../api/generated";
import Checkbox from "../../components/checkbox";
import Indicator from "../../components/indicator";
import OsIcon from "../../components/os-icon";
import { ROUTES } from "../../routes";
import "../../styling/tabs.css";
import "../../styling/workspace-data.css";
import AttackIcon from "../../svg/attack";
import ClockActivityIcon from "../../svg/clock-activity";
import HistoricalIcon from "../../svg/historical";
import LinkIcon from "../../svg/link";
import TagIcon from "../../svg/tag";
import UnknownIcon from "../../svg/unknown";
import UnverifiedIcon from "../../svg/unverified";
import VerifiedIcon from "../../svg/verified";
import { ObjectFns, handleApiError } from "../../utils/helper";
import ContextMenu, { ContextMenuEntry, GroupedMenuItem, PlainMenuItem } from "./components/context-menu";
import Domain from "./components/domain";
import EditableTags from "./components/editable-tags";
import FilterInput, { FilterOutput, useFilter } from "./components/filter-input";
import IpAddr from "./components/host";
import PortNumber from "./components/port";
import ServiceName from "./components/service";
import TableRow from "./components/table-row";
import TagList from "./components/tag-list";
import { StatelessWorkspaceTable, useTable } from "./components/workspace-table";
import { WORKSPACE_CONTEXT } from "./workspace";
import { CreateDomainForm } from "./workspace-data/workspace-data-create-domain";
import { CreateHostForm } from "./workspace-data/workspace-data-create-host";
import { CreatePortForm } from "./workspace-data/workspace-data-create-port";
import { CreateServiceForm } from "./workspace-data/workspace-data-create-service";
import { WorkspaceDataDomainDetails } from "./workspace-data/workspace-data-domain-details";
import { WorkspaceDataHostDetails } from "./workspace-data/workspace-data-host-details";
import { WorkspaceDataPortDetails } from "./workspace-data/workspace-data-port-details";
import { WorkspaceDataServiceDetails } from "./workspace-data/workspace-data-service-details";

const TABS = { domains: "Domains", hosts: "Hosts", ports: "Ports", services: "Services" };
const DETAILS_TAB = { general: "General", results: "Results", relations: "Relations", findings: "Findings" };
type SelectedUuids = { [Key in keyof typeof TABS]: Record<string, true> };

type WorkspaceDataProps = {};

export default function WorkspaceData(props: WorkspaceDataProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [tab, setTab] = React.useState<keyof typeof TABS>("hosts");
    const [detailTab, setDetailTab] = React.useState<keyof typeof DETAILS_TAB>("general");
    const [selected, setSelected] = React.useState<{ type: keyof typeof TABS; uuid: string } | null>(null);
    const [createForm, setCreateForm] = React.useState<keyof typeof TABS | null>(null);
    const [selectedUuids, setSelectedUuids] = React.useState<SelectedUuids>({
        domains: {},
        hosts: {},
        ports: {},
        services: {},
    });

    const globalFilter = useFilter(workspace, "global");
    const domainFilter = useFilter(workspace, "domain");
    const hostFilter = useFilter(workspace, "host");
    const portFilter = useFilter(workspace, "port");
    const serviceFilter = useFilter(workspace, "service");

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

    // Jump to first page if filter changed
    React.useEffect(() => {
        domainsTable.setOffset(0);
        hostsTable.setOffset(0);
        portsTable.setOffset(0);
        servicesTable.setOffset(0);
    }, [globalFilter.applied]);
    React.useEffect(() => domainsTable.setOffset(0), [domainFilter.applied]);
    React.useEffect(() => hostsTable.setOffset(0), [hostFilter.applied]);
    React.useEffect(() => portsTable.setOffset(0), [portFilter.applied]);
    React.useEffect(() => servicesTable.setOffset(0), [serviceFilter.applied]);

    function copyTagsAction(tags: SimpleTag[], filter: FilterOutput): PlainMenuItem {
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
        filter: FilterOutput,
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

    const findSimilarAction = (filter: FilterOutput, column: string, value: string) =>
        filterActionImpl(
            <>
                <LinkIcon />
                Find similar
            </>,
            filter,
            column,
            value,
        );

    const filterAction = (filter: FilterOutput, column: string, value: string, { icon }: { icon?: ReactNode } = {}) =>
        filterActionImpl(icon ? <>{icon} Filter</> : "Filter", filter, column, value);

    const dateWithinAction = (
        filter: FilterOutput,
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

    function createdAtAction(filter: FilterOutput, createdAt: Date): GroupedMenuItem {
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
        switch (tab) {
            case "domains":
                return (
                    <StatelessWorkspaceTable
                        key={"domain-table"}
                        {...domainsTable}
                        columnsTemplate={"min-content 1fr 1fr 1fr 3.5em 4em 2.25em"}
                        onAdd={() => setCreateForm("domains")}
                        filter={domainFilter}
                    >
                        <div className={"workspace-table-header"}>
                            <MultiSelectButton
                                items={domains}
                                uuids={selectedUuids.domains}
                                setUuids={(domains) => setSelectedUuids({ ...selectedUuids, domains })}
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
                                    if (selected?.type !== "domains") {
                                        setDetailTab("general");
                                    }
                                    setSelected({ type: "domains", uuid: domain.uuid });
                                }}
                                menu={[
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
                                                return ObjectFns.uniqueObjects([
                                                    ...data.directHosts.map((h) => ["ips", h.ipAddr]),
                                                    ...data.indirectHosts.map((h) => ["ips", h.ipAddr]),
                                                    ...data.directHosts.map((h) => ["ips.os", h.osType]),
                                                    ...data.indirectHosts.map((h) => ["ips.os", h.osType]),
                                                ]).map(([k, v]) => findSimilarAction(domainFilter, k, v));
                                            })
                                            .catch((e) => [["Failed loading hosts", undefined]]),
                                    createdAtAction(domainFilter, domain.createdAt),
                                ]}
                            >
                                <SelectButton
                                    uuid={domain.uuid}
                                    uuids={selectedUuids.domains}
                                    setUuids={(domains) => setSelectedUuids({ ...selectedUuids, domains })}
                                />
                                <Domain domain={domain} />
                                <TagList tags={domain.tags} globalFilter={globalFilter} filter={domainFilter} />
                                <div>{domain.comment}</div>
                                <span className="workspace-data-certainty-icon icon"></span>
                                {domain.certainty === "Unverified"
                                    ? CertaintyIcon({ certaintyType: "Unverified" })
                                    : CertaintyIcon({ certaintyType: "Verified" })}
                                <AttackButton
                                    workspaceUuid={workspace}
                                    targetUuid={domain.uuid}
                                    targetType={"domain"}
                                />
                            </ContextMenu>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "hosts":
                return (
                    <StatelessWorkspaceTable
                        key={"host-table"}
                        {...hostsTable}
                        columnsTemplate={"min-content 35ch 2em 1fr 1fr 3.5em 4em 2.25em"}
                        onAdd={() => setCreateForm("hosts")}
                        filter={hostFilter}
                    >
                        <div className={"workspace-table-header"}>
                            <MultiSelectButton
                                items={hosts}
                                uuids={selectedUuids.hosts}
                                setUuids={(hosts) => setSelectedUuids({ ...selectedUuids, hosts })}
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
                                    if (selected?.type !== "hosts") {
                                        setDetailTab("general");
                                    }
                                    setSelected({ type: "hosts", uuid: host.uuid });
                                }}
                                menu={[
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
                                                ];
                                            })
                                            .catch((e) => [["Failed loading hosts", undefined]]),
                                    createdAtAction(hostFilter, host.createdAt),
                                ]}
                            >
                                <SelectButton
                                    uuid={host.uuid}
                                    uuids={selectedUuids.hosts}
                                    setUuids={(hosts) => setSelectedUuids({ ...selectedUuids, hosts })}
                                />
                                <IpAddr host={host} />
                                <OsIcon tooltip os={host.osType} size="2em" />
                                <TagList tags={host.tags} globalFilter={globalFilter} filter={hostFilter} />
                                <div>{host.comment}</div>
                                <span className="workspace-data-certainty-icon icon"></span>
                                {host.certainty === "Verified"
                                    ? CertaintyIcon({ certaintyType: "Verified" })
                                    : host.certainty === "Historical"
                                      ? CertaintyIcon({ certaintyType: "Historical" })
                                      : CertaintyIcon({ certaintyType: "SupposedTo" })}
                                <AttackButton workspaceUuid={workspace} targetUuid={host.uuid} targetType={"host"} />
                            </ContextMenu>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "ports":
                return (
                    <StatelessWorkspaceTable
                        key={"port-table"}
                        {...portsTable}
                        columnsTemplate={"min-content 5ch 3.75em 30ch 1fr 1fr 3.5em 4em 2.25em"}
                        onAdd={() => setCreateForm("ports")}
                        filter={portFilter}
                    >
                        <div className={"workspace-table-header"}>
                            <MultiSelectButton
                                items={ports}
                                uuids={selectedUuids.ports}
                                setUuids={(ports) => setSelectedUuids({ ...selectedUuids, ports })}
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
                                    if (selected?.type !== "ports") {
                                        setDetailTab("general");
                                    }
                                    setSelected({ type: "ports", uuid: port.uuid });
                                }}
                                menu={[
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
                                                ];
                                            })
                                            .catch((e) => [["Failed loading hosts", undefined]]),
                                    createdAtAction(portFilter, port.createdAt),
                                ]}
                            >
                                <SelectButton
                                    uuid={port.uuid}
                                    uuids={selectedUuids.ports}
                                    setUuids={(ports) => setSelectedUuids({ ...selectedUuids, ports })}
                                />
                                <PortNumber port={port} />
                                <span>{port.protocol.toUpperCase()}</span>
                                <IpAddr host={port.host} />
                                <TagList tags={port.tags} globalFilter={globalFilter} filter={portFilter} />
                                <span>{port.comment}</span>
                                <span className="workspace-data-certainty-icon icon"></span>
                                {port.certainty === "Verified"
                                    ? CertaintyIcon({ certaintyType: "Verified" })
                                    : port.certainty === "Historical"
                                      ? CertaintyIcon({ certaintyType: "Historical" })
                                      : CertaintyIcon({ certaintyType: "SupposedTo" })}
                                <AttackButton workspaceUuid={workspace} targetUuid={port.uuid} targetType={"port"} />
                            </ContextMenu>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "services":
                return (
                    <StatelessWorkspaceTable
                        key={"service-table"}
                        {...servicesTable}
                        columnsTemplate={"min-content 0.8fr 30ch 5ch 3.75em 2em 2em 1fr 1fr 3.5em 4em 2.25em"}
                        onAdd={() => setCreateForm("services")}
                        filter={serviceFilter}
                    >
                        <div className={"workspace-table-header"}>
                            <MultiSelectButton
                                items={services}
                                uuids={selectedUuids.services}
                                setUuids={(services) => setSelectedUuids({ ...selectedUuids, services })}
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
                                    if (selected?.type !== "services") {
                                        setDetailTab("general");
                                    }
                                    setSelected({ type: "services", uuid: service.uuid });
                                }}
                                menu={[
                                    copyTagsAction(service.tags, serviceFilter),
                                    filterAction(serviceFilter, "service", service.name),
                                    filterAction(serviceFilter, "ips", service.host.ipAddr),
                                    filterAction(serviceFilter, "ips.os", service.host.osType),
                                    ...(service.port
                                        ? [filterAction(serviceFilter, "ports", service.port.port + "")]
                                        : []),
                                    ...(service.protocols
                                        ? (() => {
                                              let res = [];
                                              let p = service.protocols as any;
                                              if (p.sctp) {
                                                  res.push(filterAction(serviceFilter, "protocols", "Sctp"));
                                              } else if (p.tcp) {
                                                  res.push(filterAction(serviceFilter, "protocols", "Tcp"));
                                              } else if (p.udp) {
                                                  res.push(filterAction(serviceFilter, "protocols", "Udp"));
                                              } else if (p.unknown) {
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
                                    uuids={selectedUuids.services}
                                    setUuids={(services) => setSelectedUuids({ ...selectedUuids, services })}
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
                                <AttackButton
                                    workspaceUuid={workspace}
                                    targetUuid={service.uuid}
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
            case "domains":
                return (
                    <WorkspaceDataDomainDetails
                        domain={selected.uuid}
                        updateDomain={domainsTable.updateItem}
                        tab={detailTab}
                    />
                );
            case "hosts":
                return (
                    <WorkspaceDataHostDetails host={selected.uuid} updateHost={hostsTable.updateItem} tab={detailTab} />
                );
            case "ports":
                return (
                    <WorkspaceDataPortDetails port={selected.uuid} updatePort={portsTable.updateItem} tab={detailTab} />
                );
            case "services":
                return (
                    <WorkspaceDataServiceDetails
                        service={selected.uuid}
                        updateService={servicesTable.updateItem}
                        tab={detailTab}
                    />
                );
            case undefined:
                return null;
            default:
                return "Unimplemented";
        }
    })();
    const createElement = (() => {
        switch (createForm) {
            case null:
                return null;
            case "domains":
                return (
                    <CreateDomainForm
                        onSubmit={() => {
                            setCreateForm(null);
                            domainsTable.reload();
                        }}
                    />
                );
            case "hosts":
                return (
                    <CreateHostForm
                        onSubmit={() => {
                            setCreateForm(null);
                            hostsTable.reload();
                        }}
                    />
                );
            case "ports":
                return (
                    <CreatePortForm
                        onSubmit={() => {
                            setCreateForm(null);
                            hostsTable.reload();
                            portsTable.reload();
                        }}
                    />
                );
            case "services":
                return (
                    <CreateServiceForm
                        onSubmit={() => {
                            setCreateForm(null);
                            hostsTable.reload();
                            portsTable.reload();
                            servicesTable.reload();
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
                    <div className="tabs-selector-container">
                        {Object.entries(TABS).map(([key, displayName]) => (
                            <div
                                className={"tabs " + (tab !== key ? "" : "selected-tab")}
                                onClick={() => setTab(key as keyof typeof TABS)}
                            >
                                <h3 className={"heading"}>{displayName}</h3>
                            </div>
                        ))}
                    </div>
                    {tableElement}
                </div>
                <div className={"workspace-data-details pane"}>
                    {ObjectFns.isEmpty(selectedUuids.domains) &&
                    ObjectFns.isEmpty(selectedUuids.hosts) &&
                    ObjectFns.isEmpty(selectedUuids.ports) &&
                    ObjectFns.isEmpty(selectedUuids.services) ? (
                        selected ? (
                            <>
                                <h2 className={"sub-heading"}>
                                    {selected.type === "domains" ? (
                                        <span>Domain </span>
                                    ) : selected.type === "hosts" ? (
                                        <span>Host </span>
                                    ) : selected.type === "ports" ? (
                                        <span>Port </span>
                                    ) : (
                                        <span>Service </span>
                                    )}
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
                                if (!ObjectFns.isEmpty(selectedUuids.domains)) domainsTable.reload();
                                if (!ObjectFns.isEmpty(selectedUuids.hosts)) hostsTable.reload();
                                if (!ObjectFns.isEmpty(selectedUuids.ports)) portsTable.reload();
                                if (!ObjectFns.isEmpty(selectedUuids.services)) servicesTable.reload();
                            }}
                            onDelete={() => {
                                domainsTable.reload();
                                hostsTable.reload();
                                portsTable.reload();
                                servicesTable.reload();
                                setSelected(null);
                            }}
                        />
                    )}
                </div>
            </div>
            <Popup nested modal open={createForm !== null} onClose={() => setCreateForm(null)}>
                {createElement}
            </Popup>
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

type CertaintyIconProps = {
    certaintyType:
        | "Verified"
        | "Unverified"
        | "SupposedTo"
        | "MaybeVerified"
        | "DefinitelyVerified"
        | "Historical"
        | "UnknownService";
    nameVisible?: true | undefined;
};

export function CertaintyIcon(props: CertaintyIconProps) {
    const { certaintyType, nameVisible } = props;

    switch (certaintyType) {
        case "Verified":
            return (
                <Popup
                    trigger={
                        <span className="workspace-data-certainty-icon icon">
                            <VerifiedIcon />
                            {nameVisible !== undefined && nameVisible ? <span> Verified</span> : undefined}
                        </span>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className="pane-thin">
                        <h2 className="sub-heading">Verified</h2>
                        <span>{/*TODO insert description*/}Description</span>
                    </div>
                </Popup>
            );
        case "DefinitelyVerified":
            return (
                <Popup
                    trigger={
                        <span className="workspace-data-certainty-icon icon">
                            <div>
                                <VerifiedIcon />
                                <span className="workspace-data-certainty-letter">D</span>
                            </div>
                            {nameVisible !== undefined && nameVisible ? <span>Definitely Verified</span> : undefined}
                        </span>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className="pane-thin">
                        <h2 className="sub-heading">Definitely Verified</h2>
                        <span>{/*TODO insert description*/}Description</span>
                    </div>
                </Popup>
            );
        case "MaybeVerified":
            return (
                <Popup
                    trigger={
                        <span className="workspace-data-certainty-icon icon">
                            <div>
                                <VerifiedIcon />
                                <span className="workspace-data-certainty-letter">M</span>
                            </div>
                            {nameVisible !== undefined && nameVisible ? <span>Maybe Verified</span> : undefined}
                        </span>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className="pane-thin">
                        <h2 className="sub-heading">Maybe Verified</h2>
                        <span>{/*TODO insert description*/}Description</span>
                    </div>
                </Popup>
            );
        case "Unverified":
            return (
                <Popup
                    trigger={
                        <span className="workspace-data-certainty-icon icon">
                            <UnverifiedIcon />
                            {nameVisible !== undefined && nameVisible ? <span>Unverified</span> : undefined}
                        </span>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className="pane-thin">
                        <h2 className="sub-heading">Unverified</h2>
                        <span>{/*TODO insert description*/}Description</span>
                    </div>
                </Popup>
            );
        case "SupposedTo":
            return (
                <Popup
                    trigger={
                        <span className="workspace-data-certainty-icon icon">
                            <span className="workspace-data-certainty-letter">S</span>
                            {nameVisible !== undefined && nameVisible ? <span>Supposed to</span> : undefined}
                        </span>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className="pane-thin">
                        <h2 className="sub-heading">Supposed to</h2>
                        <span>{/*TODO insert description*/}Description</span>
                    </div>
                </Popup>
            );
        case "Historical":
            return (
                <Popup
                    trigger={
                        <span className="workspace-data-certainty-icon icon">
                            <HistoricalIcon />
                            {nameVisible !== undefined && nameVisible ? <span>Historical</span> : undefined}
                        </span>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className="pane-thin">
                        <h2 className="sub-heading">Historical</h2>
                        <span>{/*TODO insert description*/}Description</span>
                    </div>
                </Popup>
            );
        case "UnknownService":
            return (
                <Popup
                    trigger={
                        <span className="workspace-data-certainty-icon icon">
                            <UnknownIcon />
                            {nameVisible !== undefined && nameVisible ? <span>Unknown Service</span> : undefined}
                        </span>
                    }
                    position={"bottom center"}
                    on={"hover"}
                    arrow={true}
                >
                    <div className="pane-thin">
                        <h2 className="sub-heading">Unknown Service</h2>
                        <span>{/*TODO insert description*/}Description</span>
                    </div>
                </Popup>
            );
        default:
            return "Unimplemented";
    }
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

    const domainsLen = ObjectFns.len(selectedUuids.domains);
    const hostsLen = ObjectFns.len(selectedUuids.hosts);
    const portsLen = ObjectFns.len(selectedUuids.ports);
    const servicesLen = ObjectFns.len(selectedUuids.services);
    const totalLen = domainsLen + hostsLen + portsLen + servicesLen;

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
                                  onClick={() => setSelectedUuids({ ...selectedUuids, domains: {} })}
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
                                  onClick={() => setSelectedUuids({ ...selectedUuids, hosts: {} })}
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
                                  onClick={() => setSelectedUuids({ ...selectedUuids, ports: {} })}
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
                                  onClick={() => setSelectedUuids({ ...selectedUuids, services: {} })}
                              >
                                  Unselect all services
                              </button>,
                          ]}
                    <span className={"workspace-data-multi-select-total"}>Total</span>
                    <span className={"workspace-data-multi-select-total"}>{totalLen}</span>
                    <button
                        type={"button"}
                        className={"button workspace-data-multi-select-total"}
                        onClick={() => setSelectedUuids({ domains: {}, hosts: {}, ports: {}, services: {} })}
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
                                    domains: Object.keys(selectedUuids.domains),
                                    hosts: Object.keys(selectedUuids.hosts),
                                    ports: Object.keys(selectedUuids.ports),
                                    services: Object.keys(selectedUuids.services),
                                },
                            );
                        }}
                    >
                        <AttackIcon />
                        Attack selected
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
                modal={true}
                nested={true}
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
                                let stillSelected: SelectedUuids = { domains: {}, hosts: {}, ports: {}, services: {} };
                                if (domainsLen !== 0) {
                                    Object.keys(selectedUuids.domains).map((u) => {
                                        promises.push(
                                            Api.workspaces.domains.delete(workspace, u).then((result) => {
                                                if (result.is_err()) {
                                                    numErr += 1;
                                                    handleApiError(result);
                                                    stillSelected.domains[u] = true;
                                                } else if (result.is_ok()) {
                                                    numOk += 1;
                                                }
                                            }),
                                        );
                                    });
                                }
                                if (hostsLen !== 0) {
                                    Object.keys(selectedUuids.hosts).map((u) => {
                                        promises.push(
                                            Api.workspaces.hosts.delete(workspace, u).then((result) => {
                                                if (result.is_err()) {
                                                    numErr += 1;
                                                    handleApiError(result);
                                                    stillSelected.hosts[u] = true;
                                                } else if (result.is_ok()) {
                                                    numOk += 1;
                                                }
                                            }),
                                        );
                                    });
                                }
                                if (portsLen !== 0) {
                                    Object.keys(selectedUuids.ports).map((u) => {
                                        promises.push(
                                            Api.workspaces.ports.delete(workspace, u).then((result) => {
                                                if (result.is_err()) {
                                                    numErr += 1;
                                                    handleApiError(result);
                                                    stillSelected.ports[u] = true;
                                                } else if (result.is_ok()) {
                                                    numOk += 1;
                                                }
                                            }),
                                        );
                                    });
                                }
                                if (servicesLen !== 0) {
                                    Object.keys(selectedUuids.services).map((u) => {
                                        promises.push(
                                            Api.workspaces.services.delete(workspace, u).then((result) => {
                                                if (result.is_err()) {
                                                    numErr += 1;
                                                    handleApiError(result);
                                                    stillSelected.services[u] = true;
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

async function updateTags(workspace: string, uuids: SelectedUuids, strategy: UpdateStrategy, tags: Array<SimpleTag>) {
    await Promise.all(
        Object.keys(uuids.domains).map((uuid) =>
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
        Object.keys(uuids.hosts).map((uuid) =>
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
        Object.keys(uuids.ports).map((uuid) =>
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
        Object.keys(uuids.services).map((uuid) =>
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
