import React from "react";
import { ROUTES } from "../routes";
import USER_CONTEXT from "../context/user";
import AttackIcon from "../svg/attack";
import KrakenIcon from "../svg/kraken";
import KnowledgeIcon from "../svg/knowledge";
import WorkspaceIcon from "../svg/workspace";
import UsersIcon from "../svg/users";
import UserSettingsIcon from "../svg/user_settings";
import "../styling/menu.css";
import SettingsIcon from "../svg/settings";
import { UserPermission } from "../api/generated";
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

type MenuProps = {};
type MenuState = {
    active: MenuItem;
};

export default class Menu extends React.Component<MenuProps, MenuState> {
    state: MenuState = {
        active: "workspaces",
    };

    static contextType = USER_CONTEXT;
    declare context: React.ContextType<typeof USER_CONTEXT>;

    render() {
        return (
            <div className="menu pane">
                <div className={"menu-header"}>
                    <KrakenIcon />
                </div>
                <div className={"menu-seperator"}>Workspaces</div>
                <div className={"menu-item-container"}>
                    <div
                        className={this.state.active === "workspaces" ? "menu-item active" : "menu-item"}
                        onClick={() => {
                            this.setState({ active: "workspaces" });
                            ROUTES.WORKSPACES.visit({});
                        }}
                        onAuxClick={() => {
                            this.setState({ active: "workspaces" });
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
                        className={this.state.active === "knowledge" ? "menu-item active" : "menu-item"}
                        onClick={() => {
                            this.setState({ active: "knowledge" });
                            ROUTES.KNOWLEDGE_BASE.visit({});
                        }}
                        onAuxClick={() => {
                            this.setState({ active: "knowledge" });
                            ROUTES.KNOWLEDGE_BASE.open({});
                        }}
                    >
                        <KnowledgeIcon />
                        <div className={"menu-hint"}>Knowledge Base</div>
                    </div>
                </div>
                <div className={"menu-item-container"}>
                    <div
                        className={this.state.active === "me" ? "menu-item active" : "menu-item"}
                        onClick={() => {
                            this.setState({ active: "me" });
                            ROUTES.ME.visit({});
                        }}
                        onAuxClick={() => {
                            this.setState({ active: "me" });
                            ROUTES.ME.open({});
                        }}
                    >
                        <UserSettingsIcon />
                        <div className={"menu-hint"}>User Settings</div>
                    </div>
                </div>
                {this.context.user.permission === UserPermission.Admin ? (
                    <>
                        <div className={"menu-seperator"}>Admin</div>
                        <div className={"menu-item-container"}>
                            <div
                                className={this.state.active === "kraken" ? "menu-item active" : "menu-item"}
                                onClick={() => {
                                    this.setState({ active: "kraken" });
                                    ROUTES.KRAKEN_NETWORK.visit({});
                                }}
                                onAuxClick={() => {
                                    this.setState({ active: "kraken" });
                                    ROUTES.KRAKEN_NETWORK.open({});
                                }}
                            >
                                <KrakenIcon />
                                <div className={"menu-hint"}>Kraken Network</div>
                            </div>
                        </div>
                        <div className={"menu-item-container"}>
                            <div
                                className={this.state.active === "users_admin" ? "menu-item active" : "menu-item"}
                                onClick={() => {
                                    this.setState({ active: "users_admin" });
                                    ROUTES.ADMIN_USER_MANAGEMENT.visit({});
                                }}
                                onAuxClick={() => {
                                    this.setState({ active: "users_admin" });
                                    ROUTES.ADMIN_USER_MANAGEMENT.open({});
                                }}
                            >
                                <UsersIcon />
                                <div className={"menu-hint"}>User Controls</div>
                            </div>
                        </div>
                        <div className={"menu-item-container"}>
                            <div
                                className={this.state.active === "workspaces_admin" ? "menu-item active" : "menu-item"}
                                onClick={() => {
                                    this.setState({ active: "workspaces_admin" });
                                    ROUTES.ADMIN_WORKSPACE_MANAGEMENT.visit({});
                                }}
                                onAuxClick={() => {
                                    this.setState({ active: "workspaces_admin" });
                                    ROUTES.ADMIN_WORKSPACE_MANAGEMENT.open({});
                                }}
                            >
                                <WorkspaceIcon />
                                <div className={"menu-hint"}>Workspace Controls</div>
                            </div>
                        </div>
                        <div className={"menu-item-container"}>
                            <div
                                className={this.state.active === "settings" ? "menu-item active" : "menu-item"}
                                onClick={() => {
                                    this.setState({ active: "settings" });
                                    ROUTES.ADMIN_SETTINGS.visit({});
                                }}
                                onAuxClick={() => {
                                    this.setState({ active: "settings" });
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
