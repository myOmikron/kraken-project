import React from "react";
import SearchIcon from "../../../svg/search";
import AttackIcon from "../../../svg/attack";
import DataIcon from "../../../svg/data";
import SettingsIcon from "../../../svg/settings";
import { ROUTES } from "../../../routes";
import "../../../styling/workspace-menu.css";
import HostIcon from "../../../svg/host";
import { WorkspaceView } from "../workspace";
import USER_CONTEXT from "../../../context/user";
import LibraryIcon from "../../../svg/library";
import FindingIcon from "../../../svg/finding";

type WorkspaceMenuProps = {
    uuid: string;
    owner: string;
    active: WorkspaceView;
};
type WorkspaceMenuState = {};

export default class WorkspaceMenu extends React.Component<WorkspaceMenuProps, WorkspaceMenuState> {
    static contextType = USER_CONTEXT;
    declare context: React.ContextType<typeof USER_CONTEXT>;
    render() {
        return (
            <div className={"workspace-menu pane"}>
                <div
                    className={this.props.active === "notes" ? "workspace-menu-item active" : "workspace-menu-item"}
                    {...ROUTES.WORKSPACE_NOTES.clickHandler({ uuid: this.props.uuid })}
                >
                    <LibraryIcon /> {/* TODO: find better icon */}
                    <div className={"workspace-menu-hint"}>Notes</div>
                </div>
                <div
                    className={this.props.active === "search" ? "workspace-menu-item active" : "workspace-menu-item"}
                    {...ROUTES.WORKSPACE_SEARCH.clickHandler({ uuid: this.props.uuid })}
                >
                    <SearchIcon />
                    <div className={"workspace-menu-hint"}>Search</div>
                </div>
                <div
                    className={this.props.active === "attacks" ? "workspace-menu-item active" : "workspace-menu-item"}
                    {...ROUTES.WORKSPACE_ATTACKS.clickHandler({ uuid: this.props.uuid })}
                >
                    <AttackIcon />
                    <div className={"workspace-menu-hint"}>Attacks</div>
                </div>
                <div
                    className={this.props.active === "findings" ? "workspace-menu-item active" : "workspace-menu-item"}
                    {...ROUTES.WORKSPACE_FINDINGS.clickHandler({ uuid: this.props.uuid })}
                    style={{ padding: 0 }}
                >
                    <FindingIcon />
                    <div className={"workspace-menu-hint"}>Findings</div>
                </div>
                <div
                    className={this.props.active === "data" ? "workspace-menu-item active" : "workspace-menu-item"}
                    {...ROUTES.WORKSPACE_DATA.clickHandler({ uuid: this.props.uuid })}
                >
                    <DataIcon />
                    <div className={"workspace-menu-hint"}>Data</div>
                </div>
                <div
                    className={this.props.active === "hosts" ? "workspace-menu-item active" : "workspace-menu-item"}
                    {...ROUTES.WORKSPACE_HOSTS.clickHandler({ uuid: this.props.uuid })}
                >
                    <HostIcon />
                    <div className={"workspace-menu-hint"}>Hosts</div>
                </div>
                {this.context.user.uuid === this.props.owner ? (
                    <div
                        className={
                            this.props.active === "settings" ? "workspace-menu-item active" : "workspace-menu-item"
                        }
                        {...ROUTES.WORKSPACE_SETTINGS.clickHandler({ uuid: this.props.uuid })}
                    >
                        <SettingsIcon />
                        <div className={"workspace-menu-hint"}>Workspace Settings</div>
                    </div>
                ) : null}
            </div>
        );
    }
}
