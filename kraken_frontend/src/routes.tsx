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
import Workspace from "./views/workspace/workspace";
import OauthRequest from "./views/oauth-request";
import Menu from "./views/menu";

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
                    <Home />
                </div>
                <WorkspaceOverview />
            </>
        ),
    }),
    WORKSPACE_SETTINGS: ROUTER.add({
        url: "workspaces/{uuid}/settings",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <>
                <div className={"content-container"}>
                    <Workspace uuid={uuid} />
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
                    <Workspace uuid={uuid} />
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
                    <Workspace uuid={uuid} />
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
                    <Workspace uuid={uuid} />
                </div>
                <Menu />
            </>
        ),
    }),
    ATTACKS: ROUTER.add({
        url: "attacks",
        parser: {},
        render: () => (
            <>
                <div className={"content-container"}>
                    <Attacks />
                </div>
                <Menu />
            </>
        ),
    }),
    ATTACK_RESULTS: ROUTER.add({
        url: "attacks/{uuid}",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <>
                <div className={"content-container"}>
                    <AttackResults attackUuid={uuid} />
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
};

ROUTER.finish();
