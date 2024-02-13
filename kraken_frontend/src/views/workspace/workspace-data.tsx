import React from "react";
import "../../styling/workspace-data.css";
import { StatelessWorkspaceTable, useTable } from "./components/workspace-table";
import { Api } from "../../api/api";
import { FullDomain, FullHost, FullPort, FullService, SimpleTag, TagType } from "../../api/generated";
import { WorkspaceDataHostDetails } from "./workspace-data/workspace-data-host-details";
import { WorkspaceDataServiceDetails } from "./workspace-data/workspace-data-service-details";
import { WorkspaceDataPortDetails } from "./workspace-data/workspace-data-port-details";
import { WorkspaceDataDomainDetails } from "./workspace-data/workspace-data-domain-details";
import SourcesList from "./components/sources-list";
import TagList from "./components/tag-list";
import Popup from "reactjs-popup";
import { CreateDomainForm } from "./workspace-data/workspace-data-create-domain";
import { CreateHostForm } from "./workspace-data/workspace-data-create-host";
import { CreatePortForm } from "./workspace-data/workspace-data-create-port";
import { CreateServiceForm } from "./workspace-data/workspace-data-create-service";
import { WORKSPACE_CONTEXT } from "./workspace";
import { ROUTES } from "../../routes";
import AttackIcon from "../../svg/attack";
import FilterInput, { useFilter } from "./components/filter-input";
import { handleApiError, ObjectFns } from "../../utils/helper";
import Checkbox from "../../components/checkbox";
import EditableTags from "./components/editable-tags";
import { toast } from "react-toastify";
import UnverifiedIcon from "../../svg/unverified";
import VerifiedIcon from "../../svg/verified";
import HistoricalIcon from "../../svg/historical";
import UnknownIcon from "../../svg/unknown";
import SelectableText from "../../components/selectable-text";

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

    const globalFilter = useFilter("global");
    const domainFilter = useFilter("domain");
    const hostFilter = useFilter("host");
    const portFilter = useFilter("port");
    const serviceFilter = useFilter("service");

    const { items: domains, ...domainsTable } = useTable<FullDomain>(
        (limit, offset) =>
            Api.workspaces.domains.all(workspace, limit, offset, {
                globalFilter: globalFilter.applied,
                domainFilter: domainFilter.applied,
            }),
        [workspace, globalFilter.applied, domainFilter.applied]
    );
    const { items: hosts, ...hostsTable } = useTable<FullHost>(
        (limit, offset) =>
            Api.workspaces.hosts.all(workspace, limit, offset, {
                globalFilter: globalFilter.applied,
                hostFilter: hostFilter.applied,
            }),
        [workspace, globalFilter.applied, hostFilter.applied]
    );
    const { items: ports, ...portsTable } = useTable<FullPort>(
        (limit, offset) =>
            Api.workspaces.ports.all(workspace, limit, offset, {
                globalFilter: globalFilter.applied,
                portFilter: portFilter.applied,
            }),
        [workspace, globalFilter.applied, portFilter.applied]
    );
    const { items: services, ...servicesTable } = useTable<FullService>(
        (limit, offset) =>
            Api.workspaces.services.all(workspace, limit, offset, {
                globalFilter: globalFilter.applied,
                serviceFilter: serviceFilter.applied,
            }),
        [workspace, globalFilter.applied, serviceFilter.applied]
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

    const tableElement = (() => {
        switch (tab) {
            case "domains":
                return (
                    <StatelessWorkspaceTable
                        key={"domain-table"}
                        {...domainsTable}
                        columnsTemplate={"min-content 1fr 1fr 1fr 0.2fr 0.2fr 1fr 0.15fr"}
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
                            <span>Attacks</span>
                            <span />
                        </div>
                        {domains.map((domain) => (
                            <div
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
                            >
                                <SelectButton
                                    uuid={domain.uuid}
                                    uuids={selectedUuids.domains}
                                    setUuids={(domains) => setSelectedUuids({ ...selectedUuids, domains })}
                                />
                                <SelectableText>{domain.domain}</SelectableText>
                                <TagList tags={domain.tags} />
                                <div>{domain.comment}</div>
                                <span className="workspace-data-certainty-icon"></span>
                                {domain.certainty === "Unverified"
                                    ? CertaintyIcon({ certaintyType: "Unverified" })
                                    : CertaintyIcon({ certaintyType: "Verified" })}
                                <SourcesList sources={domain.sources} />
                                <AttackButton
                                    workspaceUuid={workspace}
                                    targetUuid={domain.uuid}
                                    targetType={"domain"}
                                />
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "hosts":
                return (
                    <StatelessWorkspaceTable
                        key={"host-table"}
                        {...hostsTable}
                        columnsTemplate={"min-content 35ch 1fr 1fr 0.2fr 0.2fr 1fr 0.15fr"}
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
                            <span>Tags</span>
                            <span>Comment</span>
                            <span>Severity</span>
                            <span>Certainty</span>
                            <span>Attacks</span>
                            <span />
                        </div>
                        {hosts.map((host) => (
                            <div
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
                            >
                                <SelectButton
                                    uuid={host.uuid}
                                    uuids={selectedUuids.hosts}
                                    setUuids={(hosts) => setSelectedUuids({ ...selectedUuids, hosts })}
                                />
                                <SelectableText>{host.ipAddr}</SelectableText>
                                <TagList tags={host.tags} />
                                <div>{host.comment}</div>
                                <span className="workspace-data-certainty-icon"></span>
                                {host.certainty === "Verified"
                                    ? CertaintyIcon({ certaintyType: "Verified" })
                                    : host.certainty === "Historical"
                                    ? CertaintyIcon({ certaintyType: "Historical" })
                                    : CertaintyIcon({ certaintyType: "SupposedTo" })}
                                <SourcesList sources={host.sources} />
                                <AttackButton workspaceUuid={workspace} targetUuid={host.uuid} targetType={"host"} />
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "ports":
                return (
                    <StatelessWorkspaceTable
                        key={"port-table"}
                        {...portsTable}
                        columnsTemplate={"min-content 5ch 8ch 35ch 1fr 1fr 0.2fr 0.2fr 1fr 0.15fr"}
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
                            <span>Attacks</span>
                            <span />
                        </div>
                        {ports.map((port) => (
                            <div
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
                            >
                                <SelectButton
                                    uuid={port.uuid}
                                    uuids={selectedUuids.ports}
                                    setUuids={(ports) => setSelectedUuids({ ...selectedUuids, ports })}
                                />
                                <span>{port.port}</span>
                                <span>{port.protocol.toUpperCase()}</span>
                                <span>{port.host.ipAddr}</span>
                                <TagList tags={port.tags} />
                                <span>{port.comment}</span>
                                <span className="workspace-data-certainty-icon"></span>
                                {port.certainty === "Verified"
                                    ? CertaintyIcon({ certaintyType: "Verified" })
                                    : port.certainty === "Historical"
                                    ? CertaintyIcon({ certaintyType: "Historical" })
                                    : CertaintyIcon({ certaintyType: "SupposedTo" })}
                                <SourcesList sources={port.sources} />
                                <AttackButton workspaceUuid={workspace} targetUuid={port.uuid} targetType={"port"} />
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "services":
                return (
                    <StatelessWorkspaceTable
                        key={"service-table"}
                        {...servicesTable}
                        columnsTemplate={"min-content 0.8fr 30ch 5ch 10ch 1fr 1fr 0.2fr 0.2fr 1fr 0.15fr"}
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
                            <span>Tags</span>
                            <span>Comment</span>
                            <span>Severity</span>
                            <span>Certainty</span>
                            <span>Attacks</span>
                            <span />
                        </div>
                        {services.map((service) => (
                            <div
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
                            >
                                <SelectButton
                                    uuid={service.uuid}
                                    uuids={selectedUuids.services}
                                    setUuids={(services) => setSelectedUuids({ ...selectedUuids, services })}
                                />
                                <span>{service.name}</span>
                                <span>{service.host.ipAddr}</span>
                                <span>{service.port?.port}</span>
                                <span>{service.port?.protocol}</span>
                                <TagList tags={service.tags} />
                                <span>{service.comment}</span>
                                <span className="workspace-data-certainty-icon"></span>
                                {service.certainty === "Historical"
                                    ? CertaintyIcon({ certaintyType: "Historical" })
                                    : service.certainty === "SupposedTo"
                                    ? CertaintyIcon({ certaintyType: "SupposedTo" })
                                    : service.certainty === "UnknownService"
                                    ? CertaintyIcon({ certaintyType: "UnknownService" })
                                    : service.certainty === "MaybeVerified"
                                    ? CertaintyIcon({ certaintyType: "MaybeVerified" })
                                    : CertaintyIcon({ certaintyType: "DefinitelyVerified" })}
                                <SourcesList sources={service.sources} />
                                <AttackButton
                                    workspaceUuid={workspace}
                                    targetUuid={service.uuid}
                                    targetType={"service"}
                                />
                            </div>
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
                <div className="workspace-findings-data-table">
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
                        <span className="workspace-data-certainty-icon">
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
                        <span className="workspace-data-certainty-icon">
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
                        <span className="workspace-data-certainty-icon">
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
                        <span className="workspace-data-certainty-icon">
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
                        <span className="workspace-data-certainty-icon">
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
                        <span className="workspace-data-certainty-icon">
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
                        <span className="workspace-data-certainty-icon">
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
                                            })
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
                                            })
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
                                            })
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
                                            })
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
    newTags: Array<SimpleTag>
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
            })
        )
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
            })
        )
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
            })
        )
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
            })
        )
    );
}

function addStrategy(curTags: Array<SimpleTag>, newTags: Array<SimpleTag>) {
    const workspaceTags = [
        ...new Set( // Use Set to eliminate duplicates
            [...curTags, ...newTags].filter(({ tagType }) => tagType === TagType.Workspace).map(({ uuid }) => uuid)
        ).keys(),
    ];
    const globalTags = [
        ...new Set( // Use Set to eliminate duplicates
            [...curTags, ...newTags].filter(({ tagType }) => tagType === TagType.Global).map(({ uuid }) => uuid)
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
