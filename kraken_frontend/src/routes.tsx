import Home from "./views/home";
import React from "react";
import { Router } from "./utils/router";
import KrakenNetwork from "./views/kraken-network";
import Me from "./views/me";
import AdminUsers from "./views/admin/users";
import AdminWorkspaces from "./views/admin/workspaces";
import AdminGuard from "./components/admin-guard";
import Attacks from "./views/attacks";
import AttackResults from "./views/attack-results";
import WorkspaceOverview from "./views/workspace-overview";
import WorkspaceHost from "./views/workspace/workspace-host";
import OauthRequest from "./views/oauth-request";
import Menu from "./views/menu";
import Settings from "./views/settings";
import WorkspaceAttacks from "./views/workspace/workspace-attacks";
import Workspace from "./views/workspace/workspace";

export const ROUTER = new Router();

export const ROUTES = {
    HOME: ROUTER.add({
        url: "",
        parser: {},
        render: () => (
            <>
                <div className="content-container">
                    <Home />
                </div>
                <Menu />
            </>
        ),
    }),
    ME: ROUTER.add({
        url: "me",
        parser: {},
        render: () => (
            <>
                <div className="content-container">
                    <Me />
                </div>
                <Menu />
            </>
        ),
    }),
    WORKSPACES: ROUTER.add({
        url: "workspaces",
        parser: {},
        render: () => (
            <>
                <div className="content-container">
                    <WorkspaceOverview />
                </div>
                <Menu />
            </>
        ),
    }),
    WORKSPACE_SETTINGS: ROUTER.add({
        url: "workspaces/{uuid}/settings",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <>
                <div className={"content-container"}>
                    <Workspace view={"settings"} uuid={uuid} />
                </div>
                <Menu />
            </>
        ),
    }),
    WORKSPACE_DATA: ROUTER.add({
        url: "workspaces/{uuid}/data",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <>
                <div className={"content-container"}>
                    <Workspace view={"data"} uuid={uuid} />
                </div>
                <Menu />
            </>
        ),
    }),
    WORKSPACE_ATTACKS: ROUTER.add({
        url: "workspaces/{uuid}/attacks",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <>
                <div className={"content-container"}>
                    <Workspace view={"attacks"} uuid={uuid} />
                </div>
                <Menu />
            </>
        ),
    }),
    WORKSPACE_SEARCH: ROUTER.add({
        url: "workspaces/{uuid}/search",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <>
                <div className={"content-container"}>
                    <Workspace view={"search"} uuid={uuid} />
                </div>
                <Menu />
            </>
        ),
    }),
    WORKSPACE_HOSTS: ROUTER.add({
        url: "workspaces/{uuid}/hosts",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <>
                <div className={"content-container"}>
                    <Workspace view={"hosts"} uuid={uuid} />
                </div>
                <Menu />
            </>
        ),
    }),
    WORKSPACE_SINGLE_HOST: ROUTER.add({
        url: "workspaces/{w_uuid}/hosts/{h_uuid}",
        parser: { w_uuid: String, h_uuid: String },
        render: ({ w_uuid, h_uuid }) => (
            <>
                <div className={"content-container"}>
                    <Workspace view={"single_host"} uuid={w_uuid} host_uuid={h_uuid} />
                </div>
                <Menu />
            </>
        ),
    }),
    KNOWLEDGE_BASE: ROUTER.add({ url: "knowledge", parser: {}, render: () => undefined }),

    OAUTH_REQUEST: ROUTER.add({
        url: "oauth-request/{uuid}",
        parser: { uuid: String },
        render: ({ uuid }) => <OauthRequest uuid={uuid} />,
    }),

    KRAKEN_NETWORK: ROUTER.add({
        url: "kraken-network",
        parser: {},
        render: () => (
            <>
                <div className="content-container">
                    <AdminGuard>
                        <KrakenNetwork />
                    </AdminGuard>
                </div>
                <Menu />
            </>
        ),
    }),
    ADMIN_USER_MANAGEMENT: ROUTER.add({
        url: "admin/users",
        parser: {},
        render: () => (
            <>
                <div className="content-container">
                    <AdminGuard>
                        <AdminUsers />
                    </AdminGuard>
                </div>
                <Menu />
            </>
        ),
    }),
    ADMIN_WORKSPACE_MANAGEMENT: ROUTER.add({
        url: "admin/workspaces",
        parser: {},
        render: () => (
            <>
                <div className="content-container">
                    <AdminGuard>
                        <AdminWorkspaces />
                    </AdminGuard>
                </div>
                <Menu />
            </>
        ),
    }),
    ADMIN_SETTINGS: ROUTER.add({
        url: "admin/settings",
        parser: {},
        render: () => (
            <>
                <div className={"content-container"}>
                    <AdminGuard>
                        <Settings />
                    </AdminGuard>
                </div>
                <Menu />
            </>
        ),
    }),
};

ROUTER.finish();
