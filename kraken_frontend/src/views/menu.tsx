import React from "react";
import { ROUTES } from "../routes";
import USER_CONTEXT from "../context/user";
import AttackIcon from "../svg/attack";
import KrakenIcon from "../svg/kraken";
import KnowledgeIcon from "../svg/knowledge";

type MenuItem = "me" | "workspaces" | "attack" | "kraken" | "users_admin" | "workspaces_admin" | "knowledge";

type MenuProps = {};
type MenuState = {
    active: MenuItem;
};

export default class Menu extends React.Component<MenuProps, MenuState> {
    state: MenuState = {
        active: "attack",
    };

    static contextType = USER_CONTEXT;
    declare context: React.ContextType<typeof USER_CONTEXT>;

    render() {
        return (
            <div className="menu pane">
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
                    Me
                </div>
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
                    My Workspaces
                </div>
                <div className={"menu-item-container"}>
                    <div
                        className={this.state.active === "attack" ? "menu-item active" : "menu-item"}
                        onClick={() => {
                            this.setState({ active: "attack" });
                            ROUTES.ATTACKS.visit({});
                        }}
                        onAuxClick={() => {
                            this.setState({ active: "attack" });
                            ROUTES.ATTACKS.open({});
                        }}
                    >
                        <AttackIcon />
                    </div>
                </div>
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
                    </div>
                </div>
                {this.context.user.admin ? (
                    <>
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
                            </div>
                        </div>
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
                            Users
                            <small>{"<Admin>"}</small>
                        </div>
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
                            Workspaces
                            <small>{"<Admin>"}</small>
                        </div>
                    </>
                ) : null}
            </div>
        );
    }
}
