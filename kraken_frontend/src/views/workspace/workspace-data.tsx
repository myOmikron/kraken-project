import React, { useContext } from "react";
import "../../styling/workspace-data.css";
import { StatelessWorkspaceTable, useTable } from "./components/workspace-table";
import { Api } from "../../api/api";
import Tag from "../../components/tag";
import {
    FullDomain,
    FullHost,
    FullPort,
    FullService,
    instanceOfPortResultsPage,
    ManualHostCertainty,
    ManualPortCertainty,
    ManualServiceCertainty,
    PortProtocol,
    SimpleAggregationSource,
    SimpleTag,
} from "../../api/generated";
import { WorkspaceDataHostDetails } from "./workspace-data/workspace-data-host-details";
import { WorkspaceDataServiceDetails } from "./workspace-data/workspace-data-service-details";
import { WorkspaceDataPortDetails } from "./workspace-data/workspace-data-port-details";
import { WorkspaceDataDomainDetails } from "./workspace-data/workspace-data-domain-details";
import SourcesList from "./components/sources-list";
import TagList from "./components/tag-list";
import Popup from "reactjs-popup";
import Input from "../../components/input";
import Select, { SingleValue } from "react-select";
import { selectStyles } from "../../components/select-menu";
import { handleApiError } from "../../utils/helper";
import { toast } from "react-toastify";
import { CreateDomainForm } from "./workspace-data/workspace-data-create-domain";
import { CreateHostForm } from "./workspace-data/workspace-data-create-host";
import { CreatePortForm } from "./workspace-data/workspace-data-create-port";
import { CreateServiceForm } from "./workspace-data/workspace-data-create-service";
import { WORKSPACE_CONTEXT } from "./workspace";
import { ROUTES } from "../../routes";
import AttackIcon from "../../svg/attack";

const TABS = { domains: "Domains", hosts: "Hosts", ports: "Ports", services: "Services" };
const DETAILS_TAB = { general: "General", results: "Results", relations: "Relations" };

type WorkspaceDataProps = {};

export default function WorkspaceData(props: WorkspaceDataProps) {
    const {
        workspace: { uuid: workspace },
    } = useContext(WORKSPACE_CONTEXT);

    const [tab, setTab] = React.useState<keyof typeof TABS>("hosts");
    const [detailTab, setDetailTab] = React.useState<keyof typeof DETAILS_TAB>("general");
    const [selected, setSelected] = React.useState<{ type: keyof typeof TABS; uuid: string } | null>(null);
    const [createForm, setCreateForm] = React.useState<keyof typeof TABS | null>(null);

    const { items: domains, ...domainsTable } = useTable<FullDomain>(
        (limit, offset) => Api.workspaces.domains.all(workspace, limit, offset),
        [workspace]
    );
    const { items: hosts, ...hostsTable } = useTable<FullHost>(
        (limit, offset) => Api.workspaces.hosts.all(workspace, limit, offset),
        [workspace]
    );
    const { items: ports, ...portsTable } = useTable<FullPort>(
        (limit, offset) => Api.workspaces.ports.all(workspace, limit, offset),
        [workspace]
    );
    const { items: services, ...servicesTable } = useTable<FullService>(
        (limit, offset) => Api.workspaces.services.all(workspace, limit, offset),
        [workspace]
    );

    const tableElement = (() => {
        switch (tab) {
            case "domains":
                return (
                    <StatelessWorkspaceTable
                        {...domainsTable}
                        columnsTemplate={"1fr 1fr 1fr 1fr min-content"}
                        onAdd={() => setCreateForm("domains")}
                    >
                        <div className={"workspace-table-header"}>
                            <span>Name</span>
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
                        {...hostsTable}
                        columnsTemplate={"39ch 1fr 1fr 1fr min-content"}
                        onAdd={() => setCreateForm("hosts")}
                    >
                        <div className={"workspace-table-header"}>
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
                        {...portsTable}
                        columnsTemplate={"5ch 8ch 39ch 1fr 1fr 1fr min-content"}
                        onAdd={() => setCreateForm("ports")}
                    >
                        <div className={"workspace-table-header"}>
                            <span>Port</span>
                            <span>Protocol</span>
                            <span>Host</span>
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
                        {...servicesTable}
                        columnsTemplate={"1fr 39ch 5ch 1fr 1fr 1fr min-content"}
                        onAdd={() => setCreateForm("services")}
                    >
                        <div className={"workspace-table-header"}>
                            <span>Name</span>
                            <span>Host</span>
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
                    <h2 className={"sub-heading"}>Details</h2>
                    {selected ? (
                        <>
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
                    ) : undefined}
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
