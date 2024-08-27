import React, { useEffect } from "react";
import { Api, UUID } from "../../api/api";
import { FullHost } from "../../api/generated";
import Input from "../../components/input";
import OsIcon from "../../components/os-icon";
import { ROUTES } from "../../routes";
import "../../styling/workspace-host.css";
import ArrowLeftIcon from "../../svg/arrow-left";

import { compareHost } from "../../utils/data-sorter";
import { handleApiError } from "../../utils/helper";
import TagList from "./components/tag-list";
import { WORKSPACE_CONTEXT } from "./workspace";
import { WorkspaceHostDomains } from "./workspace-host/workspace-host-domains";
import { WorkspaceHostHttpServices } from "./workspace-host/workspace-host-http-services";
import { WorkspaceHostPorts } from "./workspace-host/workspace-host-ports";
import { WorkspaceHostServices } from "./workspace-host/workspace-host-services";

const TABS = {
    domains: "Domains",
    ports: "Ports",
    services: "Services",
    httpServices: "HTTP Services",
    other: "Other",
};

type WorkspaceProps = {
    uuid: UUID;
};

export default function WorkspaceHost(props: WorkspaceProps) {
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);

    const [selectedTab, setSelectedTab] = React.useState<keyof typeof TABS>("domains");
    const [host, setHost] = React.useState<FullHost | null>(null);
    const [hostList, setHostList] = React.useState<Array<FullHost>>([]);
    const [searchTerm, setSearchTerm] = React.useState("");

    function getHostList() {
        return Api.workspaces.hosts.all(workspace, 1000, 0).then(
            handleApiError(({ items }) => {
                setHostList(items);
            }),
        );
    }

    function getHost() {
        return Api.workspaces.hosts.get(workspace, props.uuid).then(handleApiError(setHost));
    }

    useEffect(() => {
        getHost();
        getHostList();
    }, [props.uuid]);

    const table = (() => {
        switch (selectedTab) {
            case "domains":
                return <WorkspaceHostDomains host={host} />;
            case "ports":
                return <WorkspaceHostPorts host={host} />;
            case "services":
                return <WorkspaceHostServices host={host} />;
            case "httpServices":
                return <WorkspaceHostHttpServices host={host} />;
            default:
                return "Unimplemented";
        }
    })();

    const selectedHost = host?.uuid;

    return (
        <div className={"workspace-host-container"}>
            <div className={"workspace-host-hosts-list"}>
                <div className={"workspace-host-hosts-list-header"}>
                    <div className={"pane workspace-host-hosts-search"}>
                        <ArrowLeftIcon
                            key={"back"}
                            onClick={() => {
                                ROUTES.WORKSPACE_HOSTS.visit({
                                    uuid: workspace,
                                });
                            }}
                        />

                        <Input
                            className={"workspace-host-search-bar"}
                            placeholder={"Search host"}
                            value={searchTerm}
                            onChange={setSearchTerm}
                        />
                    </div>
                </div>
                <div className={"workspace-host-hosts-list-entries"}>
                    {hostList
                        .filter(({ ipAddr }) => ipAddr.includes(searchTerm))
                        .sort(compareHost)
                        .map((host) => {
                            return (
                                <button
                                    key={host.uuid}
                                    className={`pane workspace-host-hosts-item ${host.uuid == selectedHost ? "selected" : ""}`}
                                    onClick={() => {
                                        ROUTES.WORKSPACE_SINGLE_HOST.visit({
                                            w_uuid: workspace,
                                            h_uuid: host.uuid,
                                        });
                                    }}
                                >
                                    <OsIcon os={host.osType} />
                                    <div className={"workspace-host-hosts-info"}>
                                        <h2 className={"sub-heading"}>{host.ipAddr}</h2>
                                        <span className="workspace-host-comment-overflow">{host.comment}</span>
                                    </div>
                                </button>
                            );
                        })}
                </div>
            </div>
            <div className={"pane workspace-host-host-container"}>
                {host !== null ? (
                    <>
                        <OsIcon os={host.osType} />
                        <div className={"workspace-host-details"}>
                            <h2 className={"heading"}>Host {host.ipAddr}</h2>
                            <span>OS: {host.osType}</span>
                            <span>Comment: {host.comment}</span>
                            <TagList tags={host.tags} />
                        </div>
                    </>
                ) : (
                    <div>Loading ..</div>
                )}
            </div>

            <div className={"workspace-host-section-selector"}>
                {Object.entries(TABS).map(([key, displayName]) => (
                    <div
                        className={"pane" + (selectedTab !== key ? "" : " workspace-host-selected-tab")}
                        onClick={() => setSelectedTab(key as keyof typeof TABS)}
                    >
                        <h3 className={"heading"}>{displayName}</h3>
                    </div>
                ))}
            </div>
            {table}

            <div className={"workspace-host-content-details pane"}>
                <h2 className={"heading"}>Details</h2>
            </div>
        </div>
    );
}
