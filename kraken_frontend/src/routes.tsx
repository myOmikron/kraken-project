import Home from "./views/home";
import React from "react";
import { Router } from "./utils/router";
import Me from "./views/me";
import Login from "./views/login";

export const ROUTER = new Router();

export const ROUTES = {
    LOGIN: ROUTER.add({ url: "login", parser: {}, render: () => <Login /> }),
    HOME: ROUTER.add({ url: "", parser: {}, render: () => <Home /> }),
    ME: ROUTER.add({ url: "me", parser: {}, render: () => <Me /> }),
};

ROUTER.finish();
