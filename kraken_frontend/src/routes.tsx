import Home from "./views/home";
import React from "react";
import { Router } from "./utils/router";

export const ROUTER = new Router();

export const ROUTES = {
    HOME: ROUTER.add({ url: "", parser: {}, render: () => <Home /> }),
};

ROUTER.finish();
