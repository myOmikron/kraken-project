import React from "react";
import { ROUTES } from "../routes";
import USER_CONTEXT from "../context/user";

type MenuProps = {};
type MenuState = {};

export default class Menu extends React.Component<MenuProps, MenuState> {
    state = {};

    static contextType = USER_CONTEXT;
    declare context: React.ContextType<typeof USER_CONTEXT>;

    render() {
        return (
            <div className="menu pane">
                <div className="menu-item pane" {...ROUTES.ME.clickHandler({})}>
                    Me
                </div>
                <div className="menu-item pane" {...ROUTES.WORKSPACES.clickHandler({})}>
                    My Workspaces
                </div>
                <div className="menu-item pane" {...ROUTES.ATTACKS.clickHandler({})}>
                    Start attacks
                </div>
                {this.context.user.admin ? (
                    <>
                        <div className="menu-item pane" {...ROUTES.KRAKEN_NETWORK.clickHandler({})}>
                            Kraken Network
                            <small>{"<Admin>"}</small>
                        </div>
                        <div className="menu-item pane" {...ROUTES.ADMIN_USER_MANAGEMENT.clickHandler({})}>
                            Users
                            <small>{"<Admin>"}</small>
                        </div>
                        <div className="menu-item pane" {...ROUTES.ADMIN_WORKSPACE_MANAGEMENT.clickHandler({})}>
                            Workspaces
                            <small>{"<Admin>"}</small>
                        </div>
                    </>
                ) : null}
            </div>
        );
    }
}
