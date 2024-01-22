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
import FilterInput from "./components/filter-input";
import { handleApiError, ObjectFns } from "../../utils/helper";
import Checkbox from "../../components/checkbox";
import EditableTags from "./components/editable-tags";
import { toast } from "react-toastify";
import promise = toast.promise;
import { Toast } from "react-toastify/dist/components";
import { ApiError } from "../../api/error";
import { Result } from "../../utils/result";

const TABS = { domains: "Domains", hosts: "Hosts", ports: "Ports", services: "Services" };
const DETAILS_TAB = { general: "General", results: "Results", relations: "Relations" };
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

    const [globalFilter, setGlobalFilter] = React.useState("");
    const [domainFilter, setDomainFilter] = React.useState("");
    const [hostFilter, setHostFilter] = React.useState("");
    const [portFilter, setPortFilter] = React.useState("");
    const [serviceFilter, setServiceFilter] = React.useState("");

    const { items: domains, ...domainsTable } = useTable<FullDomain>(
        (limit, offset) => Api.workspaces.domains.all(workspace, limit, offset, { globalFilter, domainFilter }),
        [workspace, globalFilter, domainFilter]
    );
    const { items: hosts, ...hostsTable } = useTable<FullHost>(
        (limit, offset) => Api.workspaces.hosts.all(workspace, limit, offset, { globalFilter, hostFilter }),
        [workspace, globalFilter, hostFilter]
    );
    const { items: ports, ...portsTable } = useTable<FullPort>(
        (limit, offset) => Api.workspaces.ports.all(workspace, limit, offset, { globalFilter, portFilter }),
        [workspace, globalFilter, portFilter]
    );
    const { items: services, ...servicesTable } = useTable<FullService>(
        (limit, offset) => Api.workspaces.services.all(workspace, limit, offset, { globalFilter, serviceFilter }),
        [workspace, globalFilter, serviceFilter]
    );

    const tableElement = (() => {
        switch (tab) {
            case "domains":
                return (
                    <StatelessWorkspaceTable
                        key={"domain-table"}
                        {...domainsTable}
                        columnsTemplate={"min-content 1fr 1fr 1fr 1fr min-content"}
                        onAdd={() => setCreateForm("domains")}
                        applyFilter={(value) => {
                            setDomainFilter(value);
                            domainsTable.setOffset(0);
                        }}
                        filterTarget={"domain"}
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
                            <span>Attacks</span>
                            <span />
                        </div>
                        {domains.map((domain) => (
                            <div
                                className={"workspace-table-row"}
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
                                <span>{domain.domain}</span>
                                <TagList tags={domain.tags} />
                                <span>{domain.comment}</span>
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
                        columnsTemplate={"min-content 39ch 1fr 1fr 1fr min-content"}
                        onAdd={() => setCreateForm("hosts")}
                        applyFilter={(value) => {
                            setHostFilter(value);
                            hostsTable.setOffset(0);
                        }}
                        filterTarget={"host"}
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
                            <span>Attacks</span>
                            <span />
                        </div>
                        {hosts.map((host) => (
                            <div
                                className={"workspace-table-row"}
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
                                <span>{host.ipAddr}</span>
                                <TagList tags={host.tags} />
                                <span>{host.comment}</span>
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
                        columnsTemplate={"min-content 5ch 8ch 39ch 1fr 1fr 1fr min-content"}
                        onAdd={() => setCreateForm("ports")}
                        applyFilter={(value) => {
                            setPortFilter(value);
                            portsTable.setOffset(0);
                        }}
                        filterTarget={"port"}
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
                            <span>Attacks</span>
                            <span />
                        </div>
                        {ports.map((port) => (
                            <div
                                className={"workspace-table-row"}
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
                        columnsTemplate={"min-content 1fr 39ch 5ch 1fr 1fr 1fr min-content"}
                        onAdd={() => setCreateForm("services")}
                        applyFilter={(value) => {
                            setServiceFilter(value);
                            servicesTable.setOffset(0);
                        }}
                        filterTarget={"service"}
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
                            <span>Tags</span>
                            <span>Comment</span>
                            <span>Attacks</span>
                            <span />
                        </div>
                        {services.map((service) => (
                            <div
                                className={"workspace-table-row"}
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
                                <TagList tags={service.tags} />
                                <span>{service.comment}</span>
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
                    <FilterInput
                        placeholder={"Global Filter..."}
                        applyFilter={(value) => {
                            setGlobalFilter(value);
                            domainsTable.setOffset(0);
                            hostsTable.setOffset(0);
                            portsTable.setOffset(0);
                            servicesTable.setOffset(0);
                        }}
                        target={"global"}
                    />
                </div>
                <div className={"workspace-data-selector"}>
                    {Object.entries(TABS).map(([key, displayName]) => (
                        <div
                            className={"pane" + (tab !== key ? "" : " workspace-data-selected-tab")}
                            onClick={() => setTab(key as keyof typeof TABS)}
                        >
                            <h3 className={"heading"}>{displayName}</h3>
                        </div>
                    ))}
                </div>
                {tableElement}
                <div className={"workspace-data-details pane"}>
                    {ObjectFns.isEmpty(selectedUuids.domains) &&
                    ObjectFns.isEmpty(selectedUuids.hosts) &&
                    ObjectFns.isEmpty(selectedUuids.ports) &&
                    ObjectFns.isEmpty(selectedUuids.services) ? (
                        selected ? (
                            <>
                                <h2 className={"sub-heading"}>Details</h2>
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
