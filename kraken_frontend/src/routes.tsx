import Home from "./views/home";
import React from "react";
import { Router } from "./utils/router";
import KrakenNetwork from "./views/kraken-network";
import Me from "./views/me";
import AdminUsers from "./views/admin/users";
import AdminWorkspaces from "./views/admin/workspaces";
import Workspaces from "./views/workspaces";
import AdminGuard from "./components/admin-guard";
import Attacks from "./views/attacks";
import AttackResults from "./views/attack-results";

export const ROUTER = new Router();

export const ROUTES = {
    HOME: ROUTER.add({ url: "", parser: {}, render: () => <Home /> }),
    ME: ROUTER.add({ url: "me", parser: {}, render: () => <Me /> }),
    WORKSPACES: ROUTER.add({ url: "workspaces", parser: {}, render: () => <Workspaces /> }),
    ATTACKS: ROUTER.add({ url: "attacks", parser: {}, render: () => <Attacks /> }),
    ATTACK_RESULTS: ROUTER.add({
        url: "attacks/{id}",
        parser: { id: Number },
        render: ({ id }) => <AttackResults attackId={id} />,
    }),
    KNOWLEDGE_BASE: ROUTER.add({ url: "knowledge", parser: {}, render: () => undefined }),

    KRAKEN_NETWORK: ROUTER.add({
        url: "kraken-network",
        parser: {},
        render: () => (
            <AdminGuard>
                <KrakenNetwork />
            </AdminGuard>
        ),
    }),
    ADMIN_USER_MANAGEMENT: ROUTER.add({
        url: "admin/users",
        parser: {},
        render: () => (
            <AdminGuard>
                <AdminUsers />
            </AdminGuard>
        ),
    }),
    ADMIN_WORKSPACE_MANAGEMENT: ROUTER.add({
        url: "admin/workspaces",
        parser: {},
        render: () => (
            <AdminGuard>
                <AdminWorkspaces />
            </AdminGuard>
        ),
    }),
};

ROUTER.finish();
