import React from "react";
import SearchIcon from "../../svg/search";
import AttackIcon from "../../svg/attack";
import DataIcon from "../../svg/data";
import SettingsIcon from "../../svg/settings";
import { ROUTES } from "../../routes";

type WorkspaceMenuItem = "search" | "attacks" | "data" | "workspace_settings";

type WorkspaceMenuProps = {
    uuid: string;
};
type WorkspaceMenuState = {
    active: WorkspaceMenuItem;
};

export default class WorkspaceMenu extends React.Component<WorkspaceMenuProps, WorkspaceMenuState> {
    constructor(props: WorkspaceMenuProps) {
        super(props);

        this.state = {
            active: "data",
        };
    }

    render() {
        return (
            <div className={"workspace-menu pane"}>
                <div
                    className={this.state.active === "search" ? "workspace-menu-item active" : "workspace-menu-item"}
                    onClick={() => {
                        this.setState({ active: "search" });
                        ROUTES.WORKSPACE_SEARCH.visit({ uuid: this.props.uuid });
                    }}
                    onAuxClick={() => {
                        this.setState({ active: "search" });
                        ROUTES.WORKSPACE_SEARCH.open({ uuid: this.props.uuid });
                    }}
                >
                    <SearchIcon />
                    <div className={"workspace-menu-hint"}>Search</div>
                </div>
                <div
                    className={this.state.active === "attacks" ? "workspace-menu-item active" : "workspace-menu-item"}
                    onClick={() => {
                        this.setState({ active: "attacks" });
                        ROUTES.WORKSPACE_ATTACKS.visit({ uuid: this.props.uuid });
                    }}
                    onAuxClick={() => {
                        this.setState({ active: "attacks" });
                        ROUTES.WORKSPACE_ATTACKS.open({ uuid: this.props.uuid });
                    }}
                >
                    <AttackIcon />
                    <div className={"workspace-menu-hint"}>Attacks</div>
                </div>
                <div
                    className={this.state.active === "data" ? "workspace-menu-item active" : "workspace-menu-item"}
                    onClick={() => {
                        this.setState({ active: "data" });
                        ROUTES.WORKSPACE_DATA.visit({ uuid: this.props.uuid });
                    }}
                    onAuxClick={() => {
                        this.setState({ active: "data" });
                        ROUTES.WORKSPACE_DATA.open({ uuid: this.props.uuid });
                    }}
                >
                    <DataIcon />
                    <div className={"workspace-menu-hint"}>Data</div>
                </div>
                <div
                    className={
                        this.state.active === "workspace_settings"
                            ? "workspace-menu-item active"
                            : "workspace-menu-item"
                    }
                    onClick={() => {
                        this.setState({ active: "workspace_settings" });
                        ROUTES.WORKSPACE_SETTINGS.visit({ uuid: this.props.uuid });
                    }}
                    onAuxClick={() => {
                        this.setState({ active: "workspace_settings" });
                        ROUTES.WORKSPACE_SETTINGS.open({ uuid: this.props.uuid });
                    }}
                >
                    <SettingsIcon />
                    <div className={"workspace-menu-hint"}>Workspace Settings</div>
                </div>
            </div>
        );
    }
}
