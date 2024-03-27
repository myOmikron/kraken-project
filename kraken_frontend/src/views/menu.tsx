import React from "react";
import { UserPermission } from "../api/generated";
import USER_CONTEXT from "../context/user";
import { ROUTES } from "../routes";
import "../styling/menu.css";
import KnowledgeIcon from "../svg/knowledge";
import KrakenIcon from "../svg/kraken";
import SettingsIcon from "../svg/settings";
import UserSettingsIcon from "../svg/user_settings";
import UsersIcon from "../svg/users";
import WorkspaceIcon from "../svg/workspace";
import RunningAttacks from "./running-attacks";

type MenuItem =
    | "me"
    | "workspaces"
    | "attack"
    | "kraken"
    | "users_admin"
    | "workspaces_admin"
    | "knowledge"
    | "settings";

function Menu() {
    const context = React.useContext(USER_CONTEXT);
    const [active, setActive] = React.useState<MenuItem>("workspaces");

    return (
        <div className="menu pane">
            <div className={"menu-header"}>
                <KrakenIcon />
            </div>
            <div className={"menu-seperator"}>Workspaces</div>
            <div className={"menu-item-container"}>
                <div
                    className={active === "workspaces" ? "menu-item active" : "menu-item"}
                    onClick={() => {
                        setActive("workspaces");
                        ROUTES.WORKSPACES.visit({});
                    }}
                    onAuxClick={() => {
                        setActive("workspaces");
                        ROUTES.WORKSPACES.open({});
                    }}
                >
                    <WorkspaceIcon />
                    <div className={"menu-hint"}>Workspaces</div>
                </div>
            </div>
            <div className={"menu-seperator"}>General</div>
            <div className={"menu-item-container"}>
                <div
                    className={active === "knowledge" ? "menu-item active" : "menu-item"}
                    onClick={() => {
                        setActive("knowledge");
                        ROUTES.FINDING_DEFINITION_LIST.visit({});
                    }}
                    onAuxClick={() => {
                        setActive("knowledge");
                        ROUTES.FINDING_DEFINITION_LIST.open({});
                    }}
                >
                    <KnowledgeIcon />
                    <div className={"menu-hint"}>Knowledge Base</div>
                </div>
            </div>
            <div className={"menu-item-container"}>
                <div
                    className={active === "me" ? "menu-item active" : "menu-item"}
                    onClick={() => {
                        setActive("me");
                        ROUTES.ME.visit({});
                    }}
                    onAuxClick={() => {
                        setActive("me");
                        ROUTES.ME.open({});
                    }}
                >
                    <UserSettingsIcon />
                    <div className={"menu-hint"}>User Settings</div>
                </div>
            </div>
            {context.user.permission === UserPermission.Admin ? (
                <>
                    <div className={"menu-seperator"}>Admin</div>
                    <div className={"menu-item-container"}>
                        <div
                            className={active === "kraken" ? "menu-item active" : "menu-item"}
                            onClick={() => {
                                setActive("kraken");
                                ROUTES.KRAKEN_NETWORK.visit({});
                            }}
                            onAuxClick={() => {
                                setActive("kraken");
                                ROUTES.KRAKEN_NETWORK.open({});
                            }}
                        >
                            <KrakenIcon />
                            <div className={"menu-hint"}>Kraken Network</div>
                        </div>
                    </div>
                    <div className={"menu-item-container"}>
                        <div
                            className={active === "users_admin" ? "menu-item active" : "menu-item"}
                            onClick={() => {
                                setActive("users_admin");
                                ROUTES.ADMIN_USER_MANAGEMENT.visit({});
                            }}
                            onAuxClick={() => {
                                setActive("users_admin");
                                ROUTES.ADMIN_USER_MANAGEMENT.open({});
                            }}
                        >
                            <UsersIcon />
                            <div className={"menu-hint"}>User Controls</div>
                        </div>
                    </div>
                    <div className={"menu-item-container"}>
                        <div
                            className={active === "workspaces_admin" ? "menu-item active" : "menu-item"}
                            onClick={() => {
                                setActive("workspaces_admin");
                                ROUTES.ADMIN_WORKSPACE_MANAGEMENT.visit({});
                            }}
                            onAuxClick={() => {
                                setActive("workspaces_admin");
                                ROUTES.ADMIN_WORKSPACE_MANAGEMENT.open({});
                            }}
                        >
                            <WorkspaceIcon />
                            <div className={"menu-hint"}>Workspace Controls</div>
                        </div>
                    </div>
                    <div className={"menu-item-container"}>
                        <div
                            className={active === "settings" ? "menu-item active" : "menu-item"}
                            onClick={() => {
                                setActive("settings");
                                ROUTES.ADMIN_SETTINGS.visit({});
                            }}
                            onAuxClick={() => {
                                setActive("settings");
                                ROUTES.ADMIN_SETTINGS.open({});
                            }}
                        >
                            <SettingsIcon />
                            <div className={"menu-hint"}>Kraken Settings</div>
                        </div>
                    </div>
                </>
            ) : null}
        </div>
    );
}

type ContentWithMenuProps = {
    children: React.ReactNode;
};

export function ContentWithMenu(props: ContentWithMenuProps) {
    return (
        <div className={"base-layout"}>
            <div className={"content-container"}>{props.children}</div>
            <Menu />
            <div className={"top-bar"}>
                <RunningAttacks />
                <div className={"workspace-selector-container pane"}>
                    <WorkspaceIcon />
                </div>
            </div>
        </div>
    );
}
