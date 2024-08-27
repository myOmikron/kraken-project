import React from "react";
import USER_CONTEXT from "../../../context/user";
import { ROUTES } from "../../../routes";
import "../../../styling/workspace-menu.css";
import AttackIcon from "../../../svg/attack";
import DataIcon from "../../../svg/data";
import FindingIcon from "../../../svg/finding";
import HostIcon from "../../../svg/host";
import LibraryIcon from "../../../svg/library";
import SearchIcon from "../../../svg/search";
import SettingsIcon from "../../../svg/settings";
import { WorkspaceView } from "../workspace";

type WorkspaceMenuProps = {
    uuid: string;
    owner: string;
    active: WorkspaceView;
};

export default function WorkspaceMenu(props: WorkspaceMenuProps) {
    const {
        user: { uuid: user },
    } = React.useContext(USER_CONTEXT);

    return (
        <div className={"workspace-menu pane"}>
            <div
                className={props.active === "notes" ? "workspace-menu-item active" : "workspace-menu-item"}
                {...ROUTES.WORKSPACE_NOTES.clickHandler({ uuid: props.uuid })}
            >
                <LibraryIcon /> {/* TODO: find better icon */}
                <div className={"workspace-menu-hint"}>Notes</div>
            </div>
            <div
                className={props.active === "search" ? "workspace-menu-item active" : "workspace-menu-item"}
                {...ROUTES.WORKSPACE_SEARCH.clickHandler({ uuid: props.uuid })}
            >
                <SearchIcon />
                <div className={"workspace-menu-hint"}>Search</div>
            </div>
            <div
                className={props.active === "attacks" ? "workspace-menu-item active" : "workspace-menu-item"}
                {...ROUTES.WORKSPACE_ATTACKS.clickHandler({ uuid: props.uuid })}
            >
                <AttackIcon />
                <div className={"workspace-menu-hint"}>Attacks</div>
            </div>
            <div
                className={props.active === "findings" ? "workspace-menu-item active" : "workspace-menu-item"}
                {...ROUTES.WORKSPACE_FINDINGS_LIST.clickHandler({ uuid: props.uuid })}
                style={{ padding: 0 }}
            >
                <FindingIcon />
                <div className={"workspace-menu-hint"}>Findings</div>
            </div>
            <div
                className={props.active === "data" ? "workspace-menu-item active" : "workspace-menu-item"}
                {...ROUTES.WORKSPACE_DATA.clickHandler({ uuid: props.uuid })}
            >
                <DataIcon />
                <div className={"workspace-menu-hint"}>Data</div>
            </div>
            <div
                className={props.active === "hosts" ? "workspace-menu-item active" : "workspace-menu-item"}
                {...ROUTES.WORKSPACE_HOSTS.clickHandler({ uuid: props.uuid })}
            >
                <HostIcon />
                <div className={"workspace-menu-hint"}>Hosts</div>
            </div>
            {user === props.owner ? (
                <div
                    className={props.active === "settings" ? "workspace-menu-item active" : "workspace-menu-item"}
                    {...ROUTES.WORKSPACE_SETTINGS.clickHandler({ uuid: props.uuid })}
                >
                    <SettingsIcon />
                    <div className={"workspace-menu-hint"}>Workspace Settings</div>
                </div>
            ) : null}
        </div>
    );
}
