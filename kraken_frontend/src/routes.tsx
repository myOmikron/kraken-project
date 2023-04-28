import Home from "./views/home";
import React from "react";
import { Router } from "./utils/router";
import KrakenNetwork from "./views/kraken-network";
import Me from "./views/me";
import Login from "./views/login";
import UserManagement from "./views/admin/user-management";

export const ROUTER = new Router();

export const ROUTES = {
    LOGIN: ROUTER.add({ url: "login", parser: {}, render: () => <Login /> }),
    HOME: ROUTER.add({ url: "", parser: {}, render: () => <Home /> }),
    ME: ROUTER.add({ url: "me", parser: {}, render: () => <Me /> }),
    KRAKEN_NETWORK: ROUTER.add({ url: "kraken-network", parser: {}, render: () => <KrakenNetwork /> }),

    ADMIN_USER_MANAGEMENT: ROUTER.add({ url: "admin/users", parser: {}, render: () => <UserManagement /> }),
};

ROUTER.finish();
